use std::net::{SocketAddr, SocketAddrV4};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use log::log;
use pglite_oxide::PgliteServer;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

/// 连接池只有一个连接：pglite-oxide 的代理同一时刻只服务一个后端连接。
const MAX_CONNECTIONS: u32 = 1;
/// 单条语句执行超时，避免慢查询长期占用唯一连接。
const STATEMENT_TIMEOUT: &str = "30s";
/// 等待唯一连接释放的最长时间。
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(60);

/// 嵌入式数据库配置。
#[derive(Debug, Clone)]
pub struct DbConfig {
    /// 数据目录，为 `None` 时使用临时数据库（进程退出即丢弃）。
    pub path: Option<PathBuf>,
    /// 数据库用户名。
    pub username: String,
    /// 数据库名。
    pub database: String,
}

impl DbConfig {
    /// 使用指定数据目录的持久化数据库配置。
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: Some(path.into()),
            username: "postgres".to_string(),
            database: "postgres".to_string(),
        }
    }

    /// 使用临时数据库的配置，仅用于本地验证。
    pub fn temporary() -> Self {
        Self {
            path: None,
            username: "postgres".to_string(),
            database: "postgres".to_string(),
        }
    }
}

/// 已启动的数据库实例。
///
/// 持有 pglite 服务线程与连接池，必须存活到进程退出；`Drop` 时会关闭服务。
pub struct Db {
    server: PgliteServer,
    pool: PgPool,
}

impl Db {
    /// 连接池。
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// 数据库连接地址。
    pub fn url(&self) -> String {
        self.server.database_url()
    }

    /// 关闭连接池并停止 pglite 服务。
    pub async fn close(self) -> Result<()> {
        self.pool.close().await;
        self.server.shutdown().context("停止 pglite 服务失败")
    }
}

/// 启动 pglite 服务并建立连接池。
pub async fn init(config: &DbConfig) -> Result<Db> {
    if let Some(path) = config.path.as_deref() {
        std::fs::create_dir_all(path)
            .with_context(|| format!("创建数据目录失败：{}", path.display()))?;
    }

    #[cfg(not(debug_assertions))]
    let address = SocketAddr::from(([127, 0, 0, 1], 0));
    #[cfg(debug_assertions)]
    let address = SocketAddr::from(([127, 0, 0, 1], 26382));

    let mut builder = PgliteServer::builder()
        .username(config.username.clone())
        .database(config.database.clone())
        .tcp(address);
    builder = match config.path.clone() {
        Some(path) => builder.path(path),
        None => builder.temporary(),
    };
    let server = builder.start().context("启动 pglite 服务失败")?;

    let url = server.database_url();
    log::debug!("数据库连接地址: {url}");
    let pool = PgPoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .min_connections(MAX_CONNECTIONS)
        .acquire_timeout(ACQUIRE_TIMEOUT)
        .idle_timeout(None)
        .max_lifetime(None)
        .after_connect(|connection, _meta| {
            Box::pin(async move {
                sqlx::query(&format!("SET statement_timeout = '{STATEMENT_TIMEOUT}'"))
                    .execute(connection)
                    .await?;
                Ok(())
            })
        })
        .connect(&url)
        .await
        .context("连接 pglite 数据库失败")?;

    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .context("执行 PostgreSQL migration 失败")?;
    Ok(Db { server, pool })
}
