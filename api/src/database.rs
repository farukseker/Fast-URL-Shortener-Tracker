use std::collections::HashMap;

use sqlx::{PgPool, Row};

use crate::cache::CachedUrl;

pub(crate) async fn initialize(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "DO $block$ BEGIN CREATE TYPE preview_type_enum AS ENUM ('video', 'image', 'url'); \
         EXCEPTION WHEN duplicate_object THEN NULL; END $block$;",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS short_urls (\
            id SERIAL PRIMARY KEY,\
            short_code VARCHAR(16) NOT NULL UNIQUE,\
            long_url TEXT NOT NULL,\
            custom BOOLEAN NOT NULL DEFAULT FALSE,\
            preview_type preview_type_enum NOT NULL DEFAULT 'url',\
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()\
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS ix_short_urls_short_code ON short_urls (short_code)")
        .execute(pool)
        .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS categories (\
            id SERIAL PRIMARY KEY,\
            name VARCHAR(64) NOT NULL,\
            slug VARCHAR(64) NOT NULL UNIQUE,\
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()\
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS ix_category_name ON categories (name)")
        .execute(pool)
        .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS short_url_categories (\
            short_url_id INTEGER NOT NULL REFERENCES short_urls(id) ON DELETE CASCADE,\
            category_id INTEGER NOT NULL REFERENCES categories(id) ON DELETE CASCADE,\
            PRIMARY KEY (short_url_id, category_id)\
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS url_visit_actions (\
            id SERIAL PRIMARY KEY,\
            url_id INTEGER NOT NULL,\
            ip_address VARCHAR(45) NOT NULL,\
            ip_host TEXT,\
            ip_provider TEXT,\
            country TEXT,\
            city TEXT,\
            geo_data JSONB,\
            user_agent TEXT NOT NULL,\
            count INTEGER NOT NULL DEFAULT 1,\
            action_at TIMESTAMPTZ NOT NULL DEFAULT NOW()\
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query("UPDATE url_visit_actions SET count = 1 WHERE count IS NULL")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE url_visit_actions ALTER COLUMN count SET DEFAULT 1")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE url_visit_actions ALTER COLUMN count SET NOT NULL")
        .execute(pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS ix_url_visit_actions_url_id ON url_visit_actions (url_id)")
        .execute(pool)
        .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS ix_url_ip_action_at ON url_visit_actions (url_id, ip_address, action_at)",
    )
    .execute(pool)
    .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS ix_visit_country ON url_visit_actions (country)")
        .execute(pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS ix_visit_city ON url_visit_actions (city)")
        .execute(pool)
        .await?;
    Ok(())
}

pub(crate) async fn load_url_cache(
    pool: &PgPool,
) -> Result<HashMap<String, CachedUrl>, sqlx::Error> {
    let rows = sqlx::query("SELECT id, short_code, long_url FROM short_urls")
        .fetch_all(pool)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            (
                row.get::<String, _>("short_code"),
                CachedUrl {
                    id: row.get("id"),
                    url: row.get("long_url"),
                },
            )
        })
        .collect())
}

pub(crate) async fn category_ids(pool: &PgPool, short_url_id: i32) -> Result<Vec<i32>, axum::http::StatusCode> {
    sqlx::query_scalar("SELECT category_id FROM short_url_categories WHERE short_url_id = $1")
        .bind(short_url_id)
        .fetch_all(pool)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) fn is_unique_violation(error: &sqlx::Error) -> bool {
    matches!(error, sqlx::Error::Database(database_error) if database_error.code().as_deref() == Some("23505"))
}