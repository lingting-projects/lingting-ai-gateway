use anyhow::{Context, Result};
use pglite_oxide::PgliteServer;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::net::SocketAddr;

pub struct Database {
    _server: PgliteServer,
    pub pool: PgPool,
}
impl Database {
    pub async fn open(path: impl Into<std::path::PathBuf>) -> Result<Self> {
        let server = pglite_oxide::PgliteServerBuilder::new()
            .path(path)
            .tcp("127.0.0.1:0".parse::<SocketAddr>()?)
            .start()
            .context("start pglite")?;
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(&server.connection_uri())
            .await
            .context("connect sqlx pool")?;
        sqlx::migrate!("../migrations")
            .run(&pool)
            .await
            .context("run migrations")?;
        Ok(Self {
            _server: server,
            pool,
        })
    }
}
