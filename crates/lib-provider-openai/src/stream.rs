//! 流式转发驱动：消费供应商分片并原样写出，同时交给协议解析器累积转发结果。

use std::sync::Arc;

use framework_core::MultiStringValue;
use futures_util::StreamExt;
use lib_provider::{ChunkSink, ForwardCallback, ForwardFailure, ForwardOutcome};
use reqwest::Response;

use crate::utils;

/// 流式分片解析器：按分片累积转发结果，协议差异由各协议自行实现。
pub trait StreamParser {
    /// 追加一段原始分片，解析其中已完整的部分。
    fn push(&mut self, chunk: &[u8]);

    /// 处理流结束时残留的、没有换行符的最后一行。
    fn finish(&mut self);

    /// 取当前累积的转发结果。
    ///
    /// `terminal` 为真表示这是本次转发的最后一次取值，只有此时才序列化返回内容，
    /// 避免每个分片都做一次全量序列化。
    fn outcome(&mut self, http_status: u16, terminal: bool) -> ForwardOutcome;
}

/// 消费供应商分片：原样写出、解析累积，并按进度通知回调。
pub async fn forward_stream<P>(
    response: Response,
    sink: ChunkSink,
    callback: Arc<dyn ForwardCallback>,
    mut parser: P,
    response_headers: MultiStringValue,
) where
    P: StreamParser,
{
    let status = response.status().as_u16();
    let debug_mode = callback.is_debug();
    let mut stream = response.bytes_stream();

    loop {
        // 客户端断开与分片到达同时等待：断开时立即取消，
        // 不再等到下一个分片才检查，避免上游连接长时间滞留。
        let item = tokio::select! {
            _ = sink.closed() => {
                let outcome = take_outcome(&mut parser, status, &response_headers, debug_mode, true);
                utils::notify(callback.on_cancel(&outcome).await);
                return;
            }
            item = stream.next() => item,
        };

        let Some(item) = item else {
            break;
        };
        let chunk = match item {
            Ok(chunk) => chunk,
            Err(error) => {
                let failure = ForwardFailure::new(
                    utils::transport_error_type(&error),
                    error.to_string(),
                    take_outcome(&mut parser, status, &response_headers, debug_mode, true),
                );
                utils::notify(callback.on_failure(&failure).await);
                sink.send_error(error.to_string());
                return;
            }
        };

        parser.push(&chunk);
        sink.send(chunk);
        let outcome = take_outcome(&mut parser, status, &response_headers, debug_mode, false);
        utils::notify(callback.on_progress(&outcome).await);
    }

    parser.finish();
    let outcome = take_outcome(&mut parser, status, &response_headers, debug_mode, true);
    utils::notify(callback.on_success(&outcome).await);
}

/// 取当前累积结果，并按调试模式并入供应商响应头。
fn take_outcome<P>(
    parser: &mut P,
    status: u16,
    response_headers: &MultiStringValue,
    debug_mode: bool,
    terminal: bool,
) -> ForwardOutcome
where
    P: StreamParser,
{
    let mut outcome = parser.outcome(status, terminal);
    utils::record_response_headers(&mut outcome, response_headers, debug_mode);
    outcome
}
