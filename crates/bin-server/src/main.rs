mod arguments;
mod service;
mod ui;

use std::process::ExitCode;
use std::sync::Arc;

use crate::arguments::{Arguments, Command};
use anyhow::{Context, Result, anyhow};
use framework_web_axum::axum_builder;
use lib_core::{Directorys, init_logging};
use lib_db::{DbConfig, DbContext};
use lib_system_service::is_elevated;
use service_admin::service::KvConfigService;
use tracing::log;

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
    #[cfg(not(debug_assertions))]
    {
        let elevated = is_elevated()?;
        #[cfg(target_os = "windows")]
        let message = "必须使用管理员权限启动!";
        #[cfg(not(target_os = "windows"))]
        let message = "必须使用ROOT权限启动!";

        if !elevated {
            return Err(anyhow!(message));
        }
    }

    let Arguments {
        command,
        pglite,
        logs,
    } = Arguments::parse(std::env::args().skip(1))?;
    let directorys = Directorys::resolve(pglite, logs)?;

    match command {
        Some(Command::Install) => service::install(&directorys),
        Some(Command::Uninstall { stop }) => service::uninstall(&directorys, stop),
        Some(Command::Reinstall) => service::reinstall(&directorys),
        Some(Command::Restart) => match service::restart(&directorys)? {
            service::Restart::Service => Ok(()),
            service::Restart::Foreground => serve(directorys),
        },
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

    let mut bind = KvConfigService::from(db.pool().clone())
        .server_bind()
        .await?;

    // 开发模式下固定占用端口, 避免本机安装时开发和正式版本抢端口
    #[cfg(debug_assertions)]
    {
        bind.port = 26384;
    }

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
