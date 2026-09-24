use std::io;
use std::path::Path;

use framework_core::logging::{LoggingConfig, LoggingGuard};
use tracing_subscriber::filter::LevelFilter;

/// 初始化日志：全部级别输出到指定目录的文件。
pub fn init_logging(directory: &Path) -> io::Result<LoggingGuard> {
    let mut config = LoggingConfig::new();
    config.level = LevelFilter::DEBUG;
    config.directory = Some(directory.to_path_buf());

    framework_core::logging::init(&config)
}
