//! mock 接口实现：模型列表、对话补全与 Responses。

use std::io;
use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::extract::{Query, Request};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use bytes::Bytes;
use serde_json::{Value, json};

use crate::config::{MockConfig, MockQuery};
use crate::disconnect::{DisconnectGuard, DisconnectStream};
use crate::note;

/// 模型列表返回的模型名；网关的模型同步会读取它，也是未指定模型时的兜底名。
const MODEL: &str = "mock-chat";

/// 模型列表；网关的供应商模型同步使用。
pub(crate) async fn models() -> Response {
    note("收到 /v1/models");
    json_response(json!({
        "object": "list",
        "data": [{ "id": MODEL, "object": "model" }],
    }))
}

/// 对话补全。
pub(crate) async fn chat_completions(
    headers: HeaderMap,
    Query(query): Query<MockQuery>,
    request: Request,
) -> Response {
    handle(headers, query, request, "chat/completions").await
}

/// Responses 接口。
pub(crate) async fn responses(
    headers: HeaderMap,
    Query(query): Query<MockQuery>,
    request: Request,
) -> Response {
    handle(headers, query, request, "responses").await
}

/// 两个接口共用：读取请求体、按配置延迟，再按流式 / 非流式返回。
async fn handle(headers: HeaderMap, query: MockQuery, request: Request, path: &str) -> Response {
    let config = MockConfig::resolve(&headers, &query);
    let body = axum::body::to_bytes(request.into_body(), usize::MAX)
        .await
        .unwrap_or_default();
    let body: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
    let stream = body.get("stream").and_then(Value::as_bool).unwrap_or(false);
    let model = body
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or(MODEL)
        .to_string();

    note(&format!("收到 /v1/{path} stream={stream} 配置={config:?}"));

    // 响应头前的延迟：期间 handler 若被 drop，说明客户端已断开。
    if config.delay_ms > 0 {
        let mut guard = DisconnectGuard::new("等待响应头");
        note(&format!("延迟 {}ms 后再发送响应头", config.delay_ms));
        tokio::time::sleep(Duration::from_millis(config.delay_ms)).await;
        guard.disarm();
    }

    if stream {
        sse_response(&config, &model)
    } else {
        note("返回非流式响应体");
        json_response(json!({
            "id": "mock-1",
            "object": "chat.completion",
            "model": model,
            "choices": [{
                "index": 0,
                "message": { "role": "assistant", "content": "ok" },
                "finish_reason": "stop",
            }],
            "usage": { "prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2 },
        }))
    }
}

/// 构造 SSE 响应；响应流被提前 drop 即说明客户端断开。
fn sse_response(config: &MockConfig, model: &str) -> Response {
    let frames = Arc::new(sse_frames(config, model));
    let hang = config.hang;
    let first_chunk_ms = config.first_chunk_ms;
    let chunk_interval_ms = config.chunk_interval_ms;

    let stream = futures_util::stream::unfold(0usize, move |index| {
        let frames = Arc::clone(&frames);
        async move {
            if index < frames.len() {
                let delay = if index == 0 {
                    first_chunk_ms
                } else {
                    chunk_interval_ms
                };
                if delay > 0 {
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                }
                note(&format!("发送分片 {}/{}", index + 1, frames.len()));
                Some((Ok::<Bytes, io::Error>(frames[index].clone()), index + 1))
            } else if hang {
                note("分片已发完，进入挂起：不再发送且不结束");
                std::future::pending::<()>().await;
                unreachable!()
            } else {
                None
            }
        }
    });

    let body = Body::from_stream(DisconnectStream::new(stream));
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/event-stream")
        .header("cache-control", "no-cache")
        .body(body)
        .unwrap_or_default()
}

/// SSE 分片；挂起模式不发结束标记。
fn sse_frames(config: &MockConfig, model: &str) -> Vec<Bytes> {
    let mut frames = Vec::with_capacity(config.chunks + 1);
    for index in 0..config.chunks {
        let chunk = json!({
            "id": "mock-1",
            "object": "chat.completion.chunk",
            "model": model,
            "choices": [{ "index": 0, "delta": { "content": format!("chunk-{index}") } }],
        });
        frames.push(Bytes::from(format!("data: {chunk}\n\n")));
    }
    if !config.hang {
        frames.push(Bytes::from("data: [DONE]\n\n"));
    }
    frames
}

/// 构造 JSON 响应。
fn json_response(value: Value) -> Response {
    let body = serde_json::to_vec(&value).unwrap_or_default();
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json; charset=utf-8")
        .body(Body::from(body))
        .unwrap_or_default()
}
