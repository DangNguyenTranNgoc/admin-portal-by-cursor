use std::time::Duration;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub postgres: PostgresConfig,
    pub clickhouse: ClickHouseConfig,
    pub kafka: KafkaConfig,
    pub auth: AuthConfig,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "local".to_string());
        let builder = config::Config::builder()
            .add_source(config::File::with_name("config/default").required(true))
            .add_source(
                config::File::with_name(&format!("config/{env}"))
                    .required(false)
                    .format(config::FileFormat::Toml),
            )
            .add_source(config::Environment::with_prefix("APP").separator("__"));

        Ok(builder.build()?.try_deserialize()?)
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub cors_allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PostgresConfig {
    pub uri: String,
    #[serde(default = "PostgresConfig::default_pool_size")]
    pub max_connections: u32,
}

impl PostgresConfig {
    fn default_pool_size() -> u32 {
        10
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClickHouseConfig {
    pub uri: String,
    pub database: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub brokers: String,
    pub group_id: String,
    pub topic: String,
    #[serde(default = "KafkaConfig::default_poll_interval_ms")]
    pub poll_interval_ms: u64,
}

impl KafkaConfig {
    fn default_poll_interval_ms() -> u64 {
        500
    }

    pub fn poll_interval(&self) -> Duration {
        Duration::from_millis(self.poll_interval_ms)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_audience: String,
    pub jwt_issuer: String,
    pub jwt_ttl_seconds: i64,
    #[serde(default = "AuthConfig::default_password_salt")]
    pub password_salt: String,
}

impl AuthConfig {
    fn default_password_salt() -> String {
        "change_me".to_string()
    }
}
