use axum::Router;
use chrono::Local;
use std::sync::Arc;
use std::{fmt::Result, io::stdout};
use tokio::net::TcpListener;
use tracing::{debug, info};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

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
    // Initialize tracing subscriber with file logging
    let _guard = init_logging("logs", "app", "DEBUG");
    info!("Logging initialized successfully");

    info!("Loading application configuration");
    let config = Arc::new(AppConfig::load()?);
    debug!("Configuration loaded successfully");

    info!("Initializing PostgreSQL connection pool");
    let pg_pool = build_pg_pool(&config.postgres).await?;
    debug!("PostgreSQL connection pool established");

    info!("Initializing ClickHouse client");
    let clickhouse_client = build_clickhouse_client(&config.clickhouse)?;
    debug!("ClickHouse client initialized");

    info!("Initializing Kafka producer");
    let kafka_cfg = Arc::new(config.kafka.clone());
    let kafka_producer = UserEventProducer::new(&config.kafka)?;
    debug!("Kafka producer initialized");

    debug!("Creating repository instances");
    let user_repo: Arc<dyn crate::domain::user::UserRepository> =
        Arc::new(PgUserRepository::new(pg_pool.clone()));
    let permission_repo: Arc<dyn crate::domain::permission::PermissionRepository> =
        Arc::new(PgPermissionRepository::new(pg_pool.clone()));
    let catalog_repo: Arc<dyn crate::application::data_catalog_service::DataCatalogRepository> =
        Arc::new(PgDataCatalogRepository::new(pg_pool.clone()));
    debug!("Repository instances created");

    debug!("Creating application services");
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
    info!("All application services created successfully");

    debug!("Creating shared application state");
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
    debug!("Shared application state created");

    info!("Starting Kafka consumer");
    spawn_consumer(kafka_cfg, shared_state.clone()).await?;
    info!("Kafka consumer started");

    info!("Building HTTP router");
    let router: Router = build_router(shared_state.clone());
    let addr = config.addr();
    debug!("HTTP router built successfully");

    info!("Starting HTTP server on {}", addr);
    let listener = TcpListener::bind(&addr).await?;
    info!("HTTP server listening on {}", addr);

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Application shutdown complete");

    Ok(())
}

struct ShortTime;

impl FormatTime for ShortTime {
    fn format_time(&self, w: &mut Writer<'_>) -> Result {
        let now = Local::now();
        // Format: 2025-06-21T14:54:39
        write!(w, "{}", now.format("%Y-%m-%dT%H:%M:%S"))
    }
}

fn init_logging(log_dir: &str, file_name_prefix: &str, log_level: &str) -> WorkerGuard {
    // Rolling file appender (daily rotation)
    let file_appender = rolling::daily(log_dir, file_name_prefix);
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    // Console writer (stdout)
    let stdout_writer = stdout;

    // File layer (no color)
    let file_layer = fmt::layer()
        .with_timer(ShortTime) // Custom time format
        .with_file(true) // Optional: include file name in logs
        .with_line_number(true) // Optional: include line number in logs
        .with_writer(file_writer)
        .with_ansi(false)
        .with_target(false) // Optional: hide target/module path
        .with_level(true);

    // Console layer (with color)
    let console_layer = fmt::layer()
        .with_timer(ShortTime) // Custom time format
        .with_file(true) // Optional: include file name in logs
        .with_line_number(true) // Optional: include line number in logs
        .with_writer(stdout_writer)
        .with_ansi(true)
        .with_target(false)
        .with_level(true);

    // Optional: use env var like RUST_LOG=info
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    // Combine all layers
    tracing_subscriber::registry()
        .with(filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    guard // keep this alive to flush logs
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
