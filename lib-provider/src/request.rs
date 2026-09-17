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

/// 流式请求的用量上报选项。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StreamOptions {
    #[serde(default)]
    pub include_usage: bool,
}

/// 对话请求，字段对齐 OpenAI Chat Completions。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<StreamOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<Value>,

    /// 未识别字段原样透传。
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl ChatRequest {
    /// 是否请求流式返回。
    pub fn is_stream(&self) -> bool {
        self.stream.unwrap_or(false)
    }

    /// 从请求 JSON 中读取会话标识，用于会话级用量统计。
    pub fn session_id(&self) -> Option<String> {
        self.extra
            .get("session_id")
            .or_else(|| self.extra.get("sessionId"))
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| self.user.clone())
    }

    /// 参数摘要，写入子请求日志。
    pub fn param_summary(&self) -> Value {
        let mut summary = Map::new();
        summary.insert("model".into(), Value::String(self.model.clone()));
        summary.insert("stream".into(), Value::Bool(self.is_stream()));
        summary.insert("messages".into(), Value::from(self.messages.len()));

        let mut optional = Map::new();
        if let Some(value) = self.temperature {
            optional.insert("temperature".into(), Value::from(value));
        }
        if let Some(value) = self.top_p {
            optional.insert("top_p".into(), Value::from(value));
        }
        if let Some(value) = self.max_tokens {
            optional.insert("max_tokens".into(), Value::from(value));
        }
        if let Some(value) = self.max_completion_tokens {
            optional.insert("max_completion_tokens".into(), Value::from(value));
        }
        if let Some(value) = &self.reasoning_effort {
            optional.insert("reasoning_effort".into(), Value::from(value.clone()));
        }
        if !optional.is_empty() {
            summary.insert("options".into(), Value::Object(optional));
        }

        Value::Object(summary)
    }
}

/// 对话请求构造器。
#[derive(Debug, Clone)]
pub struct ChatRequestBuilder {
    request: ChatRequest,
}

impl ChatRequestBuilder {
    /// 以模型名开始构造。
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            request: ChatRequest {
                model: model.into(),
                messages: Vec::new(),
                stream: None,
                stream_options: None,
                temperature: None,
                top_p: None,
                max_tokens: None,
                max_completion_tokens: None,
                n: None,
                stop: None,
                presence_penalty: None,
                frequency_penalty: None,
                logprobs: None,
                top_logprobs: None,
                seed: None,
                user: None,
                reasoning_effort: None,
                response_format: None,
                tools: None,
                tool_choice: None,
                extra: Map::new(),
            },
        }
    }

    /// 追加一条消息。
    pub fn message(mut self, message: ChatMessage) -> Self {
        self.request.messages.push(message);
        self
    }

    /// 设置是否流式返回。
    pub fn stream(mut self, stream: bool) -> Self {
        self.request.stream = Some(stream);
        self
    }

    /// 请求流式返回时要求上报用量。
    pub fn include_usage(mut self, include: bool) -> Self {
        self.request.stream_options = Some(StreamOptions {
            include_usage: include,
        });
        self
    }

    /// 设置温度。
    pub fn temperature(mut self, temperature: f64) -> Self {
        self.request.temperature = Some(temperature);
        self
    }

    /// 设置最大输出 token 数。
    pub fn max_tokens(mut self, max_tokens: i64) -> Self {
        self.request.max_tokens = Some(max_tokens);
        self
    }

    /// 设置推理等级。
    pub fn reasoning_effort(mut self, effort: impl Into<String>) -> Self {
        self.request.reasoning_effort = Some(effort.into());
        self
    }

    /// 完成构造。
    pub fn build(self) -> ChatRequest {
        self.request
    }
}
