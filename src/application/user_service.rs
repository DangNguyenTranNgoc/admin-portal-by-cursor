use std::sync::Arc;

use async_trait::async_trait;
use serde::Serialize;

use crate::{
    domain::user::{CreateUserCommand, UserId, UserRepository, UserWithGroups},
    shared::errors::DomainError,
};

pub struct UserService {
    repo: Arc<dyn UserRepository>,
    warehouse: Arc<dyn UserWarehouse>,
}

impl UserService {
    pub fn new(repo: Arc<dyn UserRepository>, warehouse: Arc<dyn UserWarehouse>) -> Self {
        Self { repo, warehouse }
    }

    pub async fn list_users(&self) -> Result<Vec<UserWithGroups>, DomainError> {
        self.repo.list().await
    }

    pub async fn get_user(&self, id: UserId) -> Result<UserWithGroups, DomainError> {
        self.repo
            .find_by_id(&id)
            .await?
            .ok_or(DomainError::UserNotFound)
    }

    pub async fn create_user(&self, cmd: CreateUserCommand) -> Result<UserWithGroups, DomainError> {
        self.repo.create(cmd).await
    }

    pub async fn query_user_data(
        &self,
        query: WarehouseQuery,
    ) -> Result<Vec<serde_json::Value>, DomainError> {
        self.warehouse.execute(query).await
    }
}

#[async_trait]
pub trait UserWarehouse: Send + Sync {
    async fn execute(&self, query: WarehouseQuery) -> Result<Vec<serde_json::Value>, DomainError>;
}

#[derive(Debug, Clone, Serialize)]
pub struct WarehouseQuery {
    pub statement: String,
    pub limit: Option<u64>,
}
