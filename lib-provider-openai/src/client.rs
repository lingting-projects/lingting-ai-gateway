use std::collections::VecDeque;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use lib_provider::{
    ChatRequest, ChatResponse, ChatStream, ChatStreamChunk, ProviderClient, ProviderConfig,
    ProviderError, ProviderResult,
};
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde_json::Value;

use crate::sse::SseDecoder;

/// 连接超时。
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
/// 非流式请求总超时。
const REQUEST_TIMEOUT: Duration = Duration::from_secs(600);
/// 流式请求空闲超时，超过该时间没有新分块即视为中断。
const STREAM_IDLE_TIMEOUT: Duration = Duration::from_secs(300);

/// [OI] 供应商客户端。
pub struct OpenAiClient {
    config: ProviderConfig,
    request: ChatRequest,
    http: reqwest::Client,
}

impl OpenAiClient {
    /// 创建客户端，`stream` 取自请求参数。
    pub fn new(config: ProviderConfig, request: ChatRequest) -> ProviderResult<Self> {
        let http = reqwest::Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .read_timeout(STREAM_IDLE_TIMEOUT)
            .build()
            .map_err(|error| ProviderError::config(format!("创建 HTTP 客户端失败: {error}")))?;
        Ok(Self {
            config,
            request,
            http,
        })
    }

    /// 拉取供应商支持的模型名称。
    pub async fn list_models(config: &ProviderConfig) -> ProviderResult<Vec<String>> {
        let http = reqwest::Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(CONNECT_TIMEOUT * 2)
            .build()
            .map_err(|error| ProviderError::config(format!("创建 HTTP 客户端失败: {error}")))?;
        let url = format!("{}/models", config.base_url.trim_end_matches('/'));
        let response = http
            .get(&url)
            .header(ACCEPT, "application/json")
            .bearer_auth(&config.api_key)
            .send()
            .await
            .map_err(map_request_error)?;
        let status = response.status();
        let body = response.text().await.map_err(map_request_error)?;
        if !status.is_success() {
            return Err(error_from_body(status.as_u16(), &body));
        }
        let value: Value = serde_json::from_str(&body)
            .map_err(|error| ProviderError::parse(format!("模型列表解析失败: {error}")))?;
        let models = value
            .get("data")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.get("id").and_then(Value::as_str))
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Ok(models)
    }

    /// 对话补全地址。
    fn chat_url(&self) -> String {
        format!("{}/chat/completions", self.config.base_url.trim_end_matches('/'))
    }

    /// 供应商返回的错误响应转异常。
    async fn response_error(&self, response: reqwest::Response) -> ProviderError {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        error_from_body(status, &body)
    }
}

#[async_trait]
impl ProviderClient for OpenAiClient {
    fn config(&self) -> &ProviderConfig {
        &self.config
    }

    fn request_model(&self) -> &str {
        &self.request.model
    }

    fn request_url(&self) -> String {
        self.chat_url()
    }

    fn streaming(&self) -> bool {
        self.request.stream_enabled()
    }

    async fn chat(&self) -> ProviderResult<ChatResponse> {
        let mut request = self.request.clone();
        request.stream = Some(false);
        let response = self
            .http
            .post(self.chat_url())
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .bearer_auth(&self.config.api_key)
            .timeout(REQUEST_TIMEOUT)
            .json(&request)
            .send()
            .await
            .map_err(map_request_error)?;
        if !response.status().is_success() {
            return Err(self.response_error(response).await);
        }
        let body = response.text().await.map_err(map_request_error)?;
        serde_json::from_str(&body)
            .map_err(|error| ProviderError::parse(format!("响应解析失败: {error}")))
    }

    async fn chat_stream(&self) -> ProviderResult<ChatStream> {
        let mut request = self.request.clone();
        request.stream = Some(true);
        request.stream_options = Some(lib_provider::StreamOptions {
            include_usage: Some(true),
            extra: Default::default(),
        });
        let response = self
            .http
            .post(self.chat_url())
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "text/event-stream")
            .bearer_auth(&self.config.api_key)
            .json(&request)
            .send()
            .await
            .map_err(map_request_error)?;
        if !response.status().is_success() {
            return Err(self.response_error(response).await);
        }
        let bytes: Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>> =
            Box::pin(response.bytes_stream());
        Ok(Box::pin(SseStream {
            bytes,
            decoder: SseDecoder::default(),
            pending: VecDeque::new(),
            finished: false,
        }))
    }
}

/// 把 SSE 字节流转成模型分块流。
struct SseStream {
    bytes: Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
    decoder: SseDecoder,
    pending: VecDeque<ProviderResult<ChatStreamChunk>>,
    finished: bool,
}

impl Stream for SseStream {
    type Item = ProviderResult<ChatStreamChunk>;

    fn poll_next(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        loop {
            if let Some(item) = this.pending.pop_front() {
                return Poll::Ready(Some(item));
            }
            if this.finished {
                return Poll::Ready(None);
            }
            match this.bytes.poll_next_unpin(context) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => {
                    this.finished = true;
                    if this.decoder.is_done() {
                        return Poll::Ready(None);
                    }
                    return Poll::Ready(Some(Err(ProviderError::provider(
                        "流式响应未正常结束",
                    ))));
                }
                Poll::Ready(Some(Err(error))) => {
                    this.finished = true;
                    return Poll::Ready(Some(Err(map_request_error(error))));
                }
                Poll::Ready(Some(Ok(chunk))) => {
                    this.pending = this
                        .decoder
                        .push(&chunk)
                        .into_iter()
                        .map(|payload| parse_chunk(&payload))
                        .collect();
                }
            }
        }
    }
}

/// 解析单个分块，供应商在流中返回错误时按错误处理。
fn parse_chunk(payload: &str) -> ProviderResult<ChatStreamChunk> {
    let value: Value = serde_json::from_str(payload)
        .map_err(|error| ProviderError::parse(format!("分块解析失败: {error}")))?;
    if let Some(error) = value.get("error") {
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("供应商返回错误");
        return Err(ProviderError::provider(message.to_string()));
    }
    serde_json::from_value(value)
        .map_err(|error| ProviderError::parse(format!("分块解析失败: {error}")))
}

/// 把请求异常转换为供应商异常。
fn map_request_error(error: reqwest::Error) -> ProviderError {
    if error.is_timeout() {
        return ProviderError::timeout(format!("请求供应商超时: {error}"));
    }
    if error.is_connect() {
        return ProviderError::connect(format!("连接供应商失败: {error}"));
    }
    ProviderError::connect(format!("请求供应商失败: {error}"))
}

/// 按供应商返回的错误体构造异常。
fn error_from_body(status: u16, body: &str) -> ProviderError {
    let message = serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|value| {
            value
                .get("error")
                .and_then(|error| error.get("message"))
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| body.trim().to_string());
    let message = if message.is_empty() {
        format!("供应商返回状态码 {status}")
    } else {
        message
    };
    ProviderError::http(status, message)
}
