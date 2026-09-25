//! [OI] 协议的转发实现。

pub mod chat;
pub mod chat_response;
pub mod chat_stream;
pub mod error_response;
pub mod models;
pub mod responses;
pub mod responses_response;
pub mod responses_stream;
pub mod stream;
pub mod utils;

pub use chat::OpenaiChatRequest;
pub use chat_stream::OpenaiChatStreamRequest;
pub use responses::OpenaiResponsesRequest;
pub use responses_stream::OpenaiResponsesStreamRequest;
