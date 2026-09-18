//! [OI] 接口使用的工具方法。

use anyhow::Result;
use bytes::Bytes;
use framework_core::MultiStringValue;
use lib_web_core::{WebBody, WebContext, WebResponse};
use serde::Serialize;
use serde_json::Value;

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

/// 请求日志中的返回内容：仅调试模式记录，保留供应商返回的原始字节。
pub fn response_content(content: &Option<Bytes>, debug_mode: bool) -> Option<String> {
    if !debug_mode {
        return None;
    }
    content
        .as_ref()
        .map(|bytes| String::from_utf8_lossy(bytes).to_string())
}
