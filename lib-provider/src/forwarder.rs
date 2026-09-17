use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use framework_core::current_millis;
use futures_util::{Stream, StreamExt};
use types_admin::TokenInfo;

use crate::callbacks::{ForwardResult, ProviderCallbacks};
use crate::client::ChatStream;
use crate::error::ProviderError;
use crate::response::{ChatResponse, ChatStreamChunk};

/// 转发过程中的累计数据。
#[derive(Debug, Default)]
struct ForwardState {
    model: String,
    usage: TokenInfo,
    content: String,
    http_status: Option<u16>,
    finish_reason: Option<String>,
    provider_request_id: Option<String>,
    chunk_count: u64,
    end_time: Option<i64>,
}

/// 返回值转发器：记录每次转发的 token、内容与时间。
///
/// 请求失败时内部已累计的数据依然保留，供失败回调写入请求日志。
pub struct ResponseForwarder {
    debug: bool,
    start_time: i64,
    state: Mutex<ForwardState>,
}

impl ResponseForwarder {
    pub fn new(debug: bool, start_time: i64) -> Self {
        Self {
            debug,
            start_time,
            state: Mutex::new(ForwardState::default()),
        }
    }

    /// 记录非流式响应。
    pub fn record_response(&self, response: &ChatResponse) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(model) = response.model.as_ref() {
            state.model = model.clone();
        }
        if let Some(usage) = response.usage.as_ref() {
            state.usage = usage.to_token_info();
        }
        if let Some(id) = response.id.as_ref() {
            state.provider_request_id = Some(id.clone());
        }
        if let Some(finish_reason) = response.finish_reason() {
            state.finish_reason = Some(finish_reason);
        }
        if self.debug {
            state.content.push_str(&response.content());
        }
    }

    /// 记录流式分块。
    pub fn record_chunk(&self, chunk: &ChatStreamChunk) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.chunk_count += 1;
        if let Some(model) = chunk.model.as_ref() {
            state.model = model.clone();
        }
        if let Some(id) = chunk.id.as_ref() {
            state.provider_request_id = Some(id.clone());
        }
        if let Some(usage) = chunk.usage.as_ref() {
            state.usage = usage.to_token_info();
        }
        if let Some(finish_reason) = chunk.finish_reason() {
            state.finish_reason = Some(finish_reason);
        }
        if self.debug {
            state.content.push_str(&chunk.content());
        }
    }

    /// 记录供应商返回的 HTTP 状态码。
    pub fn record_http_status(&self, status: u16) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.http_status = Some(status);
    }

    /// 结束转发并取出结果，重复调用只结算一次。
    pub fn finish(&self) -> ForwardResult {
        let end_time = now_millis();
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let end_time = *state.end_time.get_or_insert(end_time);
        ForwardResult {
            model: state.model.clone(),
            usage: state.usage.clone(),
            content: self.debug.then(|| state.content.clone()),
            http_status: state.http_status,
            finish_reason: state.finish_reason.clone(),
            provider_request_id: state.provider_request_id.clone(),
            chunk_count: state.chunk_count,
            start_time: self.start_time,
            end_time,
            duration_ms: (end_time - self.start_time).max(0),
        }
    }
}

/// 包装流式响应：转发分块的同时累计数据，并在流结束时触发回调。
pub struct ForwardingStream {
    inner: ChatStream,
    forwarder: Arc<ResponseForwarder>,
    callbacks: Arc<dyn ProviderCallbacks>,
    finished: bool,
}

impl ForwardingStream {
    pub fn new(
        inner: ChatStream,
        forwarder: Arc<ResponseForwarder>,
        callbacks: Arc<dyn ProviderCallbacks>,
    ) -> Self {
        Self {
            inner,
            forwarder,
            callbacks,
            finished: false,
        }
    }

    /// 结束并触发成功回调。
    fn finish_success(&mut self) {
        self.finished = true;
        let result = self.forwarder.finish();
        let callbacks = self.callbacks.clone();
        spawn_callback(async move { callbacks.on_success(result).await });
    }

    /// 结束并触发失败回调。
    fn finish_failure(&mut self, error: ProviderError) {
        self.finished = true;
        let result = self.forwarder.finish();
        let callbacks = self.callbacks.clone();
        spawn_callback(async move { callbacks.on_failure(error, result).await });
    }
}

impl Stream for ForwardingStream {
    type Item = Result<ChatStreamChunk, ProviderError>;

    fn poll_next(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        match this.inner.poll_next_unpin(context) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => {
                this.finish_success();
                Poll::Ready(None)
            }
            Poll::Ready(Some(Ok(chunk))) => {
                this.forwarder.record_chunk(&chunk);
                Poll::Ready(Some(Ok(chunk)))
            }
            Poll::Ready(Some(Err(error))) => {
                this.finish_failure(error.clone());
                Poll::Ready(Some(Err(error)))
            }
        }
    }
}

impl Drop for ForwardingStream {
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        // 流未被消费完即被丢弃，视为客户端取消请求。
        self.finished = true;
        let result = self.forwarder.finish();
        let callbacks = self.callbacks.clone();
        spawn_callback(async move { callbacks.on_cancel(result).await });
    }
}

/// 在后台执行回调，回调异常不影响请求结果。
fn spawn_callback<F>(future: F)
where
    F: Future<Output=()> + Send + 'static,
{
    if tokio::runtime::Handle::try_current().is_err() {
        return;
    }
    tokio::spawn(future);
}

/// 当前毫秒时间戳，取不到时记录告警并返回 0。
fn now_millis() -> i64 {
    match current_millis() {
        Ok(millis) => millis,
        Err(error) => {
            tracing::warn!(%error, "获取当前时间失败");
            0
        }
    }
}
