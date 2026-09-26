//! 启动参数解析。

use std::path::PathBuf;

use anyhow::{Result, anyhow, bail};

/// pglite 数据目录参数名。
pub(crate) const PGLITE_ARGUMENT: &str = "-dir_pglite";

/// 日志目录参数名。
pub(crate) const LOGS_ARGUMENT: &str = "-dir_logs";

/// 注册服务子命令。
const INSTALL_COMMAND: &str = "install";

/// 卸载服务子命令。
const UNINSTALL_COMMAND: &str = "uninstall";

/// 重新注册服务子命令。
const REINSTALL_COMMAND: &str = "reinstall";

/// 重启服务子命令。
const RESTART_COMMAND: &str = "restart";

/// 停止服务子命令。
const STOP_COMMAND: &str = "stop";

/// 子命令。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Command {
    /// 注册系统服务并启动。
    Install,
    /// 卸载系统服务；已注册时先停止再卸载。
    Uninstall,
    /// 重新注册：先停止并卸载，再注册并启动。
    Reinstall,
    /// 重启：已注册时由服务管理器重启，未注册时终止原进程后前台启动。
    Restart,
    /// 停止系统服务。
    Stop,
}

/// 解析阶段的子命令名，用于识别与互斥校验。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandName {
    Install,
    Uninstall,
    Reinstall,
    Restart,
    Stop,
}

impl CommandName {
    /// 从参数识别子命令。
    fn parse(arg: &str) -> Option<Self> {
        match arg {
            INSTALL_COMMAND => Some(Self::Install),
            UNINSTALL_COMMAND => Some(Self::Uninstall),
            REINSTALL_COMMAND => Some(Self::Reinstall),
            RESTART_COMMAND => Some(Self::Restart),
            STOP_COMMAND => Some(Self::Stop),
            _ => None,
        }
    }

    /// 子命令名，用于报错提示。
    fn as_str(self) -> &'static str {
        match self {
            Self::Install => INSTALL_COMMAND,
            Self::Uninstall => UNINSTALL_COMMAND,
            Self::Reinstall => REINSTALL_COMMAND,
            Self::Restart => RESTART_COMMAND,
            Self::Stop => STOP_COMMAND,
        }
    }
}

/// 启动参数。
#[derive(Debug, Default)]
pub(crate) struct Arguments {
    /// 子命令；为空表示按常规方式启动服务。
    pub command: Option<Command>,
    /// pglite 数据目录；为空表示按应用目录生成。
    pub pglite: Option<PathBuf>,
    /// 日志目录；为空表示按应用目录生成。
    pub logs: Option<PathBuf>,
}

impl Arguments {
    /// 解析启动参数；不支持的参数直接报错，避免用户以为已经生效。
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self> {
        let mut arguments = Self::default();
        let mut name = None;
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                PGLITE_ARGUMENT => arguments.pglite = Some(directory(&mut args, PGLITE_ARGUMENT)?),
                LOGS_ARGUMENT => arguments.logs = Some(directory(&mut args, LOGS_ARGUMENT)?),
                _ => name = select(name, &arg)?,
            }
        }

        arguments.command = match name {
            None => None,
            Some(CommandName::Install) => Some(Command::Install),
            Some(CommandName::Uninstall) => Some(Command::Uninstall),
            Some(CommandName::Reinstall) => Some(Command::Reinstall),
            Some(CommandName::Restart) => Some(Command::Restart),
            Some(CommandName::Stop) => Some(Command::Stop),
        };

        Ok(arguments)
    }
}

/// 记录子命令；重复指定或与已有的冲突时报错。
fn select(current: Option<CommandName>, arg: &str) -> Result<Option<CommandName>> {
    let Some(parsed) = CommandName::parse(arg) else {
        bail!("不支持的参数：{arg}");
    };

    if let Some(existing) = current {
        bail!("{} 与 {} 不能同时使用", existing.as_str(), parsed.as_str());
    }

    Ok(Some(parsed))
}

/// 读取目录参数的值。
fn directory(args: &mut impl Iterator<Item = String>, name: &str) -> Result<PathBuf> {
    let value = args.next().ok_or_else(|| anyhow!("{name} 缺少目录参数"))?;

    Ok(PathBuf::from(value))
}
