use crate::token::parse_usage;
use axum::{
    body::Body,
    http::{StatusCode, header},
    response::Response,
};
use futures_util::StreamExt;
use lib_core::{Id, RequestCompletion, RequestFailure, RequestStatus, TokenUsage, current_millis};
use lib_store::Store;
use serde_json::Value;
use std::convert::Infallible;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

const MAX_METADATA_LINE_BYTES: usize = 1024 * 1024;

pub(super) fn response(
    store: Store,
    request_id: Id,
    upstream: lib_provider::ChatStream,
    started_at: i64,
) -> Response {
    let (sender, receiver) = mpsc::channel::<Result<bytes::Bytes, Infallible>>(16);
    tokio::spawn(process_stream(
        store, request_id, upstream, sender, started_at,
    ));

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from_stream(ReceiverStream::new(receiver)))
        .expect("valid SSE response")
}

async fn process_stream(
    store: Store,
    request_id: Id,
    mut upstream: lib_provider::ChatStream,
    sender: mpsc::Sender<Result<bytes::Bytes, Infallible>>,
    started_at: i64,
) {
    let mut buffer = Vec::new();
    let mut usage = TokenUsage::default();
    let mut response_model = None;

    while let Some(chunk) = upstream.next().await {
        let chunk = match chunk {
            Ok(chunk) => chunk,
            Err(error_value) => {
                fail_request(
                    &store,
                    request_id,
                    RequestStatus::Failed,
                    response_model,
                    None,
                    usage,
                    started_at,
                    "provider_error",
                    "upstream_error",
                    error_value.to_string(),
                )
                .await;
                return;
            }
        };
        buffer.extend_from_slice(&chunk);
        parse_buffer(&mut buffer, &mut response_model, &mut usage);
        if buffer.len() > MAX_METADATA_LINE_BYTES {
            buffer.clear();
        }
        if sender.send(Ok(chunk)).await.is_err() {
            fail_request(
                &store,
                request_id,
                RequestStatus::ClientDisconnected,
                response_model,
                Some(200),
                usage,
                started_at,
                "client",
                "client_disconnected",
                "client disconnected".into(),
            )
            .await;
            return;
        }
    }

    parse_line(&buffer, &mut response_model, &mut usage);
    let completed_at = current_millis().unwrap_or(started_at);
    let completion = RequestCompletion {
        completed_at,
        response_model,
        status_code: Some(200),
        usage,
        latency_ms: completed_at.saturating_sub(started_at),
    };
    if let Err(error_value) = store.complete_request(request_id, &completion).await {
        tracing::error!(error = %error_value, request_id = %request_id, "更新完成请求日志失败");
    }
}

#[allow(clippy::too_many_arguments)]
async fn fail_request(
    store: &Store,
    request_id: Id,
    status: RequestStatus,
    response_model: Option<String>,
    status_code: Option<i32>,
    usage: TokenUsage,
    started_at: i64,
    error_type: &str,
    error_code: &str,
    error_message: String,
) {
    let completed_at = current_millis().unwrap_or(started_at);
    let failure = RequestFailure {
        completed_at,
        response_model,
        status_code,
        usage,
        latency_ms: completed_at.saturating_sub(started_at),
        error_type: error_type.into(),
        error_code: error_code.into(),
        error_message,
    };
    if let Err(error_value) = store.fail_request(request_id, status, &failure).await {
        tracing::error!(error = %error_value, request_id = %request_id, "更新失败请求日志失败");
    }
}

fn parse_buffer(buffer: &mut Vec<u8>, response_model: &mut Option<String>, usage: &mut TokenUsage) {
    while let Some(position) = buffer.iter().position(|byte| *byte == b'\n') {
        let line = buffer.drain(..=position).collect::<Vec<_>>();
        parse_line(&line, response_model, usage);
    }
}

fn parse_line(line: &[u8], response_model: &mut Option<String>, usage: &mut TokenUsage) {
    let Ok(line) = std::str::from_utf8(line) else {
        return;
    };
    let line = line.trim();
    let Some(data) = line.strip_prefix("data:").map(str::trim_start) else {
        return;
    };
    if data == "[DONE]" {
        return;
    }
    if let Ok(value) = serde_json::from_str::<Value>(data) {
        if let Some(model) = value.get("model").and_then(Value::as_str) {
            *response_model = Some(model.to_owned())
        }
        if value
            .get("usage")
            .is_some_and(|usage_value| !usage_value.is_null())
        {
            *usage = parse_usage(value.get("usage"));
        }
    }
}
