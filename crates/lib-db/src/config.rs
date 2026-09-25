use std::net::{SocketAddr, SocketAddrV4};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use log::log;
use pglite_oxide::PgliteServer;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

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
    pub(crate) server: PgliteServer,
    pub(crate) pool: PgPool,
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
