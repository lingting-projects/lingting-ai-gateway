use anyhow::{Context, Result};
use framework_core::logging::LoggingConfig;
use framework_web_axum::axum_builder;
use lib_core::application_directory;
use lib_db::{DbConfig, DbContext};
use service_admin::service::KvConfigService;
use std::sync::Arc;
use tracing::log;
use tracing_subscriber::filter::LevelFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let directory = application_directory()?;
    let mut logging_config = LoggingConfig::new();
    logging_config.level = LevelFilter::DEBUG;
    logging_config.directory = Some(directory.logs);
    let logging_guard = lib_core::logging::init(&logging_config)?;

    let db_config = DbConfig::new(directory.data.join("pgsql"));
    let db = lib_db::init(&db_config).await?;
    let db_context = Arc::new(DbContext::new(db.pool().clone()));

    let bind = KvConfigService::from(db.pool().clone())
        .server_bind()
        .await?;

    let server = axum_builder(bind.address.as_str(), bind.port)
        .bind()
        .await?;
    log::info!("当前服务绑定: {}:{}", bind.address, server.context().port);
    let result = server
        .run(Some(lib_web::web_route_wrapper(db_context)))
        .await;
    drop(db);
    drop(logging_guard);
    result.context("Axum 服务运行失败")
}
