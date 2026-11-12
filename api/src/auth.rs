use axum::http::{HeaderMap, StatusCode};
use rand::{distr::Alphanumeric, Rng};

use crate::state::AppState;

pub(crate) fn require_admin(headers: &HeaderMap, state: &AppState) -> Result<(), StatusCode> {
    let token = headers
        .get("x-api-key")
        .and_then(|value| value.to_str().ok());
    if token == Some(state.admin_token.as_str()) && !state.admin_token.is_empty() {
        Ok(())
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub(crate) fn new_short_code() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect()
}