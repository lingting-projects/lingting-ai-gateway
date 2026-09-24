//! Responses 非流式转发：把客户端请求原样发给供应商，再把供应商响应原样返回。

use std::sync::Arc;

use anyhow::Result;
use lib_provider::ForwardCallback;
use lib_provider::client::ForwardRequest;
use lib_web_core::{WebContext, WebResponse};
use types_admin::entity::Provider;

use crate::utils;

/// Responses 非流式转发请求。
pub struct OpenaiResponsesRequest {
    provider: Provider,
    web_context: Arc<WebContext>,
    callback: Arc<dyn ForwardCallback>,
}

impl OpenaiResponsesRequest {
    /// 绑定供应商、当前请求上下文与转发回调。
    pub fn new(
        provider: Provider,
        web_context: Arc<WebContext>,
        callback: Arc<dyn ForwardCallback>,
    ) -> Self {
        Self {
            provider,
            web_context,
            callback,
        }
    }

    /// 发起请求并把供应商响应原样返回。
    pub async fn call(&self) -> Result<WebResponse> {
        let request = ForwardRequest::new(&self.provider, utils::RESPONSES_SUFFIX)
            .forward(&self.web_context)?;

        utils::notify(self.callback.on_start().await);

        let response = request.call().await;
        let status = response.status();
        let headers = utils::response_headers(response.headers());
        let body = match response.bytes().await {
            Ok(body) => body,
            Err(error) => return utils::fail_transport(&self.callback, error).await,
        };

        let outcome = utils::responses_outcome(status.as_u16(), &body);
        utils::finish_response(&self.callback, status, headers, body, outcome).await
    }
}
