use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::shared::errors::DomainError;

pub struct DataCatalogService {
    repo: Arc<dyn DataCatalogRepository>,
}

impl DataCatalogService {
    pub fn new(repo: Arc<dyn DataCatalogRepository>) -> Self {
        Self { repo }
    }

    pub async fn list_schemas(&self) -> Result<Vec<DatasetSchema>, DomainError> {
        self.repo.list_schemas().await
    }

    pub async fn build_query(&self, dataset_id: i64) -> Result<String, DomainError> {
        self.repo.resolve_query(dataset_id).await
    }
}

#[async_trait]
pub trait DataCatalogRepository: Send + Sync {
    async fn list_schemas(&self) -> Result<Vec<DatasetSchema>, DomainError>;
    async fn resolve_query(&self, dataset_id: i64) -> Result<String, DomainError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetSchema {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub base_query: String,
}
