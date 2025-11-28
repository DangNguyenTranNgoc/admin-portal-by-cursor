use std::sync::Arc;

use async_trait::async_trait;
use serde::Serialize;
use tracing::{debug, info};

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
        debug!("Listing all users from service");
        let users = self.repo.list().await?;
        debug!("Retrieved {} users from repository", users.len());
        Ok(users)
    }

    pub async fn get_user(&self, id: UserId) -> Result<UserWithGroups, DomainError> {
        debug!("Fetching user with id: {} from service", id);
        let user = self
            .repo
            .find_by_id(&id)
            .await?
            .ok_or(DomainError::UserNotFound)?;
        debug!("Successfully retrieved user with id: {}", id);
        Ok(user)
    }

    pub async fn create_user(&self, cmd: CreateUserCommand) -> Result<UserWithGroups, DomainError> {
        debug!("Creating user in service with email: {}", cmd.email);
        let user = self.repo.create(cmd).await?;
        info!("User created successfully via service: {}", user.user.id);
        Ok(user)
    }

    pub async fn query_user_data(
        &self,
        query: WarehouseQuery,
    ) -> Result<Vec<serde_json::Value>, DomainError> {
        debug!("Executing warehouse query: {}", query.statement);
        let result = self.warehouse.execute(query).await?;
        debug!("Warehouse query returned {} rows", result.len());
        Ok(result)
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
