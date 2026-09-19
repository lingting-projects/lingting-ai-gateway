//! Responses 接口：取供应商后按是否流式分派到具体转发。

use anyhow::Result;

use lib_provider::RouteRequest;
use lib_provider_openai::utils::responses_url;
use lib_web_core::WebResponse;

use super::OpenaiService;
use super::call_response::call_response;
use super::call_response_stream::call_response_stream;
use super::forward::{first_provider, with_wrapper};

impl OpenaiService {
    /// Responses 接口：按模型取优先级最高的供应商，普通请求走 `call_response`，
    /// 流式请求走 `call_response_stream`。
    pub async fn responses(request: RouteRequest) -> Result<WebResponse> {
        let provider = first_provider(&request.model).await?;
        let provider_url = responses_url(&provider);

        if request.stream {
            with_wrapper(provider, request, provider_url, call_response_stream).await
        } else {
            with_wrapper(provider, request, provider_url, call_response).await
        }
    }
}
