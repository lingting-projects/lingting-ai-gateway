//! [OI] 兼容接口声明，业务实现见 `service_provider::openai::OpenaiService`。

use anyhow::Result;
use framework_web::{WebError, WebResponse, use_web, web_api_post};
use lib_provider::ChatRequest;
use serde_json::Value;
use service_provider::openai::OpenaiService;

/// 聊天接口
#[web_api_post(path = "/chat")]
pub async fn chat() -> Result<WebResponse> {
    let web_context = use_web()?;
    let body = web_context.body_json()?;

    let model = body
        .get("model")
        .and_then(Value::as_str)
        .ok_or_else(|| WebError::parameter("缺少 model 参数", "model"))?
        .to_string();

    let request = ChatRequest {
        model,
        stream: body.get("stream").and_then(Value::as_bool).unwrap_or(false),
        session_id: session_id(body),
    };

    OpenaiService::chat(request).await
}

/// 会话标识：优先取 session_id / sessionId，其次取 user。
fn session_id(body: &Value) -> Option<String> {
    ["session_id", "sessionId", "user"]
        .iter()
        .find_map(|name| body.get(name).and_then(Value::as_str).map(str::to_string))
}

/// 模型列表接口
#[web_api_post(path = "/models")]
pub async fn models() -> Result<WebResponse> {
    OpenaiService::models().await
}
