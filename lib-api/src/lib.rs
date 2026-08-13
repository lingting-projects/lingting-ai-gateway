use axum::{
    Json, Router,
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use futures_util::StreamExt;
use lib_core::{
    Id, NewRequestLog, RequestCompletion, RequestFailure, RequestStatus, RequestType, TokenUsage,
    current_millis,
};
use lib_provider::OpenAICompatibleProvider;
use lib_store::Store;
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{convert::Infallible, sync::Arc};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

const MAX_SSE_METADATA_LINE_BYTES: usize = 1024 * 1024;
#[derive(Clone)]
pub struct AppState {
    store: Store,
}
#[derive(Serialize)]
struct ErrorResponse {
    error: ErrorBody,
}
#[derive(Serialize)]
struct ErrorBody {
    message: String,
    r#type: String,
    code: String,
}

pub fn router(store: Store) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/v1/models", get(models))
        .route("/v1/chat/completions", post(chat))
        .with_state(Arc::new(AppState { store }))
}

async fn authenticate(state: &AppState, headers: &HeaderMap) -> Result<Id, Response> {
    if state.store.anonymous_access().await.map_err(internal)? {
        return Ok(Id::zero());
    }
    let key = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(unauthorized)?;
    let hash = hex::encode(Sha256::digest(key.as_bytes()));
    state
        .store
        .authenticate_hash(&hash)
        .await
        .map_err(internal)?
        .ok_or_else(unauthorized)
}

async fn models(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    if let Err(response) = authenticate(&state, &headers).await {
        return response;
    }
    match state.store.default_models().await {
        Ok(Some(models)) => Json(json!({"object":"list","data":models.into_iter().map(|id| json!({"id":id,"object":"model"})).collect::<Vec<_>>() })).into_response(),
        Ok(None) => error(StatusCode::INTERNAL_SERVER_ERROR, "default_provider_not_available"),
        Err(error_value) => internal(error_value),
    }
}

async fn chat(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    let api_key_id = match authenticate(&state, &headers).await {
        Ok(id) => id,
        Err(response) => return response,
    };
    let client_model = match body.get("model").and_then(Value::as_str) {
        Some(model) if !model.is_empty() => model.to_owned(),
        _ => return error(StatusCode::BAD_REQUEST, "model_not_available"),
    };
    let resolved = match state.store.resolve_model(&client_model).await {
        Ok(Some(resolved)) => resolved,
        Ok(None) => {
            return match state.store.default_provider_available().await {
                Ok(true) => error(StatusCode::BAD_REQUEST, "model_not_available"),
                Ok(false) => error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "default_provider_not_available",
                ),
                Err(error_value) => internal(error_value),
            };
        }
        Err(error_value) => return internal(error_value),
    };
    let stream = body.get("stream").and_then(Value::as_bool).unwrap_or(false);
    if stream {
        stream_chat(state, api_key_id, resolved, body).await
    } else {
        normal_chat(state, api_key_id, resolved, body).await
    }
}

async fn normal_chat(
    state: Arc<AppState>,
    api_key_id: Id,
    resolved: lib_core::ResolvedModel,
    body: Value,
) -> Response {
    let started_at = match current_millis() {
        Ok(value) => value,
        Err(error_value) => return internal(error_value),
    };
    let request_id = match Id::new() {
        Ok(id) => id,
        Err(error_value) => return internal(error_value),
    };
    let log = NewRequestLog {
        id: request_id,
        api_key_id,
        client_model: resolved.client_model.clone(),
        provider_id: resolved.provider.id,
        provider_model: resolved.provider_model.upstream_model.clone(),
        request_type: RequestType::Normal,
        status: RequestStatus::Connecting,
        started_at,
    };
    if let Err(error_value) = state.store.create_request(&log).await {
        return internal(error_value);
    }
    let provider = OpenAICompatibleProvider::new(
        resolved.provider.id,
        resolved.provider.base_url,
        resolved.provider.api_key,
    );
    let result = provider
        .chat(body, &resolved.provider_model.upstream_model)
        .await;
    match result {
        Ok(response) => {
            let completed_at = current_millis().unwrap_or(started_at);
            let completion = RequestCompletion {
                completed_at,
                response_model: response
                    .get("model")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                status_code: Some(200),
                usage: parse_usage(response.get("usage")),
                latency_ms: completed_at.saturating_sub(started_at),
            };
            if let Err(error_value) = state.store.complete_request(request_id, &completion).await {
                tracing::error!(error = %error_value, request_id = %request_id, "更新完成请求日志失败");
            }
            Json(response).into_response()
        }
        Err(provider_error) => {
            let status_code = provider_status(&provider_error);
            let completed_at = current_millis().unwrap_or(started_at);
            let failure = RequestFailure {
                completed_at,
                response_model: None,
                status_code,
                usage: TokenUsage::default(),
                latency_ms: completed_at.saturating_sub(started_at),
                error_type: "provider_error".into(),
                error_code: "upstream_error".into(),
                error_message: provider_error.to_string(),
            };
            if let Err(error_value) = state
                .store
                .fail_request(request_id, RequestStatus::Failed, &failure)
                .await
            {
                tracing::error!(error = %error_value, request_id = %request_id, "更新失败请求日志失败")
            }
            error(StatusCode::BAD_GATEWAY, "upstream_error")
        }
    }
}

async fn stream_chat(
    state: Arc<AppState>,
    api_key_id: Id,
    resolved: lib_core::ResolvedModel,
    body: Value,
) -> Response {
    let started_at = match current_millis() {
        Ok(value) => value,
        Err(error_value) => return internal(error_value),
    };
    let request_id = match Id::new() {
        Ok(id) => id,
        Err(error_value) => return internal(error_value),
    };
    let log = NewRequestLog {
        id: request_id,
        api_key_id,
        client_model: resolved.client_model.clone(),
        provider_id: resolved.provider.id,
        provider_model: resolved.provider_model.upstream_model.clone(),
        request_type: RequestType::Stream,
        status: RequestStatus::Connecting,
        started_at,
    };
    if let Err(error_value) = state.store.create_request(&log).await {
        return internal(error_value);
    }
    let provider = OpenAICompatibleProvider::new(
        resolved.provider.id,
        resolved.provider.base_url,
        resolved.provider.api_key,
    );
    let upstream = provider
        .chat_stream(body, &resolved.provider_model.upstream_model)
        .await;
    let upstream = match upstream {
        Ok(stream) => stream,
        Err(provider_error) => {
            let completed_at = current_millis().unwrap_or(started_at);
            let failure = RequestFailure {
                completed_at,
                response_model: None,
                status_code: provider_status(&provider_error),
                usage: TokenUsage::default(),
                latency_ms: completed_at.saturating_sub(started_at),
                error_type: "provider_error".into(),
                error_code: "upstream_error".into(),
                error_message: provider_error.to_string(),
            };
            if let Err(error_value) = state
                .store
                .fail_request(request_id, RequestStatus::Failed, &failure)
                .await
            {
                tracing::error!(error = %error_value, request_id = %request_id, "更新失败请求日志失败");
            }
            return error(StatusCode::BAD_GATEWAY, "upstream_error");
        }
    };
    let (sender, receiver) = mpsc::channel::<Result<bytes::Bytes, Infallible>>(16);
    tokio::spawn(process_stream(
        state.store.clone(),
        request_id,
        upstream,
        sender,
        started_at,
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
                let completed_at = current_millis().unwrap_or(started_at);
                let failure = RequestFailure {
                    completed_at,
                    response_model,
                    status_code: None,
                    usage,
                    latency_ms: completed_at.saturating_sub(started_at),
                    error_type: "provider_error".into(),
                    error_code: "upstream_error".into(),
                    error_message: error_value.to_string(),
                };
                if let Err(store_error) = store
                    .fail_request(request_id, RequestStatus::Failed, &failure)
                    .await
                {
                    tracing::error!(error = %store_error, request_id = %request_id, "更新失败请求日志失败");
                }
                return;
            }
        };
        buffer.extend_from_slice(&chunk);
        parse_sse_buffer(&mut buffer, &mut response_model, &mut usage);
        if buffer.len() > MAX_SSE_METADATA_LINE_BYTES {
            buffer.clear();
        }
        if sender.send(Ok(chunk)).await.is_err() {
            let completed_at = current_millis().unwrap_or(started_at);
            let failure = RequestFailure {
                completed_at,
                response_model,
                status_code: Some(200),
                usage,
                latency_ms: completed_at.saturating_sub(started_at),
                error_type: "client".into(),
                error_code: "client_disconnected".into(),
                error_message: "client disconnected".into(),
            };
            if let Err(store_error) = store
                .fail_request(request_id, RequestStatus::ClientDisconnected, &failure)
                .await
            {
                tracing::error!(error = %store_error, request_id = %request_id, "更新客户端断开日志失败");
            }
            return;
        }
    }
    parse_sse_line(&buffer, &mut response_model, &mut usage);
    let completed_at = current_millis().unwrap_or(started_at);
    let completion = RequestCompletion {
        completed_at,
        response_model,
        status_code: Some(200),
        usage,
        latency_ms: completed_at.saturating_sub(started_at),
    };
    if let Err(store_error) = store.complete_request(request_id, &completion).await {
        tracing::error!(error = %store_error, request_id = %request_id, "更新完成请求日志失败");
    }
}

fn parse_sse_buffer(
    buffer: &mut Vec<u8>,
    response_model: &mut Option<String>,
    usage: &mut TokenUsage,
) {
    while let Some(position) = buffer.iter().position(|byte| *byte == b'\n') {
        let line = buffer.drain(..=position).collect::<Vec<_>>();
        parse_sse_line(&line, response_model, usage);
    }
}

fn parse_sse_line(line: &[u8], response_model: &mut Option<String>, usage: &mut TokenUsage) {
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

fn parse_usage(value: Option<&Value>) -> TokenUsage {
    let get = |path: &[&str]| {
        path.iter()
            .try_fold(value?, |current, key| current.get(*key))
            .and_then(Value::as_i64)
    };
    TokenUsage {
        input_tokens: get(&["prompt_tokens"]),
        output_tokens: get(&["completion_tokens"]),
        total_tokens: get(&["total_tokens"]),
        cache_read_input_tokens: get(&["prompt_tokens_details", "cached_tokens"]),
        cache_write_input_tokens: get(&["prompt_tokens_details", "cache_write_tokens"]),
        reasoning_tokens: get(&["completion_tokens_details", "reasoning_tokens"]),
        input_audio_tokens: get(&["prompt_tokens_details", "audio_tokens"]),
        output_audio_tokens: get(&["completion_tokens_details", "audio_tokens"]),
    }
}
fn provider_status(error_value: &lib_core::ProviderError) -> Option<i32> {
    match error_value {
        lib_core::ProviderError::Status { status } => Some(i32::from(*status)),
        _ => None,
    }
}
fn unauthorized() -> Response {
    error(StatusCode::UNAUTHORIZED, "unauthorized")
}
fn internal(error_value: impl std::fmt::Display) -> Response {
    tracing::error!(error = %error_value, "gateway internal error");
    error(StatusCode::INTERNAL_SERVER_ERROR, "internal_error")
}
fn error(status: StatusCode, code: &str) -> Response {
    (
        status,
        Json(ErrorResponse {
            error: ErrorBody {
                message: code.into(),
                r#type: "gateway_error".into(),
                code: code.into(),
            },
        }),
    )
        .into_response()
}
