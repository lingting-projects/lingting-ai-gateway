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

    /// 取当前累积的转发结果；原始内容取走后继续累积后续分片。
    fn outcome(&mut self, http_status: u16) -> ForwardOutcome;
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

    while let Some(item) = stream.next().await {
        if sink.is_closed() {
            let outcome = take_outcome(&mut parser, status, &response_headers, debug_mode);
            utils::notify(callback.on_cancel(&outcome).await);
            return;
        }

        let chunk = match item {
            Ok(chunk) => chunk,
            Err(error) => {
                let failure = ForwardFailure::new(
                    utils::transport_error_type(&error),
                    error.to_string(),
                    take_outcome(&mut parser, status, &response_headers, debug_mode),
                );
                utils::notify(callback.on_failure(&failure).await);
                sink.send_error(error.to_string());
                return;
            }
        };

        parser.push(&chunk);
        sink.send(chunk);
        let outcome = take_outcome(&mut parser, status, &response_headers, debug_mode);
        utils::notify(callback.on_progress(&outcome).await);
    }

    parser.finish();
    let outcome = take_outcome(&mut parser, status, &response_headers, debug_mode);
    utils::notify(callback.on_success(&outcome).await);
}

/// 取当前累积结果，并按调试模式并入供应商响应头。
fn take_outcome<P>(
    parser: &mut P,
    status: u16,
    response_headers: &MultiStringValue,
    debug_mode: bool,
) -> ForwardOutcome
where
    P: StreamParser,
{
    let mut outcome = parser.outcome(status);
    utils::record_response_headers(&mut outcome, response_headers, debug_mode);
    outcome
}
