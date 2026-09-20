//! OpenAI 兼容接口的通用类型与计算方法。

use serde::{Deserialize, Serialize};
use types_admin::TokenInfo;
use types_admin::entity::ProviderModel;

/// OpenAI 模型对象。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiModel {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub owned_by: String,
    /// 是否推理模型，会产出思维链。
    pub reasoning: bool,
    /// 支持的推理级别列表，级别名称由厂商与模型自行定义。
    pub levels: Vec<String>,
    /// 默认推理级别，空串表示由上游决定。
    pub level_default: String,
    /// 上下文窗口上限（输入与输出 token 合计），0 表示未知。
    pub context_window: i64,
    /// 单次响应最大输出 token，0 表示未知。
    pub max_tokens: i64,
    /// 是否支持函数调用。
    pub support_tools: bool,
    /// 是否支持图片输入。
    pub support_vision: bool,
    /// 是否支持流式返回。
    pub support_stream: bool,
    /// 是否支持结构化输出。
    pub support_json: bool,
    /// 是否支持提示词缓存。
    pub support_cache: bool,
    /// 知识截止日期（毫秒时间戳），0 表示未知。
    pub knowledge_cutoff: i64,
    /// 模型发布日期（毫秒时间戳），0 表示未知。
    pub release_date: i64,
}

impl OpenAiModel {
    /// 由供应商模型构造，创建时间取秒级时间戳，归属为当前网关。
    pub fn from_provider_model(model: ProviderModel) -> Self {
        Self {
            id: model.model,
            object: "model".to_string(),
            created: model.create_time / 1000,
            owned_by: lib_core::APP_ID.to_string(),
            reasoning: model.reasoning,
            levels: model.levels,
            level_default: model.level_default,
            context_window: model.context_window,
            max_tokens: model.max_tokens,
            support_tools: model.support_tools,
            support_vision: model.support_vision,
            support_stream: model.support_stream,
            support_json: model.support_json,
            support_cache: model.support_cache,
            knowledge_cutoff: model.knowledge_cutoff,
            release_date: model.release_date,
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
