use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// 对话角色。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
    Developer,
}

/// 消息内容：纯文本或多模态分段。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<Value>),
}

/// 一条对话消息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<MessageContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// 未识别字段原样透传，避免丢失供应商扩展参数。
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl ChatMessage {
    /// 构造一条纯文本消息。
    pub fn text(role: MessageRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: Some(MessageContent::Text(content.into())),
            name: None,
            tool_calls: None,
            tool_call_id: None,
            extra: Map::new(),
        }
    }
}

/// 对话请求，仅保留网关路由与日志所需的字段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    /// 请求的模型名。
    pub model: String,
    /// 是否请求流式返回。
    pub stream: bool,
    /// 会话标识，用于会话级用量统计。
    pub session_id: Option<String>,
}
