//! 普通（非流式）转发，当前仅占位。

use std::sync::Arc;

use anyhow::{Result, anyhow};
use lib_provider::{ChatRequest, ForwardCallback};
use lib_web_core::WebResponse;
use types_admin::entity::Provider;

/// 普通转发：等待供应商返回完整响应后原样转发。
pub async fn call_chat(
    _provider: Provider,
    _request: ChatRequest,
    _callback: Arc<dyn ForwardCallback>,
) -> Result<WebResponse> {
    Err(anyhow!("普通转发尚未实现"))
}
