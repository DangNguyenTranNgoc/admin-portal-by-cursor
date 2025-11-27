use axum::{
    body::Body,
    extract::State,
    http::{Request, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};

use crate::{shared::errors::ApiError, state::SharedState};

pub async fn authenticate_request(
    State(state): State<SharedState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    let header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::Unauthorized {
            message: "missing authorization header".into(),
        })?;

    let token = header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::Unauthorized {
            message: "invalid authorization header".into(),
        })?;

    let claims = state
        .auth_service
        .decode(token)
        .map_err(|_| ApiError::Unauthorized {
            message: "invalid token".into(),
        })?;

    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}
