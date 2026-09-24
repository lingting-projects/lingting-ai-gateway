use std::path::PathBuf;

/// 系统服务注册配置；三个操作系统共用同一份配置。
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    /// 服务名，同时作为 Windows 计划任务名、systemd 单元名与 launchd 标签。
    pub name: String,
    /// 服务描述。
    pub description: String,
    /// 要启动的可执行文件绝对路径。
    pub program: PathBuf,
    /// 启动参数。
    pub args: Vec<String>,
    /// 工作目录；为空表示由系统决定。
    pub working_directory: Option<PathBuf>,
    /// 是否随系统启动。
    pub autostart: bool,
}

impl ServiceConfig {
    /// 完整的启动命令行：可执行文件绝对路径加启动参数，含空格的路径加引号。
    pub(crate) fn command_line(&self) -> String {
        let mut line = quote(&self.program.to_string_lossy());
        for arg in &self.args {
            line.push(' ');
            line.push_str(&quote(arg));
        }
        line
    }
}

/// 命令行参数加引号；参数本身含双引号时原样返回，避免破坏用户输入。
fn quote(value: &str) -> String {
    if value.contains('"') || !value.contains(' ') {
        return value.to_string();
    }

    format!("\"{value}\"")
}
