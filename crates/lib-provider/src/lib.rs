//! 供应商协议抽象层。
//!
//! 本 crate 只定义请求 / 响应模型、流式出口、转发回调与协议驱动 trait，
//! 不接触数据库也不写请求日志：用量与内容的落库由调用方通过
//! [`ForwardCallback`] 完成。

pub mod chunk;
pub mod client;
pub mod forward;
pub mod request;
pub mod response;

pub use chunk::ChunkSink;
pub use client::{ForwardRequest, build_client};
pub use forward::{ForwardCallback, ForwardFailure, ForwardOutcome};
pub use request::{ChatMessage, ChatRequest, MessageContent, MessageRole};
pub use response::{
    ChatChoice, ChatResponse, ChatUsage, CompletionTokensDetails, PromptTokensDetails,
    ProviderResponse, ResponseMessage,
};
