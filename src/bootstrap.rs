use std::sync::Arc;

use axum::Router;
use tokio::net::TcpListener;
use tracing::{Level, info};
use tracing_subscriber::EnvFilter;

use crate::{
    application::{
        auth_service::AuthService, data_catalog_service::DataCatalogService,
        permission_service::PermissionService, user_service::UserService,
    },
    config::AppConfig,
    infrastructure::{
        auth::{jwt::JwtService, password::PasswordService},
        clickhouse::{ClickHouseUserWarehouse, build_clickhouse_client},
        kafka::{UserEventProducer, spawn_consumer},
        postgres::{
            build_pg_pool,
            repositories::{PgDataCatalogRepository, PgPermissionRepository, PgUserRepository},
        },
    },
    interfaces::http::router::build_router,
    state::AppState,
};

pub async fn run() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let config = Arc::new(AppConfig::load()?);

    let pg_pool = build_pg_pool(&config.postgres).await?;
    let clickhouse_client = build_clickhouse_client(&config.clickhouse)?;
    let kafka_cfg = Arc::new(config.kafka.clone());
    let kafka_producer = UserEventProducer::new(&config.kafka)?;

    let user_repo: Arc<dyn crate::domain::user::UserRepository> =
        Arc::new(PgUserRepository::new(pg_pool.clone()));
    let permission_repo: Arc<dyn crate::domain::permission::PermissionRepository> =
        Arc::new(PgPermissionRepository::new(pg_pool.clone()));
    let catalog_repo: Arc<dyn crate::application::data_catalog_service::DataCatalogRepository> =
        Arc::new(PgDataCatalogRepository::new(pg_pool.clone()));
    let warehouse: Arc<dyn crate::application::user_service::UserWarehouse> =
        Arc::new(ClickHouseUserWarehouse::new(clickhouse_client.clone()));
    let password_manager: Arc<dyn crate::application::auth_service::PasswordManager> =
        Arc::new(PasswordService);
    let jwt_service = Arc::new(JwtService::new(config.auth.jwt_secret.clone()));

    let user_service = Arc::new(UserService::new(user_repo.clone(), warehouse));
    let permission_service = Arc::new(PermissionService::new(permission_repo.clone()));
    let auth_service = Arc::new(AuthService::new(
        user_repo,
        password_manager.clone(),
        jwt_service,
        config.auth.clone(),
    ));
    let catalog_service = Arc::new(DataCatalogService::new(catalog_repo));

    let shared_state = Arc::new(AppState::new(
        config.clone(),
        pg_pool.clone(),
        clickhouse_client.clone(),
        kafka_producer,
        user_service,
        auth_service,
        permission_service,
        catalog_service,
        password_manager,
    ));

    spawn_consumer(kafka_cfg, shared_state.clone()).await?;

    let router: Router = build_router(shared_state.clone());
    let addr = config.addr();
    let listener = TcpListener::bind(&addr).await?;
    info!("HTTP server listening on {addr}");

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn init_tracing() {
    if tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                EnvFilter::builder()
                    .with_default_directive(Level::INFO.into())
                    .from_env_lossy()
            }))
            .finish(),
    )
    .is_ok()
    {
        return;
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
