use anyhow::Result;
use lib_db::Database;
use lib_store::Store;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    let database = Database::open("data/postgres-v1").await?;
    let app = lib_api::router(Store::new(database.pool.clone()));
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    tracing::info!(address = %listener.local_addr()?, "gateway started");
    axum::serve(listener, app).await?;
    Ok(())
}
