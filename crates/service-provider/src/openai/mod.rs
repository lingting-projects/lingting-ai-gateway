//! OpenAI 兼容接口：接口入口、模型列表与转发实现。

pub mod call_chat;
pub mod call_chat_stream;
pub mod call_response;
pub mod call_response_stream;
pub mod callback;
pub mod chat;
pub mod common;
pub mod forward;
pub mod models;
pub mod responses;
pub mod utils;

/// OpenAI 兼容接口业务入口。
pub struct OpenaiService;
