//! macOS launchd 实现：写入 `/Library/LaunchDaemons` 下的 plist，再用 `launchctl` 控制。
//!
//! 属于系统级守护进程，注册与启停都需要 root 权限。

use std::path::PathBuf;

use anyhow::Result;

use crate::command;
use crate::config::ServiceConfig;
use crate::file;
use crate::manager::Platform;

/// launchd 系统级守护进程目录。
const LAUNCHD_DIRECTORY: &str = "/Library/LaunchDaemons";

/// launchd 配置文件后缀。
const PLIST_SUFFIX: &str = ".plist";

/// launchd 服务实现。
pub(crate) struct MacosService;

impl Platform for MacosService {
    fn install(&self, config: &ServiceConfig) -> Result<()> {
        let plist = plist_path(config);
        let plist_text = plist.to_string_lossy().to_string();

        file::write_file(&plist, &plist_content(config))?;
        // -w 覆盖已有的 disabled 标记，保证随系统启动。
        command::run("launchctl", &["load", "-w", &plist_text])
    }

    fn uninstall(&self, config: &ServiceConfig) -> Result<()> {
        // 未加载时 remove 会报错，注销流程不应因此中断。
        let _ = command::run("launchctl", &["remove", &config.name]);
        file::remove_file(&plist_path(config))
    }

    fn start(&self, config: &ServiceConfig) -> Result<()> {
        command::run("launchctl", &["start", &config.name])
    }

    fn stop(&self, config: &ServiceConfig) -> Result<()> {
        command::run("launchctl", &["stop", &config.name])
    }
}

/// plist 文件路径。
fn plist_path(config: &ServiceConfig) -> PathBuf {
    PathBuf::from(LAUNCHD_DIRECTORY).join(format!("{}{PLIST_SUFFIX}", config.name))
}

/// plist 文件内容；工作目录仅在配置给出时写入。
fn plist_content(config: &ServiceConfig) -> String {
    let working_directory = config
        .working_directory
        .as_ref()
        .map(|path| {
            format!(
                "\t<key>WorkingDirectory</key>\n\t<string>{}</string>\n",
                escape(&path.to_string_lossy())
            )
        })
        .unwrap_or_default();

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
         \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\">\n\
         <dict>\n\
         \t<key>Label</key>\n\
         \t<string>{label}</string>\n\
         \t<key>ProgramArguments</key>\n\
         \t<array>\n\
         {arguments}\
         \t</array>\n\
         {working_directory}\
         \t<key>RunAtLoad</key>\n\
         \t<true/>\n\
         </dict>\n\
         </plist>\n",
        label = escape(&config.name),
        arguments = arguments(config),
    )
}

/// `ProgramArguments` 数组内容：可执行文件绝对路径加启动参数。
fn arguments(config: &ServiceConfig) -> String {
    let mut arguments = format!(
        "\t\t<string>{}</string>\n",
        escape(&config.program.to_string_lossy())
    );
    for arg in &config.args {
        arguments.push_str(&format!("\t\t<string>{}</string>\n", escape(arg)));
    }
    arguments
}

/// XML 文本转义，避免路径中的 `&`、`<` 等字符破坏 plist 结构。
fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
