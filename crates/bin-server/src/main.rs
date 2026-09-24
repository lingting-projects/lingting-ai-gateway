mod arguments;
mod service;

use std::process::ExitCode;
use std::sync::Arc;

use anyhow::{Context, Result};
use framework_web_axum::axum_builder;
use lib_core::{Directorys, init_logging};
use lib_db::{DbConfig, DbContext};
use service_admin::service::KvConfigService;
use tracing::log;

use crate::arguments::{Arguments, Command};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            // 输出完整错误链，让用户直接看到根因，而不是只看到最外层描述。
            eprintln!("{error:#}");
            ExitCode::FAILURE
        }
    }
}

/// 入口：有子命令时执行子命令，没有子命令时按常规方式启动服务。
fn run() -> Result<()> {
    let Arguments {
        command,
        pglite,
        logs,
    } = Arguments::parse(std::env::args().skip(1))?;
    let directorys = Directorys::resolve(pglite, logs)?;

    match command {
        Some(Command::Install) => service::install(&directorys),
        Some(Command::Uninstall { stop }) => service::uninstall(&directorys, stop),
        None => serve(directorys),
    }
}

/// 常规启动：初始化日志与数据库，再启动 Axum 服务。
#[tokio::main]
async fn serve(directorys: Directorys) -> Result<()> {
    let logging_guard = init_logging(&directorys.logs)?;

    let db_config = DbConfig::new(directorys.pglite);
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
