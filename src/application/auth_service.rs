use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    config::AuthConfig,
    domain::{permission::Permission, user::UserRepository},
    shared::errors::DomainError,
};

pub struct AuthService {
    user_repo: Arc<dyn UserRepository>,
    password: Arc<dyn PasswordManager>,
    token: Arc<dyn TokenEncoder>,
    auth_config: AuthConfig,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        password: Arc<dyn PasswordManager>,
        token: Arc<dyn TokenEncoder>,
        auth_config: AuthConfig,
    ) -> Self {
        Self {
            user_repo,
            password,
            token,
            auth_config,
        }
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<AuthToken, DomainError> {
        let Some(user) = self.user_repo.find_by_email(email).await? else {
            return Err(DomainError::InvalidCredentials);
        };

        let creds = user
            .credentials
            .ok_or_else(|| DomainError::Unexpected("credentials not loaded".to_string()))?;

        let valid = self
            .password
            .verify(password, &creds.password_hash, &creds.salt)
            .await
            .map_err(|e| DomainError::Unexpected(e.to_string()))?;

        if !valid {
            return Err(DomainError::InvalidCredentials);
        }

        let claims = AuthClaims {
            sub: user.user.id.0,
            email: user.user.email.clone(),
            groups: user.groups.into_iter().map(|g| g.id).collect(),
            exp: (Utc::now() + Duration::seconds(self.auth_config.jwt_ttl_seconds)).timestamp(),
            iss: self.auth_config.jwt_issuer.clone(),
            aud: self.auth_config.jwt_audience.clone(),
            permissions: vec![],
        };

        let token = self.token.encode(&claims)?;

        Ok(AuthToken {
            access_token: token,
        })
    }

    pub fn decode(&self, token: &str) -> Result<AuthClaims, DomainError> {
        self.token.decode(token)
    }
}

pub struct AuthToken {
    pub access_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthClaims {
    pub sub: i64,
    pub email: String,
    pub groups: Vec<i64>,
    pub exp: i64,
    pub iss: String,
    pub aud: String,
    pub permissions: Vec<Permission>,
}

#[async_trait]
pub trait PasswordManager: Send + Sync {
    async fn verify(&self, raw: &str, hashed: &str, salt: &str) -> anyhow::Result<bool>;
    async fn hash(&self, raw: &str, salt: &str) -> anyhow::Result<String>;
}

pub trait TokenEncoder: Send + Sync {
    fn encode(&self, claims: &AuthClaims) -> Result<String, DomainError>;
    fn decode(&self, token: &str) -> Result<AuthClaims, DomainError>;
}
