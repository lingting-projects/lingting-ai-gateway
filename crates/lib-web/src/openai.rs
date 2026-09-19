//! [OI] 兼容接口声明，业务实现见 `service_provider::openai::OpenaiService`。

use anyhow::Result;
use framework_web::{WebError, WebResponse, use_web, web_api_get, web_api_post};
use lib_provider::RouteRequest;
use serde_json::Value;
use service_provider::openai::OpenaiService;

/// 聊天接口
#[web_api_post(path = "/v1/chat/completions")]
pub async fn chat() -> Result<WebResponse> {
    let web_context = use_web()?;
    let request = forward_request(web_context.body_json()?)?;

    OpenaiService::chat(request).await
}

/// Responses 接口
#[web_api_post(path = "/v1/responses")]
pub async fn responses() -> Result<WebResponse> {
    let web_context = use_web()?;
    let request = forward_request(web_context.body_json()?)?;

    OpenaiService::responses(request).await
}

/// 由请求体构造转发请求。
fn forward_request(body: &Value) -> Result<RouteRequest> {
    let model = body
        .get("model")
        .and_then(Value::as_str)
        .ok_or_else(|| WebError::parameter("缺少 model 参数", "model"))?
        .to_string();

    Ok(RouteRequest {
        model,
        stream: body.get("stream").and_then(Value::as_bool).unwrap_or(false),
        session_id: session_id(body),
    })
}

/// 会话标识：优先取 session_id / sessionId，其次取 user。
fn session_id(body: &Value) -> Option<String> {
    ["session_id", "sessionId", "user"]
        .iter()
        .find_map(|name| body.get(name).and_then(Value::as_str).map(str::to_string))
}

/// 模型列表接口
#[web_api_get(path = "/v1/models")]
pub async fn models() -> Result<WebResponse> {
    OpenaiService::models().await
}
