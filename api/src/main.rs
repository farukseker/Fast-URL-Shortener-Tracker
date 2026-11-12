use std::net::SocketAddr;

use axum::{http::StatusCode, routing::{get, post, put}, Router};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use tracing::info;

mod analytics;
mod auth;
mod cache;
mod categories;
mod database;
mod short_urls;
mod state;

use cache::UrlCache;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let _sentry_guard = std::env::var("SENTRY_DSN")
        .ok()
        .filter(|dsn| !dsn.trim().is_empty())
        .map(|dsn| {
            let mut options = sentry::ClientOptions::default();
            options.release = sentry::release_name!();
            sentry::init((dsn, options))
        });
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/urlshort".to_owned())
        .replace("postgresql+asyncpg://", "postgres://")
        .replace("postgresql+psycopg2://", "postgres://");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;
    database::initialize(&pool).await?;

    let urls = database::load_url_cache(&pool).await?;
    let state = AppState {
        pool,
        urls: UrlCache::new(urls),
        admin_token: std::env::var("ADMIN_TOKEN").unwrap_or_default(),
        ip_query_host: std::env::var("API_QUERY_HOST").ok(),
    };

    let app = Router::new()
        .route("/healthz", get(|| async { StatusCode::OK }))
        .route("/r/{code}", get(analytics::redirect_short_url))
        .route("/admin/create", post(short_urls::create_short_url))
        .route("/admin/list", get(short_urls::list_short_urls))
        .route(
            "/admin/url",
            get(short_urls::get_short_url)
                .put(short_urls::update_short_url)
                .delete(short_urls::delete_short_url),
        )
        .route(
            "/admin/category",
            post(categories::create_category).get(categories::list_categories),
        )
        .route(
            "/admin/category/",
            post(categories::create_category).get(categories::list_categories),
        )
        .route(
            "/admin/category/{category_id}",
            put(categories::update_category).delete(categories::delete_category),
        )
        .route(
            "/admin/analytical/list",
            get(analytics::list_visit_actions),
        )
        .layer(CorsLayer::permissive())
        .with_state(state);

    let address = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_owned());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_owned())
        .parse::<u16>()?;
    let listener = tokio::net::TcpListener::bind(format!("{address}:{port}")).await?;
    info!("URL tracker listening on {address}:{port}");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
