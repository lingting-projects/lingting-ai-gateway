use std::fmt;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use types_admin::Provider;

use crate::chunk::ChunkSink;
use crate::forward::{ForwardCallback, ForwardFailure, ForwardOutcome};
use crate::request::ChatRequest;
use crate::response::ProviderResponse;

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

/// 供应商协议驱动：负责与某一类供应商协议通信，不接触请求日志。
#[async_trait]
pub trait ProviderDriver: Send + Sync {
    /// 协议标识，例如 openai。
    fn name(&self) -> &str;

    /// 对话接口的完整地址。
    fn chat_url(&self, provider: &Provider) -> String;

    /// 模型列表接口的完整地址。
    fn models_url(&self, provider: &Provider) -> String;

    /// 拉取该供应商当前可用的模型名。
    async fn list_models(&self, provider: &Provider) -> Result<Vec<String>>;

    /// 普通对话请求。
    async fn chat(&self, provider: &Provider, request: &ChatRequest) -> Result<ProviderResponse>;

    /// 流式对话请求，分片写入出口，返回聚合后的响应。
    async fn chat_stream(
        &self,
        provider: &Provider,
        request: &ChatRequest,
        sink: &ChunkSink,
    ) -> Result<ProviderResponse>;
}

/// 面向业务的一次转发：绑定协议驱动、供应商、请求体与转发回调。
///
/// 只使用匹配到的第一个供应商，失败即返回，不做熔断与重试。
pub struct ProviderServiceClient {
    driver: Arc<dyn ProviderDriver>,
    provider: Provider,
    request: ChatRequest,
    callback: Arc<dyn ForwardCallback>,
}

impl ProviderServiceClient {
    /// 组装一次转发。
    pub fn new(
        driver: Arc<dyn ProviderDriver>,
        provider: Provider,
        request: ChatRequest,
        callback: Arc<dyn ForwardCallback>,
    ) -> Self {
        Self {
            driver,
            provider,
            request,
            callback,
        }
    }

    /// 本次转发的供应商。
    pub fn provider(&self) -> &Provider {
        &self.provider
    }

    /// 本次转发的请求体。
    pub fn request(&self) -> &ChatRequest {
        &self.request
    }

    /// 本次转发实际访问的地址，写入子请求日志。
    pub fn request_url(&self) -> String {
        self.driver.chat_url(&self.provider)
    }

    /// 发起普通请求。
    pub async fn start(&self) -> Result<ForwardOutcome> {
        self.notify_start().await;

        match self.driver.chat(&self.provider, &self.request).await {
            Ok(response) => {
                let outcome = ForwardOutcome::from_response(response);
                self.notify_success(&outcome).await;
                Ok(outcome)
            }
            Err(error) => {
                let failure = describe_failure(&error, ForwardOutcome::default());
                self.notify_failure(&failure).await;
                Err(error)
            }
        }
    }

    /// 发起流式请求，分片写入出口。
    ///
    /// 流中断时返回错误，但出口中已累积的用量与内容保持不变。
    pub async fn start_stream(&self, sink: &ChunkSink) -> Result<ForwardOutcome> {
        self.notify_start().await;

        match self
            .driver
            .chat_stream(&self.provider, &self.request, sink)
            .await
        {
            Ok(response) => {
                let outcome = ForwardOutcome::from_response(response);
                self.notify_success(&outcome).await;
                Ok(outcome)
            }
            Err(error) => {
                let failure = describe_failure(&error, sink.outcome());
                self.notify_failure(&failure).await;
                Err(error)
            }
        }
    }

    async fn notify_start(&self) {
        if let Err(error) = self.callback.on_start().await {
            tracing::warn!(
                provider = %self.provider.name,
                error = %error,
                "转发开始回调执行失败"
            );
        }
    }

    async fn notify_success(&self, outcome: &ForwardOutcome) {
        if let Err(error) = self.callback.on_success(outcome).await {
            tracing::warn!(
                provider = %self.provider.name,
                error = %error,
                "转发成功回调执行失败"
            );
        }
    }

    async fn notify_failure(&self, failure: &ForwardFailure) {
        if let Err(error) = self.callback.on_failure(failure).await {
            tracing::warn!(
                provider = %self.provider.name,
                error = %error,
                "转发失败回调执行失败"
            );
        }
    }
}

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
