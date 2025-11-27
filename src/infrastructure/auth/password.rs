use argon2::{
    Argon2,
    password_hash::{
        PasswordHash, PasswordHasher as ArgonPasswordHasher,
        PasswordVerifier as ArgonPasswordVerifier, SaltString,
    },
};
use async_trait::async_trait;

use crate::application::auth_service::PasswordManager;

#[derive(Clone)]
pub struct PasswordService;

#[async_trait]
impl PasswordManager for PasswordService {
    async fn verify(&self, raw: &str, hashed: &str, salt: &str) -> anyhow::Result<bool> {
        let candidate = format!("{salt}{raw}");
        let parsed_hash = PasswordHash::new(hashed)?;
        let result = Argon2::default().verify_password(candidate.as_bytes(), &parsed_hash);
        Ok(result.is_ok())
    }

    async fn hash(&self, raw: &str, salt: &str) -> anyhow::Result<String> {
        let candidate = format!("{salt}{raw}");
        let salt = SaltString::from_b64(salt)?;
        let parsed_hash = Argon2::default().hash_password(candidate.as_bytes(), &salt)?;
        Ok(parsed_hash.to_string())
    }
}
