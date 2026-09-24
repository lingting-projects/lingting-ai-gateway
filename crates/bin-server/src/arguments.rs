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

/// 卸载时先停止服务的参数。
const STOP_ARGUMENT: &str = "-r";

/// 子命令。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Command {
    /// 注册系统服务并启动。
    Install,
    /// 卸载系统服务；`stop` 为真时先停止再卸载。
    Uninstall { stop: bool },
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
        let mut install = false;
        let mut uninstall = false;
        let mut stop = false;
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                INSTALL_COMMAND => install = true,
                UNINSTALL_COMMAND => uninstall = true,
                STOP_ARGUMENT => stop = true,
                PGLITE_ARGUMENT => arguments.pglite = Some(directory(&mut args, PGLITE_ARGUMENT)?),
                LOGS_ARGUMENT => arguments.logs = Some(directory(&mut args, LOGS_ARGUMENT)?),
                _ => bail!("不支持的参数：{arg}"),
            }
        }

        if install && uninstall {
            bail!("{INSTALL_COMMAND} 与 {UNINSTALL_COMMAND} 不能同时使用");
        }
        if stop && !uninstall {
            bail!("{STOP_ARGUMENT} 只能与 {UNINSTALL_COMMAND} 一起使用");
        }

        arguments.command = if install {
            Some(Command::Install)
        } else if uninstall {
            Some(Command::Uninstall { stop })
        } else {
            None
        };

        Ok(arguments)
    }
}

/// 读取目录参数的值。
fn directory(args: &mut impl Iterator<Item = String>, name: &str) -> Result<PathBuf> {
    let value = args.next().ok_or_else(|| anyhow!("{name} 缺少目录参数"))?;

    Ok(PathBuf::from(value))
}
