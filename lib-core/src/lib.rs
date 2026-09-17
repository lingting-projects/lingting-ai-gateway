//! 项目基础能力：日志初始化与程序文件夹。

pub use framework_core::logging;
pub use framework_core::{ApplicationDirectory, Money, current_millis, next_id};

/// 应用标识，用于程序文件夹与日志目录命名。
pub const APP_ID: &str = "lingting-ai-gateway";

/// 当前应用的用户级程序文件夹。
pub fn application_directory() -> anyhow::Result<ApplicationDirectory> {
    ApplicationDirectory::normal(APP_ID, "")
}
