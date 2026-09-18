//! [OI] 协议的转发实现。

pub mod chat;
pub mod chat_stream;
pub mod utils;

pub use chat::OpenaiChatRequest;
pub use chat_stream::OpenaiChatStreamRequest;
