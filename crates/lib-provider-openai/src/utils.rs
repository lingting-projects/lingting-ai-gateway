//! [OI] 接口使用的工具方法。

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use bytes::Bytes;
use framework_core::MultiStringValue;
use lib_provider::{ForwardCallback, ForwardFailure, ForwardOutcome};
use lib_web_core::{WebBody, WebResponse};
use reqwest::StatusCode;
use reqwest::header::HeaderMap;
use types_admin::entity::Provider;

use crate::chat_response::{ChatResponse, ProviderResponse};
use crate::responses_response::ResponsesResponse;

/// 对话补全接口的路径后缀。
pub const CHAT_COMPLETIONS_SUFFIX: &str = "/chat/completions";

/// 供应商的对话补全地址。
pub fn chat_url(provider: &Provider) -> String {
    format!(
        "{}{}",
        provider.base_url.trim_end_matches('/'),
        CHAT_COMPLETIONS_SUFFIX
    )
}

/// Responses 接口的路径后缀。
pub const RESPONSES_SUFFIX: &str = "/responses";

/// 供应商的 Responses 地址。
pub fn responses_url(provider: &Provider) -> String {
    format!(
        "{}{}",
        provider.base_url.trim_end_matches('/'),
        RESPONSES_SUFFIX
    )
}

/// 由 chat 供应商响应体构造日志用的转发结果；解析失败时只保留状态码。
///
/// 原始字节仅在调试模式下收集。
pub fn chat_outcome(http_status: u16, body: &Bytes, debug_mode: bool) -> ForwardOutcome {
    let status = i32::from(http_status);
    let mut outcome = match serde_json::from_slice::<ChatResponse>(body) {
        Ok(response) => ProviderResponse {
            response,
            http_status: status,
        }
        .into_outcome(),
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

/// 由 Responses 供应商响应体构造日志用的转发结果；解析失败时只保留状态码。
///
/// 原始字节仅在调试模式下收集。
pub fn responses_outcome(http_status: u16, body: &Bytes, debug_mode: bool) -> ForwardOutcome {
    let status = i32::from(http_status);
    let mut outcome = match serde_json::from_slice::<ResponsesResponse>(body) {
        Ok(response) => ForwardOutcome {
            token_info: response.token_info(),
            content: None,
            response_headers: None,
            return_model: response.model.clone(),
            finish_reason: response.finish_reason(),
            provider_request_id: response.id.clone(),
            http_status: Some(status),
        },
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

/// 供应商响应头转成网关响应头；逐跳头已由 `ForwardRequest::call` 移除。
pub fn response_headers(headers: &HeaderMap) -> MultiStringValue {
    lib_provider::client::headers_to_multi(headers)
}

/// 按调试模式把供应商响应头并入转发结果；关闭调试时不收集。
pub fn record_response_headers(
    outcome: &mut ForwardOutcome,
    headers: &MultiStringValue,
    debug_mode: bool,
) {
    if debug_mode {
        outcome.response_headers = Some(headers.clone());
    }
}
/// 传输层失败：构造失败信息、通知回调，并把错误原样抛出。
pub async fn fail_transport(
    callback: &Arc<dyn ForwardCallback>,
    error: reqwest::Error,
) -> Result<WebResponse> {
    let failure = ForwardFailure::new(
        transport_error_type(&error),
        error.to_string(),
        ForwardOutcome::default(),
    );
    notify(callback.on_failure(&failure).await);
    Err(error.into())
}

/// 结束一次已收到完整响应体的转发：通知回调，并把响应原样返回。
///
/// 普通转发与流式转发收到完整响应时共用这一条出口，转发结果由各协议自行解析。
pub async fn finish_response(
    callback: &Arc<dyn ForwardCallback>,
    status: StatusCode,
    headers: MultiStringValue,
    body: Bytes,
    mut outcome: ForwardOutcome,
) -> Result<WebResponse> {
    record_response_headers(&mut outcome, &headers, callback.is_debug());
    if status.is_success() {
        notify(callback.on_success(&outcome).await);
    } else {
        let mut failure = ForwardFailure::new(
            "status",
            format!("供应商返回状态码 {}", status.as_u16()),
            outcome,
        );
        failure.http_status = Some(status.as_u16() as i32);
        notify(callback.on_failure(&failure).await);
    }

    Ok(WebResponse {
        status: status.as_u16(),
        headers,
        body: WebBody::Bytes(body),
    })
}

/// 回调失败只记录日志，不影响请求本身。
///
/// 用 `{error:#}` 输出完整错误链：`{error}` 只打印最外层 context，
/// 数据库错误等根因会被丢掉，排障时看不到真正原因。
pub fn notify(result: Result<()>) {
    if let Err(error) = result {
        tracing::warn!("转发回调执行失败：{error:#}");
    }
}

/// 传输层错误的类别。
pub fn transport_error_type(error: &reqwest::Error) -> &'static str {
    if error.is_timeout() {
        "timeout"
    } else if error.is_connect() {
        "connect"
    } else {
        "internal"
    }
}
