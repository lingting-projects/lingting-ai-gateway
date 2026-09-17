//! [OI] 协议实现：Chat Completions 与模型列表。

mod driver;
mod sse;
mod client;

pub use driver::OpenAiDriver;
