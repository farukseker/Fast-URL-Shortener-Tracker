use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde_json::{json, Value};
use sqlx::Row;

use crate::{auth::require_admin, database::is_unique_violation, state::AppState};

pub(crate) async fn create_category(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    require_admin(&headers, &state)?;
    let name = query.get("name").ok_or(StatusCode::BAD_REQUEST)?;
    let slug = query.get("slug").ok_or(StatusCode::BAD_REQUEST)?;
    let result = sqlx::query("INSERT INTO categories (name, slug) VALUES ($1, $2) RETURNING id")
        .bind(name)
        .bind(slug)
        .fetch_one(&state.pool)
        .await;
    match result {
        Ok(row) => Ok(Json(json!({ "id": row.get::<i32, _>("id"), "name": name, "slug": slug }))),
        Err(error) if is_unique_violation(&error) => Err(StatusCode::BAD_REQUEST),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub(crate) async fn list_categories(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, StatusCode> {
    require_admin(&headers, &state)?;
    let rows = sqlx::query("SELECT id, name, slug FROM categories ORDER BY id")
        .fetch_all(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(Value::Array(
        rows.into_iter()
            .map(|row| json!({
                "id": row.get::<i32, _>("id"),
                "name": row.get::<String, _>("name"),
                "slug": row.get::<String, _>("slug"),
            }))
            .collect(),
    )))
}

pub(crate) async fn update_category(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(category_id): Path<i32>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    require_admin(&headers, &state)?;
    let name = query.get("name").filter(|value| !value.is_empty());
    let slug = query.get("slug").filter(|value| !value.is_empty());
    let result = sqlx::query(
        "UPDATE categories SET name = COALESCE($1, name), slug = COALESCE($2, slug) WHERE id = $3",
    )
    .bind(name)
    .bind(slug)
    .bind(category_id)
    .execute(&state.pool)
    .await
    .map_err(|error| {
        if is_unique_violation(&error) {
            StatusCode::BAD_REQUEST
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;
    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(Json(json!({ "status": "updated" })))
}

pub(crate) async fn delete_category(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(category_id): Path<i32>,
) -> Result<Json<Value>, StatusCode> {
    require_admin(&headers, &state)?;
    let result = sqlx::query("DELETE FROM categories WHERE id = $1")
        .bind(category_id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(Json(json!({ "status": "deleted" })))
}