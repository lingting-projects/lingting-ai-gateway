use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

pub use framework_core::{Snowflake, next_id};
pub use framework_datetime::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Id(i64);

impl Id {
    pub fn new() -> anyhow::Result<Self> {
        next_id().map(Self)
    }

    pub const fn zero() -> Self {
        Self(0)
    }

    pub const fn from_value(value: i64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> i64 {
        self.0
    }
}

impl fmt::Debug for Id {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for Id {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone)]
pub struct ProviderConfig {
    pub id: Id,
    pub name: String,
    pub base_url: String,
    pub api_key: String,
}

impl fmt::Debug for ProviderConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProviderConfig")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("base_url", &self.base_url)
            .field("api_key", &"[REDACTED]")
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct ProviderModel {
    pub id: Id,
    pub provider_id: Id,
    pub model_name: String,
    pub upstream_model: String,
}

#[derive(Debug, Clone)]
pub struct ResolvedModel {
    pub client_model: String,
    pub provider: ProviderConfig,
    pub provider_model: ProviderModel,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
    pub cache_read_input_tokens: Option<i64>,
    pub cache_write_input_tokens: Option<i64>,
    pub reasoning_tokens: Option<i64>,
    pub input_audio_tokens: Option<i64>,
    pub output_audio_tokens: Option<i64>,
}

#[derive(Debug, Clone, Copy)]
pub enum RequestType {
    Normal,
    Stream,
}

impl RequestType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Stream => "stream",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RequestStatus {
    Connecting,
    Completed,
    Failed,
    ClientDisconnected,
}

impl RequestStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Connecting => "connecting",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::ClientDisconnected => "client_disconnected",
        }
    }
}

#[derive(Debug, Clone)]
pub struct NewRequestLog {
    pub id: Id,
    pub api_key_id: Id,
    pub client_model: String,
    pub provider_id: Id,
    pub provider_model: String,
    pub request_type: RequestType,
    pub status: RequestStatus,
    pub started_at: i64,
}

#[derive(Debug, Clone, Default)]
pub struct RequestCompletion {
    pub completed_at: i64,
    pub response_model: Option<String>,
    pub status_code: Option<i32>,
    pub usage: TokenUsage,
    pub latency_ms: i64,
}

#[derive(Debug, Clone)]
pub struct RequestFailure {
    pub completed_at: i64,
    pub response_model: Option<String>,
    pub status_code: Option<i32>,
    pub usage: TokenUsage,
    pub latency_ms: i64,
    pub error_type: String,
    pub error_code: String,
    pub error_message: String,
}

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("upstream request failed")]
    Http,
    #[error("upstream returned status {status}")]
    Status { status: u16 },
    #[error("invalid upstream response")]
    InvalidResponse,
}
