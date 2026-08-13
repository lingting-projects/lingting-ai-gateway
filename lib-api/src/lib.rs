mod request_log;
mod sse;
pub mod token;

use crate::token::parse_usage;
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use lib_core::{
    Id, RequestCompletion, RequestFailure, RequestStatus, RequestType, TokenUsage, current_millis,
};
use lib_provider::OpenAICompatibleProvider;
use lib_store::Store;
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::sync::Arc;

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
        Ok(Some(models)) => {
            let data = models
                .into_iter()
                .map(|id| json!({"id": id, "object": "model"}))
                .collect::<Vec<_>>();
            Json(json!({"object": "list", "data": data})).into_response()
        }
        Ok(None) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "default_provider_not_available",
        ),
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
    if body.get("stream").and_then(Value::as_bool).unwrap_or(false) {
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
    let request =
        match request_log::create(&state.store, api_key_id, &resolved, RequestType::Normal).await {
            Ok(request) => request,
            Err(error_value) => return internal(error_value),
        };
    let provider = provider(&resolved);
    match provider
        .chat(body, &resolved.provider_model.upstream_model)
        .await
    {
        Ok(response) => {
            let completed_at = current_millis().unwrap_or(request.started_at);
            let completion = RequestCompletion {
                completed_at,
                response_model: response
                    .get("model")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                status_code: Some(200),
                usage: parse_usage(response.get("usage")),
                latency_ms: completed_at.saturating_sub(request.started_at),
            };
            if let Err(error_value) = state.store.complete_request(request.id, &completion).await {
                tracing::error!(error = %error_value, request_id = %request.id, "更新完成请求日志失败");
            }
            Json(response).into_response()
        }
        Err(provider_error) => {
            log_provider_failure(&state.store, request.id, request.started_at, provider_error)
                .await;
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
    let request =
        match request_log::create(&state.store, api_key_id, &resolved, RequestType::Stream).await {
            Ok(request) => request,
            Err(error_value) => return internal(error_value),
        };
    let upstream = match provider(&resolved)
        .chat_stream(body, &resolved.provider_model.upstream_model)
        .await
    {
        Ok(stream) => stream,
        Err(provider_error) => {
            log_provider_failure(&state.store, request.id, request.started_at, provider_error)
                .await;
            return error(StatusCode::BAD_GATEWAY, "upstream_error");
        }
    };
    sse::response(
        state.store.clone(),
        request.id,
        upstream,
        request.started_at,
    )
}

fn provider(resolved: &lib_core::ResolvedModel) -> OpenAICompatibleProvider {
    OpenAICompatibleProvider::new(
        resolved.provider.id,
        resolved.provider.base_url.clone(),
        resolved.provider.api_key.clone(),
    )
}

async fn log_provider_failure(
    store: &Store,
    request_id: Id,
    started_at: i64,
    provider_error: lib_core::ProviderError,
) {
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
    if let Err(error_value) = store
        .fail_request(request_id, RequestStatus::Failed, &failure)
        .await
    {
        tracing::error!(error = %error_value, request_id = %request_id, "更新失败请求日志失败");
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
