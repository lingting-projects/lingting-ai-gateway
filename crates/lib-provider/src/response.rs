use serde::{Deserialize, Serialize};
use serde_json::Value;
use types_admin::TokenInfo;

/// 输入 token 明细。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromptTokensDetails {
    #[serde(default)]
    pub cached_tokens: i64,
}

/// 输出 token 明细。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompletionTokensDetails {
    #[serde(default)]
    pub reasoning_tokens: i64,
}

/// 用量信息。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatUsage {
    #[serde(default)]
    pub prompt_tokens: i64,
    #[serde(default)]
    pub completion_tokens: i64,
    #[serde(default)]
    pub total_tokens: i64,
    #[serde(default)]
    pub prompt_tokens_details: Option<PromptTokensDetails>,
    #[serde(default)]
    pub completion_tokens_details: Option<CompletionTokensDetails>,
}

impl ChatUsage {
    /// 转换为统一用量：输入、输出、缓存读、推理与总量。
    pub fn to_token_info(&self) -> TokenInfo {
        let cache_read = self
            .prompt_tokens_details
            .as_ref()
            .map_or(0, |details| details.cached_tokens);
        let inference = self
            .completion_tokens_details
            .as_ref()
            .map_or(0, |details| details.reasoning_tokens);

        let mut info = TokenInfo::zero();
        info.input_tokens = self.prompt_tokens;
        info.output_tokens = self.completion_tokens;
        info.cache_read_tokens = cache_read;
        info.inference_tokens = inference;
        info.total_tokens = self.total_tokens;
        info
    }

    /// 按输入 / 输出增量累加，流式分片逐个上报时使用。
    pub fn accumulate(&mut self, other: &Self) {
        self.prompt_tokens += other.prompt_tokens;
        self.completion_tokens += other.completion_tokens;
        self.total_tokens += other.total_tokens;

        if let Some(details) = &other.prompt_tokens_details {
            self.prompt_tokens_details
                .get_or_insert_with(PromptTokensDetails::default)
                .cached_tokens += details.cached_tokens;
        }
        if let Some(details) = &other.completion_tokens_details {
            self.completion_tokens_details
                .get_or_insert_with(CompletionTokensDetails::default)
                .reasoning_tokens += details.reasoning_tokens;
        }
    }
}

/// 响应消息，普通响应的 message 与流式响应的 delta 结构一致。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponseMessage {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<Value>>,
}

impl ResponseMessage {
    /// 追加文本内容。
    pub fn append_content(&mut self, content: &str) {
        self.content
            .get_or_insert_with(String::new)
            .push_str(content);
    }

    /// 追加推理内容。
    pub fn append_reasoning(&mut self, content: &str) {
        self.reasoning_content
            .get_or_insert_with(String::new)
            .push_str(content);
    }
}

/// 一个候选结果，普通响应取 message，流式分片取 delta。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatChoice {
    #[serde(default)]
    pub index: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<ResponseMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delta: Option<ResponseMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

impl ChatChoice {
    /// 取消息体，普通响应看 message，流式分片看 delta。
    pub fn body(&self) -> Option<&ResponseMessage> {
        self.message.as_ref().or(self.delta.as_ref())
    }
}

/// 对话响应，普通响应与流式分片共用同一结构。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatResponse {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub choices: Vec<ChatChoice>,
    #[serde(default)]
    pub usage: Option<ChatUsage>,
}

impl ChatResponse {
    /// 首个候选的结束原因。
    pub fn finish_reason(&self) -> String {
        self.choices
            .first()
            .and_then(|choice| choice.finish_reason.clone())
            .unwrap_or_default()
    }

    /// 首个候选的文本内容。
    pub fn content(&self) -> String {
        self.choices
            .first()
            .and_then(ChatChoice::body)
            .and_then(|message| message.content.clone())
            .unwrap_or_default()
    }

    /// 首个候选的推理内容。
    pub fn reasoning_content(&self) -> String {
        self.choices
            .first()
            .and_then(ChatChoice::body)
            .and_then(|message| message.reasoning_content.clone())
            .unwrap_or_default()
    }

    /// 用量信息；供应商未上报时按内容估算输入输出为零。
    pub fn token_info(&self) -> TokenInfo {
        self.usage
            .as_ref()
            .map(ChatUsage::to_token_info)
            .unwrap_or_else(TokenInfo::zero)
    }

    /// 把流式分片合并进当前响应，用于流式转发结束后得到聚合结果。
    pub fn merge_chunk(&mut self, chunk: &ChatResponse) {
        if !chunk.id.is_empty() {
            self.id = chunk.id.clone();
        }
        if !chunk.model.is_empty() {
            self.model = chunk.model.clone();
        }

        for choice in &chunk.choices {
            let target = match self
                .choices
                .iter_mut()
                .find(|existing| existing.index == choice.index)
            {
                Some(existing) => existing,
                None => {
                    self.choices.push(ChatChoice {
                        index: choice.index,
                        ..ChatChoice::default()
                    });
                    self.choices.last_mut().expect("刚插入的候选必然存在")
                }
            };

            if let Some(finish_reason) = &choice.finish_reason {
                target.finish_reason = Some(finish_reason.clone());
            }
            if let Some(delta) = &choice.delta {
                let message = target.message.get_or_insert_with(ResponseMessage::default);
                if delta.role.is_some() {
                    message.role = delta.role.clone();
                }
                if let Some(content) = &delta.content {
                    message.append_content(content);
                }
                if let Some(reasoning) = &delta.reasoning_content {
                    message.append_reasoning(reasoning);
                }
                if let Some(tool_calls) = &delta.tool_calls {
                    message
                        .tool_calls
                        .get_or_insert_with(Vec::new)
                        .extend(tool_calls.iter().cloned());
                }
            }
        }

        if let Some(usage) = &chunk.usage {
            match self.usage.as_mut() {
                Some(current) => current.accumulate(usage),
                None => self.usage = Some(usage.clone()),
            }
        }
    }

    /// 该分片是否只携带用量，没有正文。
    pub fn is_usage_only(&self) -> bool {
        self.choices.is_empty() && self.usage.is_some()
    }
}

/// 供应商返回的结果：响应体与 HTTP 状态码。
#[derive(Debug, Clone)]
pub struct ProviderResponse {
    /// 响应体，流式请求为聚合后的结果。
    pub response: ChatResponse,
    /// 供应商返回的 HTTP 状态码。
    pub http_status: i32,
}
