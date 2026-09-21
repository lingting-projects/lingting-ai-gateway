//! 应用级全局上下文：从 kv_config 装载的调试模式与匿名访问开关。

use std::future::Future;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use types_admin::{KvConfig, KvConfigKey};

/// 应用全局上下文，构造完成后只读。
#[derive(Debug, Clone, Default)]
pub struct AppContext {
    debug_mode: bool,
    allow_anonymous: bool,
}

impl AppContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// 是否开启调试模式。
    pub fn debug_mode(&self) -> bool {
        self.debug_mode
    }

    /// 是否允许匿名访问。
    pub fn allow_anonymous(&self) -> bool {
        self.allow_anonymous
    }
}

impl From<Vec<KvConfig>> for AppContext {
    /// 从全局配置装载；未识别或非真值的配置一律保持关闭。
    fn from(configs: Vec<KvConfig>) -> Self {
        let mut context = Self::default();
        for config in configs {
            match config.config_key.parse::<KvConfigKey>() {
                Ok(KvConfigKey::DebugMode) => {
                    context.debug_mode = lib_core::string_is_true(&config.config_value)
                }
                Ok(KvConfigKey::AllowAnonymous) => {
                    context.allow_anonymous = lib_core::string_is_true(&config.config_value)
                }
                _ => {}
            }
        }
        context
    }
}

tokio::task_local! {
    static APP_CONTEXT: Arc<AppContext>;
}

/// 在应用上下文作用域内执行。
pub async fn scope_app<F>(context: Arc<AppContext>, future: F) -> F::Output
where
    F: Future,
{
    APP_CONTEXT.scope(context, future).await
}

/// 取当前请求的应用上下文。
pub fn use_app() -> Result<Arc<AppContext>> {
    APP_CONTEXT
        .try_with(Arc::clone)
        .map_err(|error| anyhow!("当前调用不在应用上下文作用域内：{error:#}"))
}
