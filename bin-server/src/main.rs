use anyhow::{Context, Result};
use framework_core::logging::LoggingConfig;
use framework_web_axum::axum_builder;
use lib_core::application_directory;
use lib_db::{DbConfig, DbContext, init_db};
use lib_web::{WebContext, WebRouter};
use std::sync::Arc;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> Result<()> {
    let directory = application_directory()?;
    let mut logging_config = LoggingConfig::new();
    logging_config.level = LevelFilter::DEBUG;
    logging_config.directory = Some(directory.logs);
    let logging_guard = lib_core::logging::init(&logging_config)?;

    let db_config = DbConfig {
        path: Some(directory.data.join("pgsql")),
        username: "pgsql".to_string(),
        database: "pgsql".to_string(),
    };
    let db = lib_db::init(&db_config).await?;
    let db_context = Arc::new(DbContext::new(db.pool().clone()));
    let server = axum_builder("127.0.0.1", 0).bind().await?;
    let result = server
        .run(Some(lib_web::web_route_wrapper(db_context)))
        .await;
    drop(db);
    drop(logging_guard);
    result.context("Axum 服务运行失败")
}
