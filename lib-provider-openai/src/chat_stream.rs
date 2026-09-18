//! 流式转发：把供应商的 SSE 分片原样转发给客户端，同时解析累积用量。

use std::sync::Arc;

use anyhow::Result;
use bytes::{Bytes, BytesMut};
use futures_util::StreamExt;
use lib_provider::client::ForwardRequest;
use lib_provider::response::ChatResponse;
use lib_provider::{ChunkSink, ForwardCallback, ForwardFailure, ForwardOutcome};
use lib_web_core::{WebBody, WebContext, WebResponse};
use reqwest::Response;
use types_admin::entity::Provider;

use crate::utils;

/// [OI] 流式转发请求。
pub struct OpenaiChatStreamRequest {
    provider: Provider,
    web_context: Arc<WebContext>,
    callback: Arc<dyn ForwardCallback>,
}

impl OpenaiChatStreamRequest {
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

    /// 发起请求：正常时返回持续输出的分片流，供应商直接报错时原样返回错误响应。
    pub async fn call(&self) -> Result<WebResponse> {
        let request = ForwardRequest::new(&self.provider, utils::CHAT_COMPLETIONS_SUFFIX)
            .forward(&self.web_context)?;

        utils::notify(self.callback.on_start().await);

        let response = request.call().await;
        let status = response.status();
        let headers = utils::response_headers(response.headers());
        if !status.is_success() {
            let body = match response.bytes().await {
                Ok(body) => body,
                Err(error) => return utils::fail_transport(&self.callback, error).await,
            };
            return utils::finish_response(&self.callback, status, headers, body).await;
        }

        let (sink, stream) = ChunkSink::channel();
        let callback = Arc::clone(&self.callback);
        let parser = StreamParser::new(callback.is_debug());
        tokio::spawn(forward_stream(response, sink, callback, parser));

        Ok(WebResponse {
            status: status.as_u16(),
            headers,
            body: WebBody::Stream(stream),
        })
    }
}

/// 消费供应商分片：原样写出、解析累积，并按进度通知回调。
async fn forward_stream(
    response: Response,
    sink: ChunkSink,
    callback: Arc<dyn ForwardCallback>,
    mut parser: StreamParser,
) {
    let status = response.status().as_u16();
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        if sink.is_closed() {
            utils::notify(callback.on_cancel(&parser.outcome(status)).await);
            return;
        }

        let chunk = match item {
            Ok(chunk) => chunk,
            Err(error) => {
                let failure = ForwardFailure::new(
                    utils::transport_error_type(&error),
                    error.to_string(),
                    parser.outcome(status),
                );
                utils::notify(callback.on_failure(&failure).await);
                sink.send_error(error.to_string());
                return;
            }
        };

        parser.push(&chunk);
        sink.send(chunk);
        utils::notify(callback.on_progress(&parser.outcome(status)).await);
    }

    parser.finish();
    utils::notify(callback.on_success(&parser.outcome(status)).await);
}

/// SSE 分片解析器：按数据行累积响应，并按调试模式保留原始内容。
struct StreamParser {
    /// 尚未构成完整行的残余字节。
    pending: Vec<u8>,
    /// 累积后的响应。
    response: ChatResponse,
    /// 原始内容，仅调试模式下收集。
    content: BytesMut,
    /// 是否处于调试模式。
    debug_mode: bool,
}

impl StreamParser {
    /// 按调试模式创建解析器。
    fn new(debug_mode: bool) -> Self {
        Self {
            pending: Vec::new(),
            response: ChatResponse::default(),
            content: BytesMut::new(),
            debug_mode,
        }
    }

    /// 追加一段原始分片，解析其中已完整的数据行。
    fn push(&mut self, chunk: &[u8]) {
        if self.debug_mode {
            self.content.extend_from_slice(chunk);
        }
        self.pending.extend_from_slice(chunk);

        while let Some(position) = self.pending.iter().position(|byte| *byte == b'\n') {
            let line: Vec<u8> = self.pending.drain(..=position).collect();
            self.merge_line(&line);
        }
    }

    /// 处理流结束时残留的、没有换行符的最后一行。
    fn finish(&mut self) {
        let line = std::mem::take(&mut self.pending);
        self.merge_line(&line);
    }

    /// 当前累积结果；原始内容取走后继续累积后续分片。
    fn outcome(&mut self, http_status: u16) -> ForwardOutcome {
        ForwardOutcome {
            token_info: self.response.token_info(),
            content: self.take_content(),
            return_model: self.response.model.clone(),
            finish_reason: self.response.finish_reason(),
            provider_request_id: self.response.id.clone(),
            http_status: Some(i32::from(http_status)),
        }
    }

    /// 合并一行 SSE 数据；`[DONE]` 与其他行直接忽略。
    fn merge_line(&mut self, line: &[u8]) {
        let line = String::from_utf8_lossy(line);
        let Some(data) = line.trim_end().strip_prefix("data:") else {
            return;
        };
        let Ok(chunk) = serde_json::from_str::<ChatResponse>(data.trim()) else {
            return;
        };
        self.response.merge_chunk(&chunk);
    }

    /// 取走已收集的原始内容。
    fn take_content(&mut self) -> Option<Bytes> {
        if self.content.is_empty() {
            return None;
        }
        Some(self.content.split().freeze())
    }
}
