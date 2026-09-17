//! OpenAI 兼容接口的通用类型与计算方法。

use serde::{Deserialize, Serialize};
use types_admin::TokenInfo;

/// OpenAI 模型对象。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiModel {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub owned_by: String,
}

impl OpenAiModel {
    /// 以模型名与创建时间（秒）构造，归属为当前网关。
    pub fn new(id: impl Into<String>, created: i64) -> Self {
        Self {
            id: id.into(),
            object: "model".to_string(),
            created,
            owned_by: lib_core::APP_ID.to_string(),
        }
    }
}

/// OpenAI 模型列表响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiModelList {
    pub object: String,
    pub data: Vec<OpenAiModel>,
}

impl OpenAiModelList {
    /// 构造模型列表响应。
    pub fn new(data: Vec<OpenAiModel>) -> Self {
        Self {
            object: "list".to_string(),
            data,
        }
    }
}

/// 把一次转发的用量累加进目标用量。
///
/// 流式场景下每个分片都会上报用量，全部字段（含总量）逐片累加。
pub fn accumulate_token_info(target: &mut TokenInfo, source: &TokenInfo) {
    target.input_tokens += source.input_tokens;
    target.output_tokens += source.output_tokens;
    target.cache_read_tokens += source.cache_read_tokens;
    target.cache_write_tokens += source.cache_write_tokens;
    target.inference_tokens += source.inference_tokens;
    target.read_tokens += source.read_tokens;
    target.write_tokens += source.write_tokens;
    target.total_tokens += source.total_tokens;
}
