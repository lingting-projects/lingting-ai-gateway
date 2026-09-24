//! 流式转发：把供应商的 SSE 分片原样转发给客户端，同时解析累积用量。

use std::sync::Arc;

use anyhow::Result;
use lib_provider::client::ForwardRequest;
use lib_provider::{ChunkSink, ForwardCallback, ForwardOutcome};
use lib_web_core::{WebBody, WebContext, WebResponse};
use types_admin::entity::Provider;

use crate::chat_response::ChatResponse;
use crate::stream::{StreamParser, forward_stream};
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
            let outcome = utils::chat_outcome(status.as_u16(), &body);
            return utils::finish_response(&self.callback, status, headers, body, outcome).await;
        }

        let (sink, stream) = ChunkSink::channel();
        let callback = Arc::clone(&self.callback);
        let parser = ChatStreamParser::new();
        tokio::spawn(forward_stream(
            response,
            sink,
            callback,
            parser,
            headers.clone(),
        ));

        Ok(WebResponse {
            status: status.as_u16(),
            headers,
            body: WebBody::Stream(stream),
        })
    }
}

/// SSE 分片解析器：按数据行累积响应，终态时把累积结果序列化为日志用的返回内容。
struct ChatStreamParser {
    /// 尚未构成完整行的残余字节。
    pending: Vec<u8>,
    /// 累积后的响应。
    response: ChatResponse,
}

impl ChatStreamParser {
    fn new() -> Self {
        Self {
            pending: Vec::new(),
            response: ChatResponse::default(),
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
}

impl StreamParser for ChatStreamParser {
    /// 追加一段原始分片，解析其中已完整的数据行。
    fn push(&mut self, chunk: &[u8]) {
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

    /// 当前累积结果；仅在终态序列化返回内容，供失败时落库。
    fn outcome(&mut self, http_status: u16, terminal: bool) -> ForwardOutcome {
        ForwardOutcome {
            token_info: self.response.token_info(),
            content: terminal
                .then(|| utils::json_content(&self.response))
                .flatten(),
            return_model: self.response.model.clone(),
            finish_reason: self.response.finish_reason(),
            provider_request_id: self.response.id.clone(),
            http_status: Some(i32::from(http_status)),
            response_headers: None,
        }
    }
}
