use serde::{Deserialize, Serialize};

/// 请求路由信息，仅保留网关路由与日志所需的字段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRequest {
    /// 请求的模型名。
    pub model: String,
    /// 是否请求流式返回。
    pub stream: bool,
    /// 会话标识，用于会话级用量统计。
    pub session_id: Option<String>,
}
