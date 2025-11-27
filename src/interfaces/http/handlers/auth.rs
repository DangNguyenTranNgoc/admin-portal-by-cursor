use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::{shared::errors::ApiError, state::SharedState};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub access_token: String,
}

pub async fn login(
    State(state): State<SharedState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let token = state
        .auth_service
        .login(&payload.email, &payload.password)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(LoginResponse {
        access_token: token.access_token,
    }))
}
