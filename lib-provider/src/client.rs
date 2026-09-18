use std::fmt;

use crate::forward::{ForwardFailure, ForwardOutcome};

/// 供应商请求错误，携带错误类别与供应商返回的状态码。
#[derive(Debug)]
pub struct ProviderError {
    /// 错误类别：timeout、connect、status、parse、provider。
    pub error_type: String,
    /// 供应商返回的错误码。
    pub error_code: String,
    /// 供应商返回的 HTTP 状态码。
    pub http_status: Option<i32>,
    /// 错误描述。
    pub message: String,
}

impl ProviderError {
    /// 按类别与描述构造。
    pub fn new(error_type: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error_type: error_type.into(),
            error_code: String::new(),
            http_status: None,
            message: message.into(),
        }
    }

    /// 附带供应商返回的状态码。
    pub fn with_status(mut self, http_status: i32) -> Self {
        self.http_status = Some(http_status);
        self
    }

    /// 附带供应商返回的错误码。
    pub fn with_code(mut self, error_code: impl Into<String>) -> Self {
        self.error_code = error_code.into();
        self
    }
}

impl fmt::Display for ProviderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for ProviderError {}

/// 把请求错误整理为失败信息，保留已累积的结果。
fn describe_failure(error: &anyhow::Error, outcome: ForwardOutcome) -> ForwardFailure {
    match error.downcast_ref::<ProviderError>() {
        Some(provider_error) => ForwardFailure {
            error_type: provider_error.error_type.clone(),
            error_code: provider_error.error_code.clone(),
            message: provider_error.message.clone(),
            http_status: provider_error.http_status,
            outcome,
        },
        None => ForwardFailure::new("internal", error.to_string(), outcome),
    }
}
