use axum::{
    Router, middleware,
    routing::{get, post},
};
use tower_http::trace::TraceLayer;

use crate::state::SharedState;

use super::{
    handlers::{auth, catalog, users},
    middleware::{auth::authenticate_request, permission::authorize_request},
};

pub fn build_router(state: SharedState) -> Router {
    let public = Router::new().route("/v1/auth/login", post(auth::login));

    let protected = Router::new()
        .nest(
            "/v1/users",
            Router::new()
                .route("/", get(users::list_users).post(users::create_user))
                .route("/:id", get(users::get_user)),
        )
        .nest(
            "/v1/catalog",
            Router::new()
                .route("/schemas", get(catalog::list_schemas))
                .route("/query", post(catalog::query_dataset)),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            authorize_request,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            authenticate_request,
        ));

    public
        .merge(protected)
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}
