use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

/// 供应商请求异常类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderErrorKind {
    /// 供应商配置不完整或不可用。
    Config,
    /// 连接供应商失败。
    Connect,
    /// 请求超时。
    Timeout,
    /// 供应商返回非成功状态码。
    Http,
    /// 响应内容无法解析。
    Parse,
    /// 供应商返回业务错误。
    Provider,
    /// 客户端取消请求。
    Cancelled,
    /// 其它内部异常。
    Internal,
}

impl ProviderErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Config => "config",
            Self::Connect => "connect",
            Self::Timeout => "timeout",
            Self::Http => "http",
            Self::Parse => "parse",
            Self::Provider => "provider",
            Self::Cancelled => "cancelled",
            Self::Internal => "internal",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Config => "配置错误",
            Self::Connect => "连接失败",
            Self::Timeout => "请求超时",
            Self::Http => "响应状态异常",
            Self::Parse => "响应解析失败",
            Self::Provider => "供应商返回错误",
            Self::Cancelled => "客户端取消",
            Self::Internal => "内部异常",
        }
    }
}

/// 供应商请求异常。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderError {
    /// 异常类型。
    pub kind: ProviderErrorKind,
    /// 异常编码，用于排查。
    pub code: String,
    /// 异常消息。
    pub message: String,
    /// 供应商返回的 HTTP 状态码。
    pub http_status: Option<u16>,
}

impl ProviderError {
    pub fn new(kind: ProviderErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            code: kind.as_str().to_string(),
            message: message.into(),
            http_status: None,
        }
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = code.into();
        self
    }

    pub fn with_status(mut self, status: u16) -> Self {
        self.http_status = Some(status);
        self
    }

    pub fn config(message: impl Into<String>) -> Self {
        Self::new(ProviderErrorKind::Config, message)
    }

    pub fn connect(message: impl Into<String>) -> Self {
        Self::new(ProviderErrorKind::Connect, message)
    }

    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new(ProviderErrorKind::Timeout, message)
    }

    pub fn http(status: u16, message: impl Into<String>) -> Self {
        Self::new(ProviderErrorKind::Http, message).with_status(status)
    }

    pub fn parse(message: impl Into<String>) -> Self {
        Self::new(ProviderErrorKind::Parse, message)
    }

    pub fn provider(message: impl Into<String>) -> Self {
        Self::new(ProviderErrorKind::Provider, message)
    }

    pub fn cancelled() -> Self {
        Self::new(ProviderErrorKind::Cancelled, "客户端取消请求")
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ProviderErrorKind::Internal, message)
    }

    pub fn is_cancelled(&self) -> bool {
        self.kind == ProviderErrorKind::Cancelled
    }
}

impl Display for ProviderError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "[{}] {} ({})",
            self.kind.as_str(),
            self.message,
            self.code
        )
    }
}

impl std::error::Error for ProviderError {}

/// 供应商侧结果类型。
pub type ProviderResult<T> = Result<T, ProviderError>;
