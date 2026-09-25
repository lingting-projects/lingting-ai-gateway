//! 系统服务注册、卸载与重启。
//!
//! 服务启动的是当前运行中的可执行文件绝对路径，并把当前数据与日志目录作为启动参数写入，
//! 保证服务方式启动与手动运行共用同一份数据。

use std::env;
use std::path::PathBuf;

use anyhow::{Context, Result};
use lib_core::{APP_ID, Directorys};
use lib_system_service::{ServiceConfig, ServiceManager, ServiceStatus, process};

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

/// 卸载系统服务；未注册时视为成功，`stop` 为真时先停止再卸载。
pub(crate) fn uninstall(directorys: &Directorys, stop: bool) -> Result<()> {
    let config = config(directorys)?;
    let manager = ServiceManager::new()?;

    if manager.status(&config)? == ServiceStatus::NotInstalled {
        println!("服务 {} 未注册", config.name);
        return Ok(());
    }

    if stop {
        manager.stop(&config)?;
    }
    manager.uninstall(&config)?;

    println!("服务 {} 已卸载", config.name);
    Ok(())
}

/// 重新注册：先停止并卸载，再注册并启动。
pub(crate) fn reinstall(directorys: &Directorys) -> Result<()> {
    uninstall(directorys, true)?;

    install(directorys)
}

/// 重启结果。
pub(crate) enum Restart {
    /// 服务已注册，已由系统服务管理器重启，命令到此结束。
    Service,
    /// 服务未注册，已终止原进程，需要由当前进程前台启动。
    Foreground,
}

/// 重启：已注册时由服务管理器重启，未注册时终止同一可执行文件的进程后前台启动。
pub(crate) fn restart(directorys: &Directorys) -> Result<Restart> {
    let config = config(directorys)?;
    let manager = ServiceManager::new()?;

    if manager.status(&config)? == ServiceStatus::Installed {
        manager.restart(&config)?;
        println!("服务 {} 已重启", config.name);

        return Ok(Restart::Service);
    }

    let killed = process::kill_same_executable(&config.program)?;
    println!(
        "服务 {} 未注册，已终止进程 {killed:?}，转为前台启动",
        config.name
    );

    Ok(Restart::Foreground)
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
