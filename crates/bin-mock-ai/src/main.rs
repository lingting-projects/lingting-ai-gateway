//! 可控慢速 mock 上游：验证网关在客户端取消时是否连带取消内部转发。
//!
//! 行为由请求头控制（网关会原样转发非逐跳头），也支持同名查询参数：
//!
//! | 请求头 / 查询参数 | 默认值 | 作用 |
//! |---|---|---|
//! | `x-mock-delay-ms` | 0 | 发送响应头前的延迟 |
//! | `x-mock-first-chunk-ms` | 0 | 首个分片前的延迟 |
//! | `x-mock-chunk-interval-ms` | 100 | 分片间隔 |
//! | `x-mock-chunks` | 3 | 分片数量 |
//! | `x-mock-hang` | false | 为真时发完分片后挂起，永不结束 |
//!
//! 关键日志以 `★★★` 标记，用于判定上游是否感知到客户端断开。

mod config;
mod disconnect;
mod handler;

use axum::Router;
use axum::routing::{get, post};

/// 监听地址。
const ADDRESS: &str = "127.0.0.1:26383";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let app = Router::new()
        .route("/v1/models", get(handler::models))
        .route("/v1/chat/completions", post(handler::chat_completions))
        .route("/v1/responses", post(handler::responses));

    let listener = tokio::net::TcpListener::bind(ADDRESS).await?;
    note(&format!("mock 上游已启动：http://{ADDRESS}"));

    axum::serve(listener, app).await?;
    Ok(())
}

/// 带时间戳输出，便于与客户端断开时刻对齐。
pub(crate) fn note(message: &str) {
    let now = chrono::Local::now().format("%H:%M:%S%.3f");
    println!("[{now}] {message}");
}
