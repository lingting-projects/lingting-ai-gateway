use anyhow::Result;
use lib_core::hash;
use lib_web_core::{AppContext, AuthKind, Authorization};
use types_admin::KvConfigKey;

use crate::service::{ApiKeyService, KvConfigService};

/// Web 层编排：统一装配请求鉴权身份与全局配置。
pub struct WebManager {
    api_key_service: ApiKeyService,
    kv_config_service: KvConfigService,
}

impl WebManager {
    /// 绑定当前请求的数据库连接池。
    pub fn new() -> Result<Self> {
        Ok(Self {
            api_key_service: ApiKeyService::new()?,
            kv_config_service: KvConfigService::new()?,
        })
    }

    /// 解析 Authorization 请求头得到请求身份。
    ///
    /// - 请求头 trim 后为空：匿名身份。
    /// - 摘要命中 API Key：API 身份，值为密钥 ID。
    /// - 管理员令牌不存在或 trim 后为空：管理员身份，值为空串。
    /// - 摘要与管理员令牌一致：管理员身份，值为摘要。
    /// - 其余情况：匿名身份。
    pub async fn build_authorization(&self, value: &str) -> Result<Authorization> {
        let value = value.trim();
        if value.is_empty() {
            return Ok(Authorization::anonymous());
        }

        let hashed = hash(value);
        if let Some(api_key) = self.api_key_service.find_by_key_hash(&hashed).await? {
            return Ok(Authorization::new(AuthKind::Api, api_key.id.to_string()));
        }

        let admin_token = self
            .kv_config_service
            .find_value(KvConfigKey::AdminToken)
            .await?
            .unwrap_or_default();
        let admin_token = admin_token.trim();
        if admin_token.is_empty() {
            return Ok(Authorization::new(AuthKind::Admin, ""));
        }

        if admin_token == hashed {
            return Ok(Authorization::new(AuthKind::Admin, hashed));
        }

        Ok(Authorization::anonymous())
    }

    /// 一次性装载应用全局上下文。
    pub async fn build_app(&self) -> Result<AppContext> {
        let configs = self
            .kv_config_service
            .find_keys(&[KvConfigKey::DebugMode, KvConfigKey::AllowAnonymous])
            .await?;

        Ok(AppContext::from(configs))
    }
}
