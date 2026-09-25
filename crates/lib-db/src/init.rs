use crate::{Db, DbConfig};
use anyhow::Context;
use pglite_oxide::PgliteServer;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::time::Duration;

/// 连接池只有一个连接：pglite-oxide 的代理同一时刻只服务一个后端连接。
pub(crate) const MAX_CONNECTIONS: u32 = 1;
/// 单条语句执行超时，避免慢查询长期占用唯一连接。
const STATEMENT_TIMEOUT: &str = "30s";
/// 等待唯一连接释放的最长时间。
pub(crate) const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(60);

/// 启动 pglite 服务并建立连接池。
pub async fn init(config: &DbConfig) -> anyhow::Result<Db> {
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

    #[cfg(debug_assertions)]
    log_last_migrations(&pool, 3).await?;

    Ok(Db { server, pool })
}

async fn log_last_migrations(pool: &PgPool, count: i64) -> anyhow::Result<()> {
    #[derive(sqlx::FromRow)]
    struct Migration {
        version: i64,
        description: String,
        checksum: Vec<u8>,
    }

    let migrations = sqlx::query_as::<_, Migration>(
        r#"
        SELECT version, description, checksum
        FROM "_sqlx_migrations"
        WHERE success = true
        ORDER BY version DESC
        LIMIT $1
        "#,
    )
    .bind(count)
    .fetch_all(pool)
    .await
    .context("查询最近 migration 失败")?;

    for migration in migrations.iter().rev() {
        log::info!(
            "migration: id={}, name={:?}, sum={}",
            migration.version,
            migration.description,
            checksum_to_hex(&migration.checksum),
        );
    }

    Ok(())
}

fn checksum_to_hex(checksum: &[u8]) -> String {
    checksum.iter().map(|b| format!("{b:02x}")).collect()
}
