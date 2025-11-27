use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::{
    application::{data_catalog_service::DatasetSchema, user_service::WarehouseQuery},
    shared::errors::ApiError,
    state::SharedState,
};

#[derive(Deserialize)]
pub struct QueryRequest {
    pub dataset_id: i64,
    pub limit: Option<u64>,
}

#[derive(Serialize)]
pub struct QueryResponse {
    pub rows: Vec<serde_json::Value>,
}

pub async fn list_schemas(
    State(state): State<SharedState>,
) -> Result<Json<Vec<DatasetSchema>>, ApiError> {
    let schemas = state
        .catalog_service
        .list_schemas()
        .await
        .map_err(ApiError::from)?;
    Ok(Json(schemas))
}

pub async fn query_dataset(
    State(state): State<SharedState>,
    Json(payload): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, ApiError> {
    let query = state
        .catalog_service
        .build_query(payload.dataset_id)
        .await
        .map_err(ApiError::from)?;

    let rows = state
        .user_service
        .query_user_data(WarehouseQuery {
            statement: query,
            limit: payload.limit,
        })
        .await
        .map_err(ApiError::from)?;

    Ok(Json(QueryResponse { rows }))
}
