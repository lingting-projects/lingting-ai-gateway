//! 聊天接口：取供应商后按是否流式分派到具体转发。

use anyhow::Result;

use lib_provider::RouteRequest;
use lib_provider_openai::utils::chat_url;
use lib_web_core::WebResponse;

use super::OpenaiService;
use super::call_chat::call_chat;
use super::call_chat_stream::call_chat_stream;
use super::forward::{first_provider, with_wrapper};

impl OpenaiService {
    /// 聊天接口：按模型取优先级最高的供应商，普通请求走 `call_chat`，
    /// 流式请求走 `call_chat_stream`。
    pub async fn chat(request: RouteRequest) -> Result<WebResponse> {
        let provider = first_provider(&request.model).await?;
        let provider_url = chat_url(&provider);

        if request.stream {
            with_wrapper(provider, request, provider_url, call_chat_stream).await
        } else {
            with_wrapper(provider, request, provider_url, call_chat).await
        }
    }
}
