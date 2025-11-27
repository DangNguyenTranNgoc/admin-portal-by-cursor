use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::config::PostgresConfig;

pub mod repositories;

pub async fn build_pg_pool(cfg: &PostgresConfig) -> anyhow::Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(cfg.max_connections)
        .connect(&cfg.uri)
        .await
        .map_err(Into::into)
}
