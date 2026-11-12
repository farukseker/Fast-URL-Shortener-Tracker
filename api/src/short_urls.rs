use std::collections::HashMap;

use axum::{
    body::Bytes,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use sqlx::Row;

use crate::{
    auth::{new_short_code, require_admin},
    cache::CachedUrl,
    database::{category_ids, is_unique_violation},
    state::{AppState, CreateShortRequest, UpdateShortRequest},
};

pub(crate) async fn create_short_url(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    body: Bytes,
) -> Result<Json<Value>, StatusCode> {
    require_admin(&headers, &state)?;
    let payload: CreateShortRequest = if body.is_empty() {
        CreateShortRequest {
            long_url: query.get("long_url").cloned().unwrap_or_default(),
            custom_code: query.get("custom_code").cloned(),
            preview_type: query.get("preview_type").cloned(),
            ..Default::default()
        }
    } else {
        serde_json::from_slice(&body).map_err(|_| StatusCode::BAD_REQUEST)?
    };
    if payload.long_url.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let preview_type = payload.preview_type.as_deref().unwrap_or("url");
    if !["url", "image", "video"].contains(&preview_type) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let code = payload
        .custom_code
        .clone()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(new_short_code);
    if code.len() > 16 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut transaction = state.pool.begin().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let inserted = sqlx::query(
        "INSERT INTO short_urls (short_code, long_url, custom, preview_type) \
         VALUES ($1, $2, $3, $4::preview_type_enum) RETURNING id",
    )
    .bind(&code)
    .bind(&payload.long_url)
    .bind(payload.custom_code.as_deref().is_some_and(|value| !value.is_empty()))
    .bind(preview_type)
    .fetch_one(&mut *transaction)
    .await;
    let id: i32 = match inserted {
        Ok(row) => row.get("id"),
        Err(error) if is_unique_violation(&error) => return Err(StatusCode::CONFLICT),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    for category_id in &payload.category_ids {
        sqlx::query("INSERT INTO short_url_categories (short_url_id, category_id) VALUES ($1, $2)")
            .bind(id)
            .bind(category_id)
            .execute(&mut *transaction)
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;
    }
    transaction.commit().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.urls.insert(
        code.clone(),
        CachedUrl {
            id,
            url: payload.long_url,
        },
    ).await;
    Ok(Json(json!({ "short": format!("/r/{code}"), "code": code })))
}

pub(crate) async fn list_short_urls(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, StatusCode> {
    require_admin(&headers, &state)?;
    let rows = sqlx::query(
        "SELECT u.id, u.short_code, u.long_url, u.custom, u.preview_type::text AS preview_type, \
         u.created_at, COALESCE(SUM(a.count), 0)::BIGINT AS click_count \
         FROM short_urls u LEFT JOIN url_visit_actions a ON a.url_id = u.id \
         GROUP BY u.id ORDER BY u.id",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut urls = Vec::with_capacity(rows.len());
    for row in rows {
        let id: i32 = row.get("id");
        urls.push(json!({
            "id": id,
            "code": row.get::<String, _>("short_code"),
            "url": row.get::<String, _>("long_url"),
            "custom": row.get::<bool, _>("custom"),
            "preview_type": row.get::<String, _>("preview_type"),
            "created_at": row.get::<DateTime<Utc>, _>("created_at"),
            "categories": category_ids(&state.pool, id).await?,
            "click_count": row.get::<i64, _>("click_count"),
        }));
    }
    Ok(Json(Value::Array(urls)))
}

pub(crate) async fn get_short_url(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    require_admin(&headers, &state)?;
    let code = query.get("code").ok_or(StatusCode::BAD_REQUEST)?;
    let row = sqlx::query(
        "SELECT id, short_code, long_url, preview_type::text AS preview_type, custom, created_at \
         FROM short_urls WHERE short_code = $1",
    )
    .bind(code)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;
    let id: i32 = row.get("id");
    Ok(Json(json!({
        "id": id,
        "code": row.get::<String, _>("short_code"),
        "url": row.get::<String, _>("long_url"),
        "url_type": row.get::<String, _>("preview_type"),
        "custom": row.get::<bool, _>("custom"),
        "created_at": row.get::<DateTime<Utc>, _>("created_at"),
        "categories": category_ids(&state.pool, id).await?,
    })))
}

pub(crate) async fn update_short_url(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, StatusCode> {
    require_admin(&headers, &state)?;
    let payload: UpdateShortRequest =
        serde_json::from_slice(&body).map_err(|_| StatusCode::BAD_REQUEST)?;
    if payload.long_url.is_empty()
        || payload
            .preview_type
            .as_deref()
            .is_some_and(|value| !["url", "image", "video"].contains(&value))
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let mut transaction = state.pool.begin().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let old = sqlx::query("SELECT short_code FROM short_urls WHERE id = $1")
        .bind(payload.id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let old_code: String = old.get("short_code");
    let code = payload
        .custom_code
        .as_deref()
        .filter(|value| !value.is_empty())
        .unwrap_or(&old_code);
    if code.len() > 16 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let updated = sqlx::query(
        "UPDATE short_urls SET long_url = $1, short_code = $2, \
         preview_type = COALESCE($3::text::preview_type_enum, preview_type), \
         custom = CASE WHEN $4 THEN TRUE ELSE custom END WHERE id = $5",
    )
    .bind(&payload.long_url)
    .bind(code)
    .bind(payload.preview_type.as_deref())
    .bind(payload.custom_code.as_deref().is_some_and(|value| !value.is_empty()))
    .bind(payload.id)
    .execute(&mut *transaction)
    .await;
    if let Err(error) = updated {
        if is_unique_violation(&error) {
            return Err(StatusCode::CONFLICT);
        }
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    if let Some(category_ids) = payload.category_ids {
        sqlx::query("DELETE FROM short_url_categories WHERE short_url_id = $1")
            .bind(payload.id)
            .execute(&mut *transaction)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        for category_id in category_ids {
            sqlx::query("INSERT INTO short_url_categories (short_url_id, category_id) VALUES ($1, $2)")
                .bind(payload.id)
                .bind(category_id)
                .execute(&mut *transaction)
                .await
                .map_err(|_| StatusCode::BAD_REQUEST)?;
        }
    }
    transaction.commit().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.urls.replace(
        &old_code,
        code.to_owned(),
        CachedUrl {
            id: payload.id,
            url: payload.long_url,
        },
    ).await;
    Ok(Json(json!({ "status": "updated" })))
}

pub(crate) async fn delete_short_url(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    require_admin(&headers, &state)?;
    let code = query.get("code").ok_or(StatusCode::BAD_REQUEST)?;
    let result = sqlx::query("DELETE FROM short_urls WHERE short_code = $1")
        .bind(code)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    state.urls.remove(code).await;
    Ok(Json(json!({ "status": "deleted" })))
}

#[cfg(test)]
mod tests {
    use crate::auth::new_short_code;

    #[test]
    fn generated_codes_are_six_alphanumeric_characters() {
        let code = new_short_code();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|character| character.is_ascii_alphanumeric()));
    }
}