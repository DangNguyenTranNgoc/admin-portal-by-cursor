use axum::{
    body::Body,
    extract::State,
    http::{Method, Request},
    middleware::Next,
    response::Response,
};

use crate::{domain::permission::PermissionMethod, shared::errors::ApiError, state::SharedState};

pub async fn authorize_request(
    State(state): State<SharedState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    let method = map_method(request.method());
    let path = request.uri().path().to_string();

    let claims = request
        .extensions()
        .get::<crate::application::auth_service::AuthClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Unauthorized {
            message: "missing auth context".into(),
        })?;

    state
        .permission_service
        .ensure_access(&claims.groups, &path, method)
        .await
        .map_err(ApiError::from)?;

    Ok(next.run(request).await)
}

fn map_method(method: &Method) -> PermissionMethod {
    match *method {
        Method::GET | Method::HEAD => PermissionMethod::Read,
        Method::POST | Method::PUT | Method::PATCH => PermissionMethod::Write,
        Method::DELETE => PermissionMethod::Delete,
        _ => PermissionMethod::Read,
    }
}
