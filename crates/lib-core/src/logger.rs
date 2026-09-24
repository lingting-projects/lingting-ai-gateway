use framework_core::logging::{LoggingConfig, LoggingGuard};
use std::io;
use std::path::Path;
use std::sync::Arc;
use tracing::Metadata;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::{Context, Filter};

/// 初始化日志：全部级别输出到指定目录的文件。
pub fn init_logging(directory: &Path) -> io::Result<LoggingGuard> {
    let mut config = LoggingConfig::new();
    config.level = LevelFilter::DEBUG;
    config.directory = Some(directory.to_path_buf());
    config.push_filter(Arc::new(DefaultFilter));
    framework_core::logging::init(&config)
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultFilter;

impl<S> Filter<S> for DefaultFilter {
    fn enabled(&self, meta: &Metadata<'_>, _: &Context<'_, S>) -> bool {
        if meta.target().starts_with("h2")
            && matches!(*meta.level(), tracing::Level::DEBUG | tracing::Level::TRACE)
        {
            return false;
        }

        true
    }
}
