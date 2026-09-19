//! Responses 普通（非流式）转发：构造 OpenAI 请求并执行。

use std::sync::Arc;

use anyhow::Result;
use lib_provider::{ForwardCallback, RouteRequest};
use lib_provider_openai::OpenaiResponsesRequest;
use lib_web_core::{WebResponse, use_web};
use types_admin::entity::Provider;

/// 普通转发：等待供应商返回完整响应后原样转发。
pub async fn call_response(
    provider: Provider,
    _request: RouteRequest,
    callback: Arc<dyn ForwardCallback>,
) -> Result<WebResponse> {
    let web_context = use_web()?;
    OpenaiResponsesRequest::new(provider, web_context, callback)
        .call()
        .await
}
