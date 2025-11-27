use admin_backend::bootstrap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap::run().await
}
