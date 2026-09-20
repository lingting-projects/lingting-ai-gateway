use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use framework_core::MultiStringValue;
use types_admin::TokenInfo;

/// 一次转发累积的用量与返回信息。
#[derive(Debug, Clone, Default)]
pub struct ForwardOutcome {
    /// 输入、输出、缓存读、缓存写、推理与总量。
    pub token_info: TokenInfo,
    /// 返回内容的原始字节，是否落库由回调按调试模式决定。
    pub content: Option<Bytes>,
    /// 供应商返回的响应头，是否落库由回调按调试模式决定。
    pub response_headers: Option<MultiStringValue>,
    /// 供应商实际返回的模型名。
    pub return_model: String,
    /// 结束原因。
    pub finish_reason: String,
    /// 供应商返回的请求 ID。
    pub provider_request_id: String,
    /// 供应商返回的 HTTP 状态码。
    pub http_status: Option<i32>,
}

/// 转发失败信息，携带失败前已累积的用量与内容。
#[derive(Debug, Clone)]
pub struct ForwardFailure {
    /// 错误类别，例如 timeout、connect、parse、provider。
    pub error_type: String,
    /// 供应商返回的错误码。
    pub error_code: String,
    /// 错误描述。
    pub message: String,
    /// 供应商返回的 HTTP 状态码。
    pub http_status: Option<i32>,
    /// 失败前已累积的结果，不允许丢弃。
    pub outcome: ForwardOutcome,
}

impl ForwardFailure {
    /// 由错误与已累积结果构造。
    pub fn new(
        error_type: impl Into<String>,
        message: impl Into<String>,
        outcome: ForwardOutcome,
    ) -> Self {
        Self {
            error_type: error_type.into(),
            error_code: String::new(),
            message: message.into(),
            http_status: None,
            outcome,
        }
    }
}

/// 转发回调。
///
/// 客户端只负责把进度交出来，日志字段由调用方在回调中落库；
/// 回调返回的错误不得影响正常请求的完成。
#[async_trait]
pub trait ForwardCallback: Send + Sync {
    /// 是否处于调试模式；客户端据此决定是否把返回的原始内容交给回调。
    fn is_debug(&self) -> bool {
        false
    }

    /// 请求已发出。
    async fn on_start(&self) -> Result<()> {
        Ok(())
    }

    /// 转发过程中产生新的累积结果。
    async fn on_progress(&self, _outcome: &ForwardOutcome) -> Result<()> {
        Ok(())
    }

    /// 正常结束。
    async fn on_success(&self, _outcome: &ForwardOutcome) -> Result<()> {
        Ok(())
    }

    /// 失败。
    async fn on_failure(&self, _failure: &ForwardFailure) -> Result<()> {
        Ok(())
    }

    /// 客户端取消请求；携带取消前已累积的结果。
    async fn on_cancel(&self, _outcome: &ForwardOutcome) -> Result<()> {
        Ok(())
    }
}
