//! [OI] 兼容接口声明，业务实现见 `service_provider::openai::OpenaiService`。

use anyhow::Result;
use framework_web::{WebError, WebResponse, use_web, web_api_post};
use lib_provider::ChatRequest;
use service_provider::openai::OpenaiService;

/// 聊天接口

#[web_api_post(path = "/chat")]
pub async fn chat() -> Result<WebResponse> {
    let web_context = use_web()?;
    let request = serde_json::from_value::<ChatRequest>(web_context.body_json()?.clone())
        .map_err(|error| WebError::parameter("请求参数解析失败", error))?;

    OpenaiService::chat(request).await
}

/// 模型列表接口

#[web_api_post(path = "/models")]
pub async fn models() -> Result<WebResponse> {
    OpenaiService::models().await
}
