use crate::{
    application::auth_service::{AuthClaims, TokenEncoder},
    shared::errors::DomainError,
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use tracing::{debug, error, info};

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
        debug!(
            "Encoding JWT token for user: {} with issuer: {} and audience: {}",
            claims.email, claims.iss, claims.aud
        );
        encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| {
            error!("Failed to encode JWT token: {}", e);
            DomainError::Unexpected(e.to_string())
        })
        .map(|token| {
            info!("JWT token encoded successfully for user: {}", claims.email);
            token
        })
    }

    fn decode(&self, token: &str, auth_claims: AuthClaims) -> Result<AuthClaims, DomainError> {
        debug!("Decoding JWT token");
        let mut validation = Validation::default();
        validation.set_issuer(&[auth_claims.iss.as_str()]);
        validation.set_audience(&[auth_claims.aud.as_str()]);

        let data = decode::<AuthClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map_err(|e| {
            error!("Error occurred while decoding JWT: {:?}", e);
            DomainError::InvalidCredentials
        })?;

        info!(
            "User {} authenticated with valid JWT token",
            data.claims.email
        );
        Ok(data.claims)
    }
}
