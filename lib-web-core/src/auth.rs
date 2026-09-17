//! 统一鉴权：定义请求身份类型与接口鉴权规则，并据此放行接口。
//!
//! 身份的装配（请求头摘要匹配、API Key 与管理员令牌查询）不在这里完成，
//! 本模块只负责类型定义与规则判定。

use std::future::Future;

use anyhow::{Result, anyhow};
use framework_web::AuthRule;

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

