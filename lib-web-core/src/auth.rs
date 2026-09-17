//! 统一鉴权：把 Authorization 请求头解析为请求身份，并按接口规则放行。
//!
//! 数据库只保存 API Key 的 SHA1，请求头同样取 SHA1 后匹配，
//! 因此原始 key 不会出现在内存以外的任何位置。

use std::collections::HashMap;
use std::future::Future;

use anyhow::{Result, anyhow};
use framework_web::{AuthRule, WebError};
use sha1::Digest;
use types_admin::ApiKey;

/// 管理员接口规则标识。
pub const ROLE_ADMIN: &str = "admin";
/// AI 接口规则标识。
pub const ROLE_API: &str = "api";

/// 鉴权类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthKind {
    /// 通过 API Key 鉴权。
    Api,
    /// 管理员身份。
    Admin,
    /// 匿名访问。
    Anonymous,
}

impl AuthKind {
    /// 存储与日志使用的标识。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Api => "api",
            Self::Admin => "admin",
            Self::Anonymous => "anonymous",
        }
    }

    /// 中文展示名。
    pub fn label(self) -> &'static str {
        match self {
            Self::Api => "API",
            Self::Admin => "管理员",
            Self::Anonymous => "匿名",
        }
    }
}

/// 请求身份。
#[derive(Debug, Clone)]
pub struct Authorization {
    kind: AuthKind,
    value: String,
}

impl Authorization {
    pub fn new(kind: AuthKind, value: impl Into<String>) -> Self {
        Self {
            kind,
            value: value.into(),
        }
    }

    /// 匿名身份。
    pub fn anonymous() -> Self {
        Self::new(AuthKind::Anonymous, "")
    }

    pub fn kind(&self) -> AuthKind {
        self.kind
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn is_api(&self) -> bool {
        self.kind == AuthKind::Api
    }

    pub fn is_admin(&self) -> bool {
        self.kind == AuthKind::Admin
    }

    pub fn is_anonymous(&self) -> bool {
        self.kind == AuthKind::Anonymous
    }

    /// 写入请求日志的凭证标识，匿名请求为空串。
    pub fn credential_id(&self) -> &str {
        match self.kind {
            AuthKind::Anonymous => "",
            _ => &self.value,
        }
    }
}

/// 计算字符串的 SHA1 十六进制摘要。
pub fn sha1_hex(value: &str) -> String {
    format!(
        "{:x}",
        sha1::Sha1::new().chain_update(value.as_bytes()).finalize()
    )
}

/// 鉴权服务：持有 API Key 摘要与管理员令牌摘要。
#[derive(Debug, Default)]
pub struct AuthorizationService {
    api_keys: HashMap<String, i64>,
    admin_token_hash: Option<String>,
}

impl AuthorizationService {
    pub fn new() -> Self {
        Self::default()
    }

    /// 装载 API Key，空摘要不参与匹配。
    pub fn set_api_keys(&mut self, api_keys: Vec<ApiKey>) {
        self.api_keys = api_keys
            .into_iter()
            .filter(|api_key| !api_key.key_hash.is_empty())
            .map(|api_key| (api_key.key_hash, api_key.id))
            .collect();
    }

    /// 装载管理员令牌摘要，空值表示未配置。
    pub fn set_admin_token_hash(&mut self, hash: Option<String>) {
        self.admin_token_hash = hash.filter(|value| !value.is_empty());
    }

    /// 已装载的 API Key 数量。
    pub fn api_key_count(&self) -> usize {
        self.api_keys.len()
    }

    /// 解析请求头得到请求身份。
    ///
    /// - 请求头为空：匿名身份。
    /// - 摘要命中 API Key：API 身份，value 为密钥 ID。
    /// - 未配置管理员令牌：直接视为管理员身份。
    /// - 摘要命中管理员令牌：管理员身份，value 为原始请求头。
    /// - 其余情况：鉴权失败。
    pub fn resolve(&self, header: Option<&str>) -> Result<Authorization> {
        let Some(value) = header.map(str::trim).filter(|value| !value.is_empty()) else {
            return Ok(Authorization::anonymous());
        };

        let hash = sha1_hex(value);
        if let Some(id) = self.api_keys.get(&hash) {
            return Ok(Authorization::new(AuthKind::Api, id.to_string()));
        }

        match self.admin_token_hash.as_ref() {
            None => Ok(Authorization::new(AuthKind::Admin, "")),
            Some(expected) if expected == &hash => Ok(Authorization::new(AuthKind::Admin, value)),
            Some(_) => Err(WebError::unauthorized("Authorization 请求头无效").into()),
        }
    }
}

/// 接口鉴权规则扩展。
pub trait AuthRuleExt {
    /// 管理接口：需要管理员身份。
    fn admin() -> AuthRule;

    /// AI 接口：需要 API Key，允许匿名访问时直接放行。
    fn api() -> AuthRule;

    /// 公开接口：不做鉴权。
    fn public() -> AuthRule;
}

impl AuthRuleExt for AuthRule {
    fn admin() -> AuthRule {
        AuthRule {
            anonymous: Some(false),
            roles: Some(vec![ROLE_ADMIN.to_string()]),
            ..Default::default()
        }
    }

    fn api() -> AuthRule {
        AuthRule {
            anonymous: Some(false),
            roles: Some(vec![ROLE_API.to_string()]),
            ..Default::default()
        }
    }

    fn public() -> AuthRule {
        AuthRule::anonymous()
    }
}

/// 接口规则是否放行当前身份。
///
/// `allow_anonymous` 为全局配置，仅对 AI 接口生效。
pub fn allows(rule: &AuthRule, authorization: &Authorization, allow_anonymous: bool) -> bool {
    if rule.anonymous == Some(true) {
        return true;
    }
    match rule_role(rule) {
        Some(ROLE_ADMIN) => authorization.is_admin(),
        Some(ROLE_API) => allow_anonymous || authorization.is_api(),
        _ => !authorization.is_anonymous(),
    }
}

fn rule_role(rule: &AuthRule) -> Option<&str> {
    match rule.roles.as_deref() {
        Some([role]) => Some(role.as_str()),
        _ => None,
    }
}

tokio::task_local! {
    static AUTHORIZATION: Authorization;
}

/// 在请求身份作用域内执行。
pub async fn scope_authorization<F>(authorization: Authorization, future: F) -> F::Output
where
    F: Future,
{
    AUTHORIZATION.scope(authorization, future).await
}

/// 取当前请求的鉴权身份。
pub fn use_authorization() -> Result<Authorization> {
    AUTHORIZATION
        .try_with(Clone::clone)
        .map_err(|error| anyhow!("当前调用不在鉴权上下文作用域内：{error}"))
}

