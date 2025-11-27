use axum::{async_trait, extract::FromRequestParts, http::request::Parts};

use crate::{application::auth_service::AuthClaims, shared::errors::ApiError};

#[derive(Clone)]
pub struct AuthContext(pub AuthClaims);

#[async_trait]
impl<S> FromRequestParts<S> for AuthContext
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthClaims>()
            .cloned()
            .map(AuthContext)
            .ok_or(ApiError::Unauthorized {
                message: "missing auth context".into(),
            })
    }
}
