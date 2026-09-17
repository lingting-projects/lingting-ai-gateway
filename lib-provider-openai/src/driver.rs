use std::time::Duration;

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use futures_util::StreamExt;
use lib_provider::{
    ChatRequest, ChatResponse, ChunkSink, ForwardOutcome, ProviderDriver, ProviderError,
    ProviderResponse,
};
use serde::Deserialize;
use serde_json::{Map, Value};
use types_admin::Provider;

use crate::sse::SseDecoder;

/// 流结束标记。
const DONE: &str = "[DONE]";

/// 错误正文在日志中的最大长度。
const ERROR_BODY_LIMIT: usize = 512;

/// [OI] 协议驱动。
pub struct OpenAiDriver {
    client: reqwest::Client,
    request_timeout: Duration,
}

impl OpenAiDriver {
    /// 创建驱动。
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(30))
            .build()
            .expect("构建 HTTP 客户端失败");

        Self {
            client,
            request_timeout: Duration::from_secs(300),
        }
    }

    fn build(
        &self,
        provider: &Provider,
        url: &str,
        payload: Option<&Value>,
    ) -> reqwest::RequestBuilder {
        let mut builder = self.client.post(url);
        if !provider.api_key.is_empty() {
            builder = builder.bearer_auth(&provider.api_key);
        }
        if let Some(payload) = payload {
            builder = builder.json(payload);
        }
        builder
    }
}

impl Default for OpenAiDriver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProviderDriver for OpenAiDriver {
    fn name(&self) -> &str {
        "openai"
    }

    fn chat_url(&self, provider: &Provider) -> String {
        join_url(&provider.base_url, "chat/completions")
    }

    fn models_url(&self, provider: &Provider) -> String {
        join_url(&provider.base_url, "models")
    }

    async fn list_models(&self, provider: &Provider) -> Result<Vec<String>> {
        let url = self.models_url(provider);
        let response = self
            .build(provider, &url, None)
            .timeout(self.request_timeout)
            .send()
            .await
            .map_err(send_error)?;

        let http_status = response.status().as_u16() as i32;
        let body = response.text().await.map_err(send_error)?;
        if !(200..300).contains(&http_status) {
            return Err(status_error(http_status, &body));
        }

        let list: ModelListResponse = serde_json::from_str(&body)
            .map_err(|error| anyhow!(ProviderError::new("parse", format!("解析模型列表失败: {error}"))))?;

        Ok(list
            .data
            .into_iter()
            .map(|item| item.id)
            .filter(|id| !id.is_empty())
            .collect())
    }

    async fn chat(&self, provider: &Provider, request: &ChatRequest) -> Result<ProviderResponse> {
        let url = self.chat_url(provider);
        let mut payload = serde_json::to_value(request)?;
        set_stream(&mut payload, false);

        let response = self
            .build(provider, &url, Some(&payload))
            .timeout(self.request_timeout)
            .send()
            .await
            .map_err(send_error)?;

        let http_status = response.status().as_u16() as i32;
        let body = response.text().await.map_err(send_error)?;
        if !(200..300).contains(&http_status) {
            return Err(status_error(http_status, &body));
        }

        let response = serde_json::from_str(&body).map_err(|error| {
            anyhow!(ProviderError::new("parse", format!("解析响应失败: {error}")))
        })?;

        Ok(ProviderResponse {
            response,
            http_status,
        })
    }

    async fn chat_stream(
        &self,
        provider: &Provider,
        request: &ChatRequest,
        sink: &ChunkSink,
    ) -> Result<ProviderResponse> {
        let url = self.chat_url(provider);
        let mut payload = serde_json::to_value(request)?;
        set_stream(&mut payload, true);
        set_include_usage(&mut payload);

        let response = self
            .build(provider, &url, Some(&payload))
            .send()
            .await
            .map_err(send_error)?;

        let http_status = response.status().as_u16() as i32;
        if !(200..300).contains(&http_status) {
            let body = response.text().await.unwrap_or_default();
            return Err(status_error(http_status, &body));
        }

        let mut aggregated = ChatResponse::default();
        let mut decoder = SseDecoder::new();
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|error| {
                anyhow!(ProviderError::new("stream", format!("读取流式响应失败: {error}")))
            })?;

            for payload in decoder.push(&chunk) {
                if payload == DONE {
                    sink.send(format!("data: {DONE}\n\n"));
                    return Ok(ProviderResponse {
                        response: aggregated,
                        http_status,
                    });
                }

                let parsed: ChatResponse = serde_json::from_str(&payload).map_err(|error| {
                    anyhow!(ProviderError::new(
                        "parse",
                        format!("解析流式分片失败: {error}")
                    ))
                })?;

                aggregated.merge_chunk(&parsed);
                // 每收到一片就刷新累积结果，流中断时已累积的用量不会丢失。
                sink.record(ForwardOutcome::from_response(ProviderResponse {
                    response: aggregated.clone(),
                    http_status,
                }));
                sink.send(format!("data: {payload}\n\n"));
            }

            if sink.is_closed() {
                return Err(anyhow!(ProviderError::new(
                    "cancelled",
                    "客户端已断开连接"
                )));
            }
        }

        Ok(ProviderResponse {
            response: aggregated,
            http_status,
        })
    }
}

/// 拼接供应商基础地址与接口路径。
fn join_url(base_url: &str, path: &str) -> String {
    format!(
        "{}/{}",
        base_url.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

/// 设置流式开关，同时清掉对端的用量上报选项。
fn set_stream(payload: &mut Value, stream: bool) {
    let Value::Object(map) = payload else {
        return;
    };
    map.insert("stream".into(), Value::Bool(stream));
    if !stream {
        map.remove("stream_options");
    }
}

/// 流式请求强制要求供应商在末尾上报用量。
fn set_include_usage(payload: &mut Value) {
    let Value::Object(map) = payload else {
        return;
    };
    let options = map
        .entry("stream_options")
        .or_insert_with(|| Value::Object(Map::new()));
    if let Value::Object(options) = options {
        options.insert("include_usage".into(), Value::Bool(true));
    }
}

/// 网络层错误分类。
fn send_error(error: reqwest::Error) -> anyhow::Error {
    let error_type = if error.is_timeout() {
        "timeout"
    } else if error.is_connect() {
        "connect"
    } else if error.is_body() || error.is_decode() {
        "parse"
    } else {
        "request"
    };
    anyhow!(ProviderError::new(error_type, error.to_string()))
}

/// 非 2xx 响应，尽量提取供应商给出的错误码与描述。
fn status_error(http_status: i32, body: &str) -> anyhow::Error {
    let parsed: Option<Value> = serde_json::from_str(body).ok();
    let error = parsed.as_ref().and_then(|value| value.get("error"));

    let message = error
        .and_then(|error| error.get("message"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| truncate(body, ERROR_BODY_LIMIT));

    let code = error
        .and_then(|error| error.get("code"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    anyhow!(ProviderError::new("status", message)
        .with_status(http_status)
        .with_code(code))
}

/// 截断过长的错误正文。
fn truncate(text: &str, limit: usize) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= limit {
        return trimmed.to_string();
    }
    trimmed.chars().take(limit).collect::<String>() + "..."
}

/// 模型列表响应。
#[derive(Debug, Deserialize)]
struct ModelListResponse {
    #[serde(default)]
    data: Vec<ModelItem>,
}

/// 模型列表中的一项。
#[derive(Debug, Deserialize)]
struct ModelItem {
    #[serde(default)]
    id: String,
}
