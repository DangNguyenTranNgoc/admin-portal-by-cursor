use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};

use crate::{
    application::auth_service::{AuthClaims, TokenEncoder},
    shared::errors::DomainError,
};

pub struct JwtService {
    secret: String,
}

impl JwtService {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }
}

impl TokenEncoder for JwtService {
    fn encode(&self, claims: &AuthClaims) -> Result<String, DomainError> {
        encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| DomainError::Unexpected(e.to_string()))
    }

    fn decode(&self, token: &str) -> Result<AuthClaims, DomainError> {
        let data = decode::<AuthClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| DomainError::InvalidCredentials)?;

        Ok(data.claims)
    }
}
