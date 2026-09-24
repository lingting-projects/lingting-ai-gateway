//! [OI] 接口使用的工具方法。

use anyhow::Result;
use bytes::Bytes;
use framework_core::MultiStringValue;
use lib_web_core::{WebBody, WebContext, WebResponse};
use serde::Serialize;
use serde_json::{Map, Value, json};
use types_admin::entity::RequestStatus;

/// 日志中需要脱敏的头名。
const MASKED_HEADER: &str = "authorization";

/// 脱敏后写入日志的值。
const MASKED_VALUE: &str = "***";
/// 读取请求头的首个值。
pub fn header(context: &WebContext, name: &str) -> Option<String> {
    context.request().headers.get_first(name).cloned()
}

/// 构造 [OI] 兼容的 JSON 响应，直接返回原始 JSON，不套用网关统一响应体。
pub fn json_response<T>(value: &T) -> Result<WebResponse>
where
    T: Serialize,
{
    let body = serde_json::to_vec(value)?;
    let mut headers = MultiStringValue::default();
    headers.set_content_type("application/json; charset=utf-8");
    headers.set_content_length(body.len());

    Ok(WebResponse {
        status: 200,
        headers,
        body: WebBody::Bytes(Bytes::from(body)),
    })
}

/// 请求日志中的请求参数：调试模式记录完整请求体，否则不记录。
pub fn request_params(context: &WebContext, debug_mode: bool) -> Option<Value> {
    if !debug_mode {
        return None;
    }
    context.body_json().ok().cloned()
}

/// 请求日志中的返回内容：调试模式全部保留，非调试模式只在请求失败时保留，
/// 便于排查供应商返回的错误；内容本身由转发链路给出（非流式为原始响应体，流式为累积响应 JSON）。
pub fn response_content(
    content: &Option<Bytes>,
    status: &RequestStatus,
    debug_mode: bool,
) -> Option<String> {
    if !debug_mode && *status != RequestStatus::Failed {
        return None;
    }
    content
        .as_ref()
        .map(|bytes| String::from_utf8_lossy(bytes).to_string())
}

/// 请求日志中的客户端请求头：仅调试模式记录，已移除逐跳头。
pub fn request_headers(context: &WebContext, debug_mode: bool) -> Option<Value> {
    if !debug_mode {
        return None;
    }
    Some(headers_to_json(&context.request().headers))
}

/// 请求日志中实际转发给供应商的请求头：仅调试模式记录，已移除逐跳头且鉴权已脱敏。
pub fn forwarded_headers(
    context: &WebContext,
    api_key: &str,
    debug_mode: bool,
) -> Result<Option<Value>> {
    if !debug_mode {
        return Ok(None);
    }
    let headers = lib_provider::client::build_forward_headers(context, api_key)?;
    let headers = lib_provider::client::headers_to_multi(&headers);
    Ok(Some(headers_to_json(&headers)))
}

/// 请求日志中的响应头：仅调试模式记录，已移除逐跳头。
pub fn response_headers(headers: Option<&MultiStringValue>, debug_mode: bool) -> Option<Value> {
    if !debug_mode {
        return None;
    }
    headers.map(headers_to_json)
}

/// 头信息转为日志用的 JSON：移除逐跳头，鉴权值脱敏。
fn headers_to_json(headers: &MultiStringValue) -> Value {
    let mut values = Map::new();
    headers.for_each(|name, items| {
        if lib_provider::client::is_skipped_header(name) {
            return;
        }
        let items = if name.eq_ignore_ascii_case(MASKED_HEADER) {
            vec![MASKED_VALUE.to_string()]
        } else {
            items.clone()
        };
        values.insert(name.clone(), json!(items));
    });
    Value::Object(values)
}
