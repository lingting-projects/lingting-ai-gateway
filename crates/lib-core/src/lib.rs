//! 项目基础能力：日志初始化、程序文件夹与通用工具函数。

mod directory;
mod logger;

use sha1::Digest;

pub use directory::Directorys;
pub use framework_core::logging;
pub use framework_core::{ApplicationDirectory, Money, current_millis, next_id};
pub use logger::init_logging;

/// 应用标识，用于程序文件夹与日志目录命名。
pub const APP_ID: &str = "lingting-ai-gateway";

/// 计算字符串的 SHA1 十六进制摘要；API Key 与管理令牌的摘要统一走本函数。
pub fn hash(value: &str) -> String {
    format!("{:x}", sha1::Sha1::new().chain_update(value).finalize())
}

/// 判断字符串是否为真值：trim 后为 t、true、y、yes、ok、1 之一才算真。
pub fn string_is_true(value: &str) -> bool {
    matches!(value.trim(), "t" | "true" | "y" | "yes" | "ok" | "1")
}
