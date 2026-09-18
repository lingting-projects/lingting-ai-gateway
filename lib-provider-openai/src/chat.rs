//! 普通（非流式）转发：把客户端请求原样发给供应商，再把供应商响应原样返回。

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use bytes::Bytes;
use framework_core::MultiStringValue;
use lib_provider::response::{ChatResponse, ProviderResponse};
use lib_provider::{ForwardCallback, ForwardFailure, ForwardOutcome, build_client};
use lib_web_core::{WebBody, WebContext, WebResponse};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderName, HeaderValue};
use types_admin::entity::Provider;

use crate::utils;

/// 转发时跳过的请求头与响应头。
///
/// 逐跳头只对单条连接有效；`host` 与 `content-length` 必须按实际连接和请求体重新生成，
/// 原样转发会与 reqwest 冲突。
const SKIPPED_HEADERS: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
    "host",
    "content-length",
];

/// [OI] 普通转发客户端。
pub struct OpenaiChatClient {
    provider: Provider,
    web_context: Arc<WebContext>,
    callback: Arc<dyn ForwardCallback>,
}

impl OpenaiChatClient {
    /// 绑定供应商、当前请求上下文与转发回调。
    pub fn new(
        provider: Provider,
        web_context: Arc<WebContext>,
        callback: Arc<dyn ForwardCallback>,
    ) -> Self {
        Self {
            provider,
            web_context,
            callback,
        }
    }

    /// 发起请求并把供应商响应原样返回。
    pub async fn call(&self) -> Result<WebResponse> {
        let client = build_client(&self.provider);
        let request = client
            .post(utils::chat_url(&self.provider))
            .headers(self.forward_headers()?)
            .body(self.web_context.body());

        notify(self.callback.on_start().await);

        let response = match request.send().await {
            Ok(response) => response,
            Err(error) => return self.fail_transport(error).await,
        };
        let status = response.status();
        let headers = response.headers().clone();
        let body = match response.bytes().await {
            Ok(body) => body,
            Err(error) => return self.fail_transport(error).await,
        };

        let outcome = outcome(status.as_u16(), &body, self.callback.is_debug());
        if status.is_success() {
            notify(self.callback.on_success(&outcome).await);
        } else {
            let mut failure = ForwardFailure::new(
                "status",
                format!("供应商返回状态码 {}", status.as_u16()),
                outcome,
            );
            failure.http_status = Some(status.as_u16() as i32);
            notify(self.callback.on_failure(&failure).await);
        }

        Ok(WebResponse {
            status: status.as_u16(),
            headers: response_headers(&headers),
            body: WebBody::Bytes(body),
        })
    }

    /// 原样转发客户端请求头，鉴权换成供应商自己的 api_key。
    fn forward_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        self.web_context.request().headers.for_each(|name, values| {
            if is_skipped_header(name) {
                return;
            }
            let Ok(name) = HeaderName::from_bytes(name.as_bytes()) else {
                return;
            };
            for value in values {
                if let Ok(value) = HeaderValue::from_str(value) {
                    headers.append(name.clone(), value);
                }
            }
        });

        let authorization = HeaderValue::from_str(&format!("Bearer {}", self.provider.api_key))
            .map_err(|error| anyhow!("供应商 api_key 无法作为请求头使用：{error}"))?;
        headers.insert(AUTHORIZATION, authorization);
        Ok(headers)
    }

    /// 传输层失败：写失败日志后把错误原样抛出。
    async fn fail_transport(&self, error: reqwest::Error) -> Result<WebResponse> {
        let failure = ForwardFailure::new(
            transport_error_type(&error),
            error.to_string(),
            ForwardOutcome::default(),
        );
        notify(self.callback.on_failure(&failure).await);
        Err(error.into())
    }
}

/// 由供应商响应体构造日志用的转发结果；解析失败时只保留状态码。
/// 原始字节仅在调试模式下收集。
fn outcome(http_status: u16, body: &Bytes, debug_mode: bool) -> ForwardOutcome {
    let status = i32::from(http_status);
    let mut outcome = match serde_json::from_slice::<ChatResponse>(body) {
        Ok(response) => ForwardOutcome::from_response(ProviderResponse {
            response,
            http_status: status,
        }),
        Err(_) => ForwardOutcome {
            http_status: Some(status),
            ..ForwardOutcome::default()
        },
    };
    if debug_mode {
        outcome.content = Some(body.clone());
    }
    outcome
}

/// 供应商响应头转成网关响应头，跳过逐跳头。
fn response_headers(headers: &HeaderMap) -> MultiStringValue {
    let mut values = HashMap::<String, Vec<String>>::new();
    for (name, value) in headers {
        if is_skipped_header(name.as_str()) {
            continue;
        }
        let Ok(value) = value.to_str() else {
            continue;
        };
        values
            .entry(name.as_str().to_string())
            .or_default()
            .push(value.to_string());
    }
    MultiStringValue::create(true, values)
}

/// 是否为需要跳过的请求头 / 响应头。
fn is_skipped_header(name: &str) -> bool {
    SKIPPED_HEADERS
        .iter()
        .any(|skipped| name.eq_ignore_ascii_case(skipped))
}

/// 传输层错误的类别。
fn transport_error_type(error: &reqwest::Error) -> &'static str {
    if error.is_timeout() {
        "timeout"
    } else if error.is_connect() {
        "connect"
    } else {
        "internal"
    }
}

/// 回调失败只记录日志，不影响请求本身。
fn notify(result: Result<()>) {
    if let Err(error) = result {
        tracing::warn!("转发回调执行失败：{error}");
    }
}
