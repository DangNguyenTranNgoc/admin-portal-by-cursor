use argon2::password_hash::SaltString;
use axum::{
    Json,
    extract::{Path, State},
};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};

use crate::{
    domain::user::{CreateUserCommand, UserId, UserWithGroups},
    shared::errors::ApiError,
    state::SharedState,
};

#[derive(Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub status: String,
    pub groups: Vec<GroupResponse>,
}

#[derive(Serialize)]
pub struct GroupResponse {
    pub id: i64,
    pub name: String,
}

impl From<UserWithGroups> for UserResponse {
    fn from(value: UserWithGroups) -> Self {
        Self {
            id: value.user.id.0,
            email: value.user.email,
            first_name: value.user.first_name,
            last_name: value.user.last_name,
            status: format!("{:?}", value.user.status),
            groups: value
                .groups
                .into_iter()
                .map(|g| GroupResponse {
                    id: g.id,
                    name: g.name,
                })
                .collect(),
        }
    }
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub password: String,
    #[serde(default)]
    pub groups: Vec<i64>,
}

pub async fn list_users(
    State(state): State<SharedState>,
) -> Result<Json<Vec<UserResponse>>, ApiError> {
    let users = state
        .user_service
        .list_users()
        .await
        .map_err(ApiError::from)?;
    Ok(Json(users.into_iter().map(UserResponse::from).collect()))
}

pub async fn get_user(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> Result<Json<UserResponse>, ApiError> {
    let user = state
        .user_service
        .get_user(UserId(id))
        .await
        .map_err(ApiError::from)?;

    Ok(Json(UserResponse::from(user)))
}

pub async fn create_user(
    State(state): State<SharedState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    let salt_str = salt.to_string();
    let password_hash = state
        .password_manager
        .hash(&payload.password, &salt_str)
        .await
        .map_err(|_| ApiError::Internal)?;

    let command = CreateUserCommand {
        email: payload.email,
        first_name: payload.first_name,
        last_name: payload.last_name,
        password_hash,
        salt: salt_str,
        groups: payload.groups,
    };

    let user = state
        .user_service
        .create_user(command)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(UserResponse::from(user)))
}
