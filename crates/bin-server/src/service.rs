//! 系统服务注册与卸载。
//!
//! 服务启动的是当前运行中的可执行文件绝对路径，并把当前数据与日志目录作为启动参数写入，
//! 保证服务方式启动与手动运行共用同一份数据。

use std::env;
use std::path::PathBuf;

use anyhow::{Context, Result};
use lib_core::{APP_ID, Directorys};
use lib_system_service::{ServiceConfig, ServiceManager};

use crate::arguments::{LOGS_ARGUMENT, PGLITE_ARGUMENT};

/// 服务描述。
const DESCRIPTION: &str = "个人使用的轻量 AI 路由网关";

/// 注册系统服务并启动。
pub(crate) fn install(directorys: &Directorys) -> Result<()> {
    let config = config(directorys)?;
    let manager = ServiceManager::new()?;

    manager.install(&config)?;
    manager.start(&config)?;

    println!("服务 {} 已注册并启动", config.name);
    Ok(())
}

/// 卸载系统服务；`stop` 为真时先停止再卸载。
pub(crate) fn uninstall(directorys: &Directorys, stop: bool) -> Result<()> {
    let config = config(directorys)?;
    let manager = ServiceManager::new()?;

    if stop {
        manager.stop(&config)?;
    }
    manager.uninstall(&config)?;

    println!("服务 {} 已卸载", config.name);
    Ok(())
}

/// 服务配置：以当前可执行文件为启动目标，并带上当前数据与日志目录。
fn config(directorys: &Directorys) -> Result<ServiceConfig> {
    let program = env::current_exe().context("获取当前可执行文件路径失败")?;

    Ok(ServiceConfig {
        name: APP_ID.to_string(),
        description: DESCRIPTION.to_string(),
        args: vec![
            PGLITE_ARGUMENT.to_string(),
            directorys.pglite.display().to_string(),
            LOGS_ARGUMENT.to_string(),
            directorys.logs.display().to_string(),
        ],
        working_directory: program.parent().map(PathBuf::from),
        program,
        autostart: true,
    })
}
