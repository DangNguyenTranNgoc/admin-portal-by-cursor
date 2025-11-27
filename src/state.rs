use std::sync::Arc;

use clickhouse::Client as ClickHouseClient;
use sqlx::PgPool;

use crate::{
    application::{
        auth_service::{AuthService, PasswordManager},
        data_catalog_service::DataCatalogService,
        permission_service::PermissionService,
        user_service::UserService,
    },
    config::AppConfig,
    infrastructure::kafka::UserEventProducer,
};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub pg_pool: PgPool,
    pub clickhouse: ClickHouseClient,
    pub kafka_producer: UserEventProducer,
    pub user_service: Arc<UserService>,
    pub auth_service: Arc<AuthService>,
    pub permission_service: Arc<PermissionService>,
    pub catalog_service: Arc<DataCatalogService>,
    pub password_manager: Arc<dyn PasswordManager>,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        config: Arc<AppConfig>,
        pg_pool: PgPool,
        clickhouse: ClickHouseClient,
        kafka_producer: UserEventProducer,
        user_service: Arc<UserService>,
        auth_service: Arc<AuthService>,
        permission_service: Arc<PermissionService>,
        catalog_service: Arc<DataCatalogService>,
        password_manager: Arc<dyn PasswordManager>,
    ) -> Self {
        Self {
            config,
            pg_pool,
            clickhouse,
            kafka_producer,
            user_service,
            auth_service,
            permission_service,
            catalog_service,
            password_manager,
        }
    }
}

pub type SharedState = Arc<AppState>;
