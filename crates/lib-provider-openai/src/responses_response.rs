//! Responses 响应模型：普通响应与流式事件携带的响应对象共用同一结构。

use serde::{Deserialize, Serialize};
use types_admin::TokenInfo;

/// 输入 token 明细。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponsesInputTokensDetails {
    #[serde(default)]
    pub cached_tokens: i64,
}

/// 输出 token 明细。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponsesOutputTokensDetails {
    #[serde(default)]
    pub reasoning_tokens: i64,
}

/// 用量信息。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponsesUsage {
    #[serde(default)]
    pub input_tokens: i64,
    #[serde(default)]
    pub output_tokens: i64,
    #[serde(default)]
    pub total_tokens: i64,
    #[serde(default)]
    pub input_tokens_details: Option<ResponsesInputTokensDetails>,
    #[serde(default)]
    pub output_tokens_details: Option<ResponsesOutputTokensDetails>,
}

impl ResponsesUsage {
    /// 转换为统一用量：输入、输出、缓存读、推理与总量。
    pub fn to_token_info(&self) -> TokenInfo {
        let cache_read = self
            .input_tokens_details
            .as_ref()
            .map_or(0, |details| details.cached_tokens);
        let inference = self
            .output_tokens_details
            .as_ref()
            .map_or(0, |details| details.reasoning_tokens);

        let mut info = TokenInfo::zero();
        info.input_tokens = self.input_tokens;
        info.output_tokens = self.output_tokens;
        info.cache_read_tokens = cache_read;
        info.inference_tokens = inference;
        info.total_tokens = self.total_tokens;
        info
    }
}

/// Responses 响应，普通响应与流式事件携带的响应对象共用同一结构。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponsesResponse {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub model: String,
    /// 响应状态，例如 completed、incomplete、failed。
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub usage: Option<ResponsesUsage>,
}

impl ResponsesResponse {
    /// 用量信息；供应商未上报时全部为零。
    pub fn token_info(&self) -> TokenInfo {
        self.usage
            .as_ref()
            .map(ResponsesUsage::to_token_info)
            .unwrap_or_else(TokenInfo::zero)
    }

    /// 结束原因直接取响应状态。
    pub fn finish_reason(&self) -> String {
        self.status.clone()
    }

    /// 合并流式事件携带的响应对象：事件里是完整响应，逐字段覆盖。
    pub fn merge(&mut self, chunk: &Self) {
        if !chunk.id.is_empty() {
            self.id = chunk.id.clone();
        }
        if !chunk.model.is_empty() {
            self.model = chunk.model.clone();
        }
        if !chunk.status.is_empty() {
            self.status = chunk.status.clone();
        }
        if chunk.usage.is_some() {
            self.usage = chunk.usage.clone();
        }
    }
}

/// 流式事件；只有携带响应对象的事件需要合并，其余事件忽略。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponsesStreamEvent {
    #[serde(default)]
    pub response: Option<ResponsesResponse>,
}
