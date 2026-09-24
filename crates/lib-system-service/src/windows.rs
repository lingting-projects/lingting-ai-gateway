//! Windows 任务计划程序实现：注册开机任务，任务直接启动目标可执行文件。
//!
//! 任务以 `SYSTEM` 身份运行，因此数据与日志目录由启动参数显式给出，不依赖用户环境变量。

use anyhow::Result;

use crate::command;
use crate::config::ServiceConfig;
use crate::manager::Platform;

/// 计划任务名与服务名一致。
pub(crate) struct WindowsService;

impl Platform for WindowsService {
    fn install(&self, config: &ServiceConfig) -> Result<()> {
        // /f 覆盖同名任务；/sc onstart 开机启动；/ru SYSTEM 与 /rl highest 以系统最高权限运行。
        command::run(
            "schtasks",
            &[
                "/create",
                "/tn",
                &config.name,
                "/tr",
                &config.command_line(),
                "/sc",
                "onstart",
                "/ru",
                "SYSTEM",
                "/rl",
                "highest",
                "/f",
            ],
        )
    }

    fn uninstall(&self, config: &ServiceConfig) -> Result<()> {
        // /f 跳过删除确认，避免命令等待输入。
        command::run("schtasks", &["/delete", "/tn", &config.name, "/f"])
    }

    fn start(&self, config: &ServiceConfig) -> Result<()> {
        command::run("schtasks", &["/run", "/tn", &config.name])
    }

    fn stop(&self, config: &ServiceConfig) -> Result<()> {
        command::run("schtasks", &["/end", "/tn", &config.name])
    }
}
