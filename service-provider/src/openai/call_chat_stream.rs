//! 流式转发：构造 [OI] 请求并执行。

use std::sync::Arc;

use anyhow::Result;
use lib_provider::{ChatRequest, ForwardCallback};
use lib_provider_openai::OpenaiChatStreamRequest;
use lib_web_core::{WebResponse, use_web};
use types_admin::entity::Provider;

/// 流式转发：按分块持续把供应商返回转发给客户端。
pub async fn call_chat_stream(
    provider: Provider,
    _request: ChatRequest,
    callback: Arc<dyn ForwardCallback>,
) -> Result<WebResponse> {
    let web_context = use_web()?;
    OpenaiChatStreamRequest::new(provider, web_context, callback)
        .call()
        .await
}
