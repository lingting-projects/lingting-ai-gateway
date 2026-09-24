//! Linux systemd 实现：写入 `/etc/systemd/system` 下的单元文件，再用 `systemctl` 控制。
//!
//! 属于系统级单元，注册与启停都需要 root 权限。

use std::path::PathBuf;

use anyhow::Result;

use crate::command;
use crate::config::ServiceConfig;
use crate::file;
use crate::manager::Platform;

/// systemd 系统级单元目录。
const SYSTEMD_DIRECTORY: &str = "/etc/systemd/system";

/// systemd 单元文件后缀。
const UNIT_SUFFIX: &str = ".service";

/// systemd 服务实现。
pub(crate) struct LinuxService;

impl Platform for LinuxService {
    fn install(&self, config: &ServiceConfig) -> Result<()> {
        file::write_file(&unit_path(config), &unit_content(config))?;
        command::run("systemctl", &["daemon-reload"])?;

        if config.autostart {
            command::run("systemctl", &["enable", &config.name])?;
        }

        Ok(())
    }

    fn uninstall(&self, config: &ServiceConfig) -> Result<()> {
        // 未启用时 disable 会报错，注销流程不应因此中断。
        let _ = command::run("systemctl", &["disable", &config.name]);
        file::remove_file(&unit_path(config))?;
        command::run("systemctl", &["daemon-reload"])
    }

    fn start(&self, config: &ServiceConfig) -> Result<()> {
        command::run("systemctl", &["start", &config.name])
    }

    fn stop(&self, config: &ServiceConfig) -> Result<()> {
        command::run("systemctl", &["stop", &config.name])
    }
}

/// 单元文件路径。
fn unit_path(config: &ServiceConfig) -> PathBuf {
    PathBuf::from(SYSTEMD_DIRECTORY).join(format!("{}{UNIT_SUFFIX}", config.name))
}

/// 单元文件内容；工作目录仅在配置给出时写入。
fn unit_content(config: &ServiceConfig) -> String {
    let working_directory = config
        .working_directory
        .as_ref()
        .map(|path| format!("WorkingDirectory={}\n", path.display()))
        .unwrap_or_default();

    format!(
        "[Unit]\n\
         Description={description}\n\
         After=network-online.target\n\
         Wants=network-online.target\n\
         \n\
         [Service]\n\
         Type=simple\n\
         ExecStart={command}\n\
         {working_directory}\
         Restart=on-failure\n\
         RestartSec=5\n\
         \n\
         [Install]\n\
         WantedBy=multi-user.target\n",
        description = config.description,
        command = config.command_line(),
    )
}
