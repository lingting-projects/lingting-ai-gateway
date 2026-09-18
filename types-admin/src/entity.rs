use framework_proc_auto::{auto_enum, auto_enum_impl, auto_type};

/// 请求状态：进行中、成功、失败、取消。
#[auto_enum(clone = false, copy = false)]
#[derive(Default)]
pub enum RequestStatus {
    #[default]
    Processing,
    Success,
    Failed,
    Cancelled,
}

#[auto_enum_impl]
impl RequestStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Processing => "进行中",
            Self::Success => "成功",
            Self::Failed => "失败",
            Self::Cancelled => "取消",
        }
    }

    #[auto_enum_field]
    pub fn color(&self) -> &'static str {
        match self {
            Self::Processing => "var(--ant-color-processing-text)",
            Self::Success => "var(--ant-color-success-text)",
            Self::Failed => "var(--ant-color-error-text)",
            Self::Cancelled => "var(--ant-color-warning-text)",
        }
    }
}

impl RequestStatus {
    /// 数据库存储值。
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Processing => "processing",
            Self::Success => "success",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    /// 数据库存储值转换为状态，未知值按进行中处理。
    pub fn from_db(value: &str) -> Self {
        match value {
            "success" => Self::Success,
            "failed" => Self::Failed,
            "cancelled" => Self::Cancelled,
            _ => Self::Processing,
        }
    }
}

/// 推理级别。
#[auto_enum(clone = false, copy = false)]
pub enum InferenceLevel {
    Standard,
    Economy,
    Performance,
}

#[auto_enum_impl]
impl InferenceLevel {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Standard => "标准",
            Self::Economy => "经济",
            Self::Performance => "性能",
        }
    }

    #[auto_enum_field]
    pub fn color(&self) -> &'static str {
        match self {
            Self::Standard => "var(--ant-color-primary-text)",
            Self::Economy => "var(--ant-color-success-text)",
            Self::Performance => "var(--ant-color-warning-text)",
        }
    }
}

impl InferenceLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Economy => "economy",
            Self::Performance => "performance",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value {
            "economy" => Self::Economy,
            "performance" => Self::Performance,
            _ => Self::Standard,
        }
    }
}

/// 供应商
#[auto_type(default = false)]
pub struct Provider {
    pub id: i64,
    pub name: String,
    pub display_name: String,
    pub base_url: String,
    pub api_key: String,
    pub priority: i32,
    pub enabled: bool,
    pub config: serde_json::Value,
    pub create_time: i64,
    pub update_time: i64,
    /// 逻辑删除时间（毫秒时间戳），0 表示未删除。
    pub deleted_at: i64,
}

/// 供应商模型，一个供应商对应多条模型数据。
#[auto_type(default = false)]
pub struct ProviderModel {
    pub id: i64,
    pub provider_id: i64,
    pub model: String,
    pub display_name: String,
    pub inference_level: InferenceLevel,
    pub enabled: bool,
    pub create_time: i64,
    pub update_time: i64,
}

/// API Key，仅保存原始 key 的 sha1 值。
#[auto_type(default = false)]
pub struct ApiKey {
    pub id: i64,
    pub name: String,
    pub key_hash: String,
    pub enabled: bool,
    pub deleted: bool,
    pub remark: String,
    pub create_time: i64,
    pub update_time: i64,
}

/// 全局配置
#[auto_type(default = false)]
pub struct KvConfig {
    pub id: i64,
    pub config_key: String,
    pub config_value: String,
    pub description: String,
    pub create_time: i64,
    pub update_time: i64,
}

/// 主请求日志
#[auto_type(default = false)]
pub struct RequestMain {
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

/// 子请求日志
#[auto_type(default = false)]
pub struct RequestSub {
    pub id: i64,
    pub main_request_id: i64,
    pub provider_id: i64,
    pub provider_name: String,
    pub model: String,
    pub provider_url: String,
    pub request_params: Option<serde_json::Value>,
    pub status: RequestStatus,
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

/// Token 计数，未使用的一律为 0。
#[auto_type(clone = true)]
pub struct TokenInfo {
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
    pub inference_tokens: i64,
    pub read_tokens: i64,
    pub write_tokens: i64,
    pub total_tokens: i64,
}

impl TokenInfo {
    pub fn zero() -> Self {
        Self::default()
    }

    /// 按输入、输出、缓存读写、推理、读写合计总 Token。
    pub fn total(&self) -> i64 {
        self.input_tokens
            + self.output_tokens
            + self.cache_read_tokens
            + self.cache_write_tokens
            + self.inference_tokens
            + self.read_tokens
            + self.write_tokens
    }
}
