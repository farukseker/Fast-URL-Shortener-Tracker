use std::{net::SocketAddr, sync::LazyLock, time::Duration};

use axum::{
    extract::{ConnectInfo, Query, Request, State},
    http::{HeaderMap, StatusCode},
    response::Redirect,
    Json,
};
use regex::Regex;
use serde_json::{json, Value};
use sqlx::PgPool;
use tracing::error;

use crate::{auth::require_admin, state::{AppState, VisitAction}};

static BOT_USER_AGENT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)bot|crawler|spider|crawling|slurp|bingpreview|facebookexternalhit|github")
        .expect("valid bot user-agent regex")
});

pub(crate) async fn redirect_short_url(
    axum::extract::Path(code): axum::extract::Path<String>,
    State(state): State<AppState>,
    request: Request,
) -> Result<Redirect, StatusCode> {
    let cached = state.urls.get(&code).await.ok_or(StatusCode::NOT_FOUND)?;
    let headers = request.headers();
    let user_agent = headers
        .get("user-agent")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_owned();
    let skip_analytics = BOT_USER_AGENT.is_match(&user_agent);
    let forwarded_ip = headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    let remote_ip = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(address)| address.ip().to_string());
    let ip_address = forwarded_ip.or(remote_ip).unwrap_or_else(|| "unknown".to_owned());

    if !skip_analytics {
        let pool = state.pool.clone();
        let query_host = state.ip_query_host.clone();
        tokio::spawn(async move {
            if let Err(error) = record_visit(pool, query_host, cached.id, ip_address, user_agent).await {
                error!("failed to record URL visit: {error}");
            }
        });
    }

    Ok(Redirect::temporary(&cached.url))
}

async fn record_visit(
    pool: PgPool,
    query_host: Option<String>,
    url_id: i32,
    ip_address: String,
    user_agent: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let geo_data = lookup_ip(query_host, &ip_address).await;
    let recent_id: Option<i32> = sqlx::query_scalar(
        "SELECT id FROM url_visit_actions \
         WHERE url_id = $1 AND ip_address = $2 AND action_at >= NOW() - INTERVAL '1 hour' \
         ORDER BY action_at DESC LIMIT 1",
    )
    .bind(url_id)
    .bind(&ip_address)
    .fetch_optional(&pool)
    .await?;

    if let Some(id) = recent_id {
        sqlx::query("UPDATE url_visit_actions SET count = count + 1, action_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&pool)
            .await?;
    } else {
        let ip_host = geo_data.get("host").and_then(Value::as_str).map(str::to_owned);
        let ip_provider = geo_data.get("isp").and_then(Value::as_str).map(str::to_owned);
        let country = geo_data.get("country").and_then(Value::as_str).map(str::to_owned);
        let city = geo_data.get("city").and_then(Value::as_str).map(str::to_owned);
        sqlx::query(
            "INSERT INTO url_visit_actions \
             (url_id, ip_address, ip_host, ip_provider, country, city, geo_data, user_agent, count) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(url_id)
        .bind(&ip_address)
        .bind(ip_host)
        .bind(ip_provider)
        .bind(country)
        .bind(city)
        .bind(if geo_data.is_null() { None } else { Some(geo_data) })
        .bind(user_agent)
        .bind(1_i32)
        .execute(&pool)
        .await?;
    }
    Ok(())
}

async fn lookup_ip(query_host: Option<String>, ip_address: &str) -> Value {
    let Some(host) = query_host.filter(|host| !host.is_empty()) else {
        report_ip_lookup_failure("missing_query_host");
        return Value::Null;
    };
    if ip_address == "unknown" {
        return Value::Null;
    }
    let url = format!("{}/{}", host.trim_end_matches('/'), ip_address);
    let response = match reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
    {
        Ok(client) => client.get(url).send().await,
        Err(_) => {
            report_ip_lookup_failure("client_build_failed");
            return Value::Null;
        }
    };
    let Ok(response) = response else {
        report_ip_lookup_failure("request_failed");
        return Value::Null;
    };
    if !response.status().is_success() {
        report_ip_lookup_failure("http_status_not_success");
        return Value::Null;
    }
    let Ok(data) = response.json::<Value>().await else {
        report_ip_lookup_failure("invalid_json_response");
        return Value::Null;
    };
    if data.get("status").and_then(Value::as_str) != Some("success") {
        report_ip_lookup_failure("provider_status_not_success");
        return Value::Null;
    }
    json!({
        "ip": data.get("query"),
        "country": data.get("country"),
        "countryCode": data.get("countryCode"),
        "region": data.get("regionName"),
        "city": data.get("city"),
        "lat": data.get("lat"),
        "lon": data.get("lon"),
        "isp": data.get("isp"),
        "org": data.get("org"),
        "asn": data.get("as"),
        "host": data.get("org"),
    })
}

fn report_ip_lookup_failure(reason: &'static str) {
    tracing::warn!(reason, "IP geolocation query failed");
    sentry::with_scope(
        |scope| {
            scope.set_tag("component", "ip_geolocation");
            scope.set_tag("failure_reason", reason);
        },
        || sentry::capture_message("IP geolocation query failed", sentry::Level::Error),
    );
}

pub(crate) async fn list_visit_actions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    require_admin(&headers, &state)?;
    let url_id = query
        .get("url_id")
        .and_then(|value| value.parse::<i32>().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;
    let actions = sqlx::query_as::<_, VisitAction>(
        "SELECT id, url_id, ip_address, ip_host, ip_provider, country, city, geo_data, \
         user_agent, count, action_at FROM url_visit_actions WHERE url_id = $1 ORDER BY action_at",
    )
    .bind(url_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!(actions.into_iter().map(|action| json!({
        "id": action.id,
        "url_id": action.url_id,
        "ip_address": action.ip_address,
        "ip_host": action.ip_host,
        "ip_provider": action.ip_provider,
        "country": action.country,
        "city": action.city,
        "geo_data": action.geo_data,
        "user_agent": action.user_agent,
        "count": action.count,
        "action_at": action.action_at,
    })).collect::<Vec<_>>())) )
}

#[cfg(test)]
mod tests {
    use std::{
        str::FromStr,
        time::{SystemTime, UNIX_EPOCH},
    };

    use sqlx::{
        postgres::{PgConnectOptions, PgPoolOptions},
        Row,
    };

    use super::{record_visit, BOT_USER_AGENT};

    #[test]
    fn bot_user_agents_are_recognized_case_insensitively() {
        assert!(BOT_USER_AGENT.is_match("GoogleBot/2.1"));
        assert!(!BOT_USER_AGENT.is_match("Mozilla/5.0"));
    }

    #[tokio::test]
    async fn visit_without_count_input_is_inserted_with_count_one() {
        let Ok(database_url) = std::env::var("TEST_DATABASE_URL") else {
            eprintln!("skipping PostgreSQL visit test: TEST_DATABASE_URL is not set");
            return;
        };
        let admin_pool = PgPoolOptions::new()
            .connect(&database_url)
            .await
            .expect("connect to test PostgreSQL database");
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_nanos();
        let schema = format!("analytics_test_{}_{}", std::process::id(), timestamp);
        sqlx::query(&format!("CREATE SCHEMA {schema}"))
            .execute(&admin_pool)
            .await
            .expect("create isolated test schema");

        let options = PgConnectOptions::from_str(&database_url)
            .expect("valid PostgreSQL URL")
            .options([("search_path", schema.as_str())]);
        let pool = PgPoolOptions::new()
            .connect_with(options)
            .await
            .expect("connect to isolated test schema");
        let result = async {
            sqlx::query(
                "CREATE TABLE url_visit_actions (\
                    id SERIAL PRIMARY KEY,\
                    url_id INTEGER NOT NULL,\
                    ip_address VARCHAR(45) NOT NULL,\
                    ip_host TEXT,\
                    ip_provider TEXT,\
                    country TEXT,\
                    city TEXT,\
                    geo_data JSONB,\
                    user_agent TEXT NOT NULL,\
                    count INTEGER,\
                    action_at TIMESTAMPTZ NOT NULL DEFAULT NOW()\
                )",
            )
            .execute(&pool)
            .await?;
            sqlx::query(
                "INSERT INTO url_visit_actions (url_id, ip_address, user_agent, count) \
                 VALUES (1, '198.51.100.1', 'test-agent', NULL)",
            )
            .execute(&pool)
            .await?;
            crate::database::initialize(&pool).await?;
            record_visit(
                pool.clone(),
                None,
                1,
                "198.51.100.1".to_owned(),
                "test-agent".to_owned(),
            )
            .await?;
            record_visit(
                pool.clone(),
                None,
                1,
                "198.51.100.2".to_owned(),
                "test-agent".to_owned(),
            )
            .await?;
            let rows = sqlx::query(
                "SELECT ip_address, count FROM url_visit_actions ORDER BY ip_address",
            )
            .fetch_all(&pool)
            .await?;
            let migrated_count = rows[0].get("count");
            let inserted_count = rows[1].get("count");
            Ok::<(i32, i32), Box<dyn std::error::Error + Send + Sync>>(
                (migrated_count, inserted_count),
            )
        }
        .await;

        pool.close().await;
        sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
            .execute(&admin_pool)
            .await
            .expect("drop isolated test schema");
        admin_pool.close().await;
        assert_eq!(result.expect("record visits in isolated schema"), (2, 1));
    }
}