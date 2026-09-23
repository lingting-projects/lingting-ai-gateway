use std::future::Future;

use anyhow::{Result, anyhow};

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
        .map_err(|error| anyhow!("当前调用不在鉴权上下文作用域内：{error:#}"))
}
