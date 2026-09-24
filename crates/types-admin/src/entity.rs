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
    pub enabled: bool,
    pub create_time: i64,
    pub update_time: i64,
}

/// 模型名称映射，把非官方模型名映射到官方模型名，用于复用官方模型的基础配置。
#[auto_type(default = false)]
pub struct ProviderModelRedirect {
    pub id: i64,
    /// 非官方模型名。
    pub source: String,
    /// 映射到的官方模型名。
    pub target: String,
    pub create_time: i64,
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

/// 全局配置键，供前端解析与展示。
///
/// 数据库存储值统一为 `_` 分隔的全大写形式，与 `Display` / `FromStr` 保持一致；
/// `FromStr` 不区分大小写，历史的小写值也能解析。
#[auto_enum]
pub enum KvConfigKey {
    /// 是否允许匿名访问 AI 接口
    AllowAnonymous,
    /// 是否开启调试模式
    DebugMode,
    /// 管理接口令牌
    AdminToken,
    /// 服务绑定地址
    BindAddress,
    /// 服务绑定端口
    BindPort,
}

#[auto_enum_impl]
impl KvConfigKey {
    pub fn label(&self) -> &'static str {
        match self {
            Self::AllowAnonymous => "允许匿名访问",
            Self::DebugMode => "调试模式",
            Self::AdminToken => "管理令牌",
            Self::BindAddress => "绑定地址",
            Self::BindPort => "绑定端口",
        }
    }

    #[auto_enum_field]
    pub fn description(&self) -> &'static str {
        match self {
            Self::AllowAnonymous => "是否允许匿名访问 AI 接口",
            Self::DebugMode => "是否开启调试模式，开启后记录完整请求参数与返回内容",
            Self::AdminToken => "管理接口令牌，保存 sha1 值，为空表示不校验",
            Self::BindAddress => "服务绑定地址",
            Self::BindPort => "服务绑定端口，0 表示随机端口",
        }
    }
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
    /// 网关收到的请求头，仅调试模式记录，已移除逐跳头。
    pub request_headers: Option<serde_json::Value>,
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
    pub total_tokens: i64,
    pub http_status: Option<i32>,
    pub finish_reason: String,
    /// 供应商返回的请求 ID 集合。
    pub provider_request_ids: Vec<String>,
    pub response_content: Option<String>,
    /// 返回给客户端的响应头，仅调试模式记录，已移除逐跳头。
    pub response_headers: Option<serde_json::Value>,
    /// 请求收到时间（毫秒时间戳）。
    pub start_time: i64,
    /// 首字时间（毫秒时间戳），非流式与完成时间一致，为空表示未产生分片。
    pub first_chunk_time: Option<i64>,
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
    /// 实际转发给供应商的请求头，仅调试模式记录，已移除逐跳头且鉴权已脱敏。
    pub request_headers: Option<serde_json::Value>,
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
    pub total_tokens: i64,
    pub http_status: Option<i32>,
    pub finish_reason: String,
    /// 供应商返回的请求 ID 集合。
    pub provider_request_ids: Vec<String>,
    pub response_content: Option<String>,
    /// 供应商返回的响应头，仅调试模式记录，已移除逐跳头。
    pub response_headers: Option<serde_json::Value>,
    /// 转发开始时间（毫秒时间戳）。
    pub start_time: i64,
    /// 首字时间（毫秒时间戳），非流式与完成时间一致，为空表示未产生分片。
    pub first_chunk_time: Option<i64>,
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
    pub total_tokens: i64,
}

impl TokenInfo {
    pub fn zero() -> Self {
        Self::default()
    }
}
