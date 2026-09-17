//! 应用级全局上下文：从 kv_config 装载的调试模式、匿名访问与管理员令牌。

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use types_admin::KvConfig;

/// 全局配置键：调试模式，开启后记录完整请求参数与返回内容。
pub const CONFIG_DEBUG_MODE: &str = "debug_mode";
/// 全局配置键：是否允许匿名访问 AI 接口。
pub const CONFIG_ALLOW_ANONYMOUS: &str = "allow_anonymous";
/// 全局配置键：管理员令牌，库中保存的是 SHA1。
pub const CONFIG_ADMIN_TOKEN: &str = "admin_token";

/// 应用全局上下文。
#[derive(Debug, Clone, Default)]
pub struct AppContext {
    debug_mode: bool,
    allow_anonymous: bool,
    admin_token_hash: Option<String>,
    configs: HashMap<String, String>,
}

impl AppContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// 从全局配置装载。
    pub fn load_from_kv_configs(configs: Vec<KvConfig>) -> Self {
        let mut context = Self::new();
        for config in configs {
            context.set_config(config.config_key, config.config_value);
        }
        context
    }

    /// 写入一项配置，已识别的键同时刷新快捷字段。
    pub fn set_config(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let key = key.into();
        let value = value.into();
        match key.as_str() {
            CONFIG_DEBUG_MODE => self.debug_mode = parse_bool(&value),
            CONFIG_ALLOW_ANONYMOUS => self.allow_anonymous = parse_bool(&value),
            CONFIG_ADMIN_TOKEN => {
                self.admin_token_hash = Some(value.clone()).filter(|item| !item.is_empty());
            }
            _ => {}
        }
        self.configs.insert(key, value);
    }

    /// 是否开启调试模式。
    pub fn debug_mode(&self) -> bool {
        self.debug_mode
    }

    /// 是否允许匿名访问。
    pub fn allow_anonymous(&self) -> bool {
        self.allow_anonymous
    }

    /// 管理员令牌摘要。
    pub fn admin_token_hash(&self) -> Option<&str> {
        self.admin_token_hash.as_deref()
    }

    /// 读取原始配置值。
    pub fn get_config(&self, key: &str) -> Option<&str> {
        self.configs.get(key).map(String::as_str)
    }
}

/// 解析布尔配置，兼容 true/1/yes/on。
fn parse_bool(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true" | "1" | "yes" | "on"
    )
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
        .map_err(|error| anyhow!("当前调用不在应用上下文作用域内：{error}"))
}

