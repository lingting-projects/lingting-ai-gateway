use crate::entity::{
    ApiKey, KvConfig, Provider, ProviderModel, RequestMain, RequestStatus, RequestSub,
};
use framework_proc_auto::auto_type;

// ---------------------------------------------------------------------------
// 供应商
// ---------------------------------------------------------------------------

#[auto_type]
pub struct ProviderQO {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub enabled: Option<bool>,
}

#[auto_type(default = false)]
pub struct ProviderCreatePO {
    pub name: String,
    pub display_name: String,
    pub base_url: String,
    pub api_key: String,
    pub priority: i32,
    pub enabled: bool,
    pub config: Option<serde_json::Value>,
}

#[auto_type(default = false)]
pub struct ProviderUpdatePO {
    pub id: i64,
    pub display_name: String,
    pub base_url: String,
    /// 为空表示不修改已保存的 key。
    pub api_key: Option<String>,
    pub priority: i32,
    pub enabled: bool,
    pub config: Option<serde_json::Value>,
}

#[auto_type(default = false)]
pub struct ProviderVO {
    pub id: i64,
    pub name: String,
    pub display_name: String,
    pub base_url: String,
    /// 已脱敏，只保留首尾字符。
    pub api_key: String,
    pub priority: i32,
    pub enabled: bool,
    pub config: serde_json::Value,
    pub create_time: i64,
    pub update_time: i64,
}

impl From<Provider> for ProviderVO {
    fn from(provider: Provider) -> Self {
        Self {
            id: provider.id,
            name: provider.name,
            display_name: provider.display_name,
            base_url: provider.base_url,
            api_key: mask_secret(&provider.api_key),
            priority: provider.priority,
            enabled: provider.enabled,
            config: provider.config,
            create_time: provider.create_time,
            update_time: provider.update_time,
        }
    }
}

// ---------------------------------------------------------------------------
// 供应商模型
// ---------------------------------------------------------------------------

#[auto_type]
pub struct ProviderModelQO {
    pub id: Option<i64>,
    pub provider_id: Option<i64>,
    pub model: Option<String>,
    pub reasoning: Option<bool>,
    pub enabled: Option<bool>,
}

/// 供应商模型的写入参数，模型同步与新建共用；新增记录固定为启用。
#[auto_type(default = false)]
pub struct ProviderModelCreatePO {
    pub provider_id: i64,
    pub model: String,
    pub display_name: String,
    pub reasoning: bool,
    pub levels: Vec<String>,
    pub level_default: String,
    pub context_window: i64,
    pub max_tokens: i64,
    pub support_tools: bool,
    pub support_vision: bool,
    pub support_stream: bool,
    pub support_json: bool,
    pub support_cache: bool,
    pub knowledge_cutoff: String,
    pub release_date: String,
}

#[auto_type(default = false)]
pub struct ProviderModelUpdatePO {
    pub id: i64,
    pub display_name: String,
    pub levels: Vec<String>,
    pub level_default: String,
    pub enabled: bool,
}

#[auto_type(default = false)]
pub struct ProviderModelVO {
    pub id: i64,
    pub provider_id: i64,
    pub provider_name: String,
    pub model: String,
    pub display_name: String,
    pub reasoning: bool,
    pub levels: Vec<String>,
    pub level_default: String,
    pub context_window: i64,
    pub max_tokens: i64,
    pub support_tools: bool,
    pub support_vision: bool,
    pub support_stream: bool,
    pub support_json: bool,
    pub support_cache: bool,
    pub knowledge_cutoff: String,
    pub release_date: String,
    pub enabled: bool,
    pub create_time: i64,
    pub update_time: i64,
}

impl ProviderModelVO {
    pub fn of(model: ProviderModel, provider_name: String) -> Self {
        Self {
            id: model.id,
            provider_id: model.provider_id,
            provider_name,
            model: model.model,
            display_name: model.display_name,
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
            enabled: model.enabled,
            create_time: model.create_time,
            update_time: model.update_time,
        }
    }
}

/// 供应商分页结果，附带该供应商下的全部模型。
#[auto_type(default = false)]
pub struct ProviderDetailVO {
    pub id: i64,
    pub provider: ProviderVO,
    pub models: Vec<ProviderModelVO>,
}

// ---------------------------------------------------------------------------
// API Key
// ---------------------------------------------------------------------------

#[auto_type]
pub struct ApiKeyQO {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub enabled: Option<bool>,
}

#[auto_type(default = false)]
pub struct ApiKeyCreatePO {
    pub name: String,
    pub remark: String,
}

#[auto_type(default = false)]
pub struct ApiKeyUpdatePO {
    pub id: i64,
    pub name: String,
    pub enabled: bool,
    pub remark: String,
}

#[auto_type(default = false)]
pub struct ApiKeyVO {
    pub id: i64,
    pub name: String,
    pub enabled: bool,
    pub remark: String,
    pub create_time: i64,
    pub update_time: i64,
    /// 原始 key，仅在创建时返回一次。
    pub key: Option<String>,
}

impl From<ApiKey> for ApiKeyVO {
    fn from(api_key: ApiKey) -> Self {
        Self {
            id: api_key.id,
            name: api_key.name,
            enabled: api_key.enabled,
            remark: api_key.remark,
            create_time: api_key.create_time,
            update_time: api_key.update_time,
            key: None,
        }
    }
}

// ---------------------------------------------------------------------------
// 全局配置
// ---------------------------------------------------------------------------

#[auto_type]
pub struct ConfigQO {
    pub config_key: Option<String>,
}

#[auto_type(default = false)]
pub struct ConfigVO {
    pub id: i64,
    pub config_key: String,
    pub config_value: String,
    pub description: String,
    pub create_time: i64,
    pub update_time: i64,
}

impl From<KvConfig> for ConfigVO {
    fn from(config: KvConfig) -> Self {
        Self {
            id: config.id,
            config_key: config.config_key,
            config_value: config.config_value,
            description: config.description,
            create_time: config.create_time,
            update_time: config.update_time,
        }
    }
}

#[auto_type(default = false)]
pub struct ConfigUpdatePO {
    pub config_key: String,
    pub config_value: String,
}

// ---------------------------------------------------------------------------
// 请求日志
// ---------------------------------------------------------------------------

#[auto_type]
pub struct RequestMainQO {
    pub id: Option<i64>,
    pub request_id: Option<String>,
    pub client_request_id: Option<String>,
    pub session_id: Option<String>,
    pub model: Option<String>,
    pub credential_id: Option<String>,
    pub client_ip: Option<String>,
    pub status: Option<RequestStatus>,
    pub start_time_begin: Option<i64>,
    pub start_time_end: Option<i64>,
}

#[auto_type(default = false)]
pub struct RequestMainVO {
    pub id: i64,
    pub request_id: String,
    pub trace_id: String,
    pub client_request_id: String,
    pub session_id: String,
    pub client_ip: String,
    pub user_agent: String,
    pub credential_id: String,
    pub model: String,
    pub stream: bool,
    pub method: String,
    pub path: String,
    pub request_params: Option<serde_json::Value>,
    pub status: RequestStatus,
    pub status_label: String,
    pub current_status: String,
    pub error_type: String,
    pub error_code: String,
    pub error_message: String,
    pub provider_count: i32,
    pub return_model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
    pub inference_tokens: i64,
    pub read_tokens: i64,
    pub write_tokens: i64,
    pub total_tokens: i64,
    pub http_status: Option<i32>,
    pub finish_reason: String,
    pub provider_request_id: String,
    pub response_content: Option<String>,
    pub start_time: i64,
    pub end_time: Option<i64>,
    pub duration_ms: Option<i64>,
    pub create_time: i64,
}

impl From<RequestMain> for RequestMainVO {
    fn from(main: RequestMain) -> Self {
        Self {
            id: main.id,
            request_id: main.request_id,
            trace_id: main.trace_id,
            client_request_id: main.client_request_id,
            session_id: main.session_id,
            client_ip: main.client_ip,
            user_agent: main.user_agent,
            credential_id: main.credential_id,
            model: main.model,
            stream: main.stream,
            method: main.method,
            path: main.path,
            request_params: main.request_params,
            status_label: main.status.label().to_string(),
            status: main.status,
            current_status: main.current_status,
            error_type: main.error_type,
            error_code: main.error_code,
            error_message: main.error_message,
            provider_count: main.provider_count,
            return_model: main.return_model,
            input_tokens: main.input_tokens,
            output_tokens: main.output_tokens,
            cache_read_tokens: main.cache_read_tokens,
            cache_write_tokens: main.cache_write_tokens,
            inference_tokens: main.inference_tokens,
            read_tokens: main.read_tokens,
            write_tokens: main.write_tokens,
            total_tokens: main.total_tokens,
            http_status: main.http_status,
            finish_reason: main.finish_reason,
            provider_request_id: main.provider_request_id,
            response_content: main.response_content,
            start_time: main.start_time,
            end_time: main.end_time,
            duration_ms: main.duration_ms,
            create_time: main.create_time,
        }
    }
}

#[auto_type]
pub struct RequestSubQO {
    pub id: Option<i64>,
    pub main_request_id: Option<i64>,
    pub provider_id: Option<i64>,
    pub provider_name: Option<String>,
    pub model: Option<String>,
    pub status: Option<RequestStatus>,
}

#[auto_type(default = false)]
pub struct RequestSubVO {
    pub id: i64,
    pub main_request_id: i64,
    pub provider_id: i64,
    pub provider_name: String,
    pub model: String,
    pub provider_url: String,
    pub request_params: Option<serde_json::Value>,
    pub status: RequestStatus,
    pub status_label: String,
    pub error_type: String,
    pub error_code: String,
    pub error_message: String,
    pub return_model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
    pub inference_tokens: i64,
    pub read_tokens: i64,
    pub write_tokens: i64,
    pub total_tokens: i64,
    pub http_status: Option<i32>,
    pub finish_reason: String,
    pub provider_request_id: String,
    pub response_content: Option<String>,
    pub start_time: i64,
    pub end_time: Option<i64>,
    pub duration_ms: Option<i64>,
    pub create_time: i64,
}

impl From<RequestSub> for RequestSubVO {
    fn from(sub: RequestSub) -> Self {
        Self {
            id: sub.id,
            main_request_id: sub.main_request_id,
            provider_id: sub.provider_id,
            provider_name: sub.provider_name,
            model: sub.model,
            provider_url: sub.provider_url,
            request_params: sub.request_params,
            status_label: sub.status.label().to_string(),
            status: sub.status,
            error_type: sub.error_type,
            error_code: sub.error_code,
            error_message: sub.error_message,
            return_model: sub.return_model,
            input_tokens: sub.input_tokens,
            output_tokens: sub.output_tokens,
            cache_read_tokens: sub.cache_read_tokens,
            cache_write_tokens: sub.cache_write_tokens,
            inference_tokens: sub.inference_tokens,
            read_tokens: sub.read_tokens,
            write_tokens: sub.write_tokens,
            total_tokens: sub.total_tokens,
            http_status: sub.http_status,
            finish_reason: sub.finish_reason,
            provider_request_id: sub.provider_request_id,
            response_content: sub.response_content,
            start_time: sub.start_time,
            end_time: sub.end_time,
            duration_ms: sub.duration_ms,
            create_time: sub.create_time,
        }
    }
}

/// 主请求日志分页，额外带上该主请求下的所有子请求日志。
#[auto_type(default = false)]
pub struct RequestMainDetailVO {
    pub id: i64,
    pub request: RequestMainVO,
    pub children: Vec<RequestSubVO>,
}

/// 可用模型查询结果。
#[auto_type(default = false)]
pub struct AvailableModelsResult {
    pub models: Vec<String>,
    pub total: i64,
}

/// 请求日志写入时的初始数据。
#[auto_type(default = false)]
pub struct RequestMainCreatePO {
    pub request_id: String,
    pub trace_id: String,
    pub client_request_id: String,
    pub session_id: String,
    pub client_ip: String,
    pub user_agent: String,
    pub credential_id: String,
    pub model: String,
    pub stream: bool,
    pub method: String,
    pub path: String,
    pub request_params: Option<serde_json::Value>,
    pub start_time: i64,
}

#[auto_type(default = false)]
pub struct RequestSubCreatePO {
    pub main_request_id: i64,
    pub provider_id: i64,
    pub provider_name: String,
    pub model: String,
    pub provider_url: String,
    pub request_params: Option<serde_json::Value>,
    pub start_time: i64,
}

/// 请求日志结束时的公共字段。
#[auto_type]
pub struct RequestFinishPO {
    pub status: RequestStatus,
    pub error_type: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub return_model: Option<String>,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
    pub inference_tokens: i64,
    pub read_tokens: i64,
    pub write_tokens: i64,
    pub total_tokens: i64,
    pub http_status: Option<i32>,
    pub finish_reason: Option<String>,
    pub provider_request_id: Option<String>,
    pub response_content: Option<String>,
    pub end_time: i64,
    pub duration_ms: i64,
}

/// 供应商模型更新任务的执行结果。
#[auto_type(default = false)]
pub struct ProviderModelSyncResult {
    pub provider_id: i64,
    pub provider_name: String,
    pub added: i64,
    pub removed: i64,
    pub total: i64,
}

fn mask_secret(value: &str) -> String {
    let length = value.chars().count();
    if length <= 8 {
        return "*".repeat(length);
    }
    let prefix: String = value.chars().take(4).collect();
    let suffix: String = value.chars().skip(length - 4).collect();
    format!("{prefix}****{suffix}")
}
