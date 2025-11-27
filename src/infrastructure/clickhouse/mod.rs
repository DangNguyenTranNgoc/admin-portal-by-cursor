use async_trait::async_trait;
use clickhouse::{Client, Row};
use serde::Deserialize;

use crate::{
    application::user_service::{UserWarehouse, WarehouseQuery},
    config::ClickHouseConfig,
    shared::errors::DomainError,
};

pub fn build_clickhouse_client(cfg: &ClickHouseConfig) -> anyhow::Result<Client> {
    let client = Client::default()
        .with_url(&cfg.uri)
        .with_database(&cfg.database);
    Ok(client)
}

pub struct ClickHouseUserWarehouse {
    client: Client,
}

impl ClickHouseUserWarehouse {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl UserWarehouse for ClickHouseUserWarehouse {
    async fn execute(&self, query: WarehouseQuery) -> Result<Vec<serde_json::Value>, DomainError> {
        let mut statement = query.statement;
        if let Some(limit) = query.limit {
            statement = format!("{statement} LIMIT {limit}");
        }

        let wrapped = format!("SELECT toJSONString(t) AS row_json FROM ({statement}) AS t");

        let rows = self
            .client
            .query(&wrapped)
            .fetch_all::<JsonRow>()
            .await
            .map_err(|e| DomainError::Unexpected(e.to_string()))?;

        rows.into_iter()
            .map(|row| {
                serde_json::from_str(&row.row_json)
                    .map_err(|e| DomainError::Unexpected(e.to_string()))
            })
            .collect()
    }
}

#[derive(Row, Deserialize)]
struct JsonRow {
    row_json: String,
}
