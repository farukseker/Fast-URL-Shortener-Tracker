use serde::Deserialize;
use serde_json::Value;
use sqlx::PgPool;
use chrono::{DateTime, Utc};

use crate::cache::UrlCache;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) pool: PgPool,
    pub(crate) urls: UrlCache,
    pub(crate) admin_token: String,
    pub(crate) ip_query_host: Option<String>,
}

#[derive(Deserialize, Default)]
pub(crate) struct CreateShortRequest {
    #[serde(default)]
    pub(crate) long_url: String,
    pub(crate) preview_type: Option<String>,
    pub(crate) custom_code: Option<String>,
    #[serde(default)]
    pub(crate) category_ids: Vec<i32>,
}

#[derive(Deserialize)]
pub(crate) struct UpdateShortRequest {
    pub(crate) id: i32,
    pub(crate) long_url: String,
    pub(crate) preview_type: Option<String>,
    pub(crate) custom_code: Option<String>,
    pub(crate) category_ids: Option<Vec<i32>>,
}

#[derive(sqlx::FromRow)]
pub(crate) struct VisitAction {
    pub(crate) id: i32,
    pub(crate) url_id: i32,
    pub(crate) ip_address: String,
    pub(crate) ip_host: Option<String>,
    pub(crate) ip_provider: Option<String>,
    pub(crate) country: Option<String>,
    pub(crate) city: Option<String>,
    pub(crate) geo_data: Option<Value>,
    pub(crate) user_agent: String,
    pub(crate) count: i32,
    pub(crate) action_at: DateTime<Utc>,
}