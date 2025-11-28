use std::fmt::Display;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::errors::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserId(pub i32);

impl Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub status: UserStatus,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn is_active(&self) -> bool {
        self.status == UserStatus::Active
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UserStatus {
    Active,
    Suspended,
    Disabled,
}

impl TryFrom<String> for UserStatus {
    type Error = DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "active" => Ok(Self::Active),
            "suspended" => Ok(Self::Suspended),
            "disabled" => Ok(Self::Disabled),
            other => Err(DomainError::InvalidStatus(other.to_string())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserWithGroups {
    pub user: User,
    pub groups: Vec<UserGroup>,
    pub credentials: Option<UserCredentials>,
}

#[derive(Debug, Clone)]
pub struct UserGroup {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct UserCredentials {
    pub password_hash: String,
    pub salt: String,
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: &UserId) -> Result<Option<UserWithGroups>, DomainError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<UserWithGroups>, DomainError>;
    async fn list(&self) -> Result<Vec<UserWithGroups>, DomainError>;
    async fn create(&self, cmd: CreateUserCommand) -> Result<UserWithGroups, DomainError>;
}

#[derive(Debug, Clone)]
pub struct CreateUserCommand {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub password_hash: String,
    pub salt: String,
    pub groups: Vec<i32>,
}
