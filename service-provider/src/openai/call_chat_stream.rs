//! 流式转发，当前仅占位。

use std::sync::Arc;

use anyhow::{Result, anyhow};
use lib_provider::{ChatRequest, ForwardCallback};
use lib_web_core::WebResponse;
use types_admin::entity::Provider;

/// 流式转发：按分块持续把供应商返回转发给客户端。
pub async fn call_chat_stream(
    _provider: Provider,
    _request: ChatRequest,
    _callback: Arc<dyn ForwardCallback>,
) -> Result<WebResponse> {
    Err(anyhow!("流式转发尚未实现"))
}
