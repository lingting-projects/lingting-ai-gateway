//! 远程模型列表类型。
//!
//! 供应商模型同步把上游返回的模型先落到 [`RemoteModel`]，再与默认模型配置合并。
//! 本类型与协议无关：各协议的具体响应结构由对应的 `lib-provider-*` 解析后转换。

/// 上游返回的单个模型，未返回的字段为 `None`，由默认模型配置补足。
#[derive(Debug, Clone)]
pub struct RemoteModel {
    /// 模型名，即请求时使用的模型标识。
    pub id: String,
    /// 模型展示名。
    pub display_name: Option<String>,
    /// 是否推理模型，会产出思维链。
    pub reasoning: Option<bool>,
    /// 支持的推理级别列表，级别名称由厂商与模型自行定义。
    pub levels: Option<Vec<String>>,
    /// 默认推理级别，空串表示由上游决定。
    pub level_default: Option<String>,
    /// 上下文窗口上限（输入与输出 token 合计）。
    pub context_window: Option<i64>,
    /// 单次响应最大输出 token。
    pub max_tokens: Option<i64>,
    /// 是否支持函数调用。
    pub support_tools: Option<bool>,
    /// 是否支持图片输入。
    pub support_vision: Option<bool>,
    /// 是否支持流式返回。
    pub support_stream: Option<bool>,
    /// 是否支持结构化输出。
    pub support_json: Option<bool>,
    /// 是否支持提示词缓存。
    pub support_cache: Option<bool>,
    /// 知识截止日期（毫秒时间戳）。
    pub knowledge_cutoff: Option<i64>,
    /// 模型发布日期（毫秒时间戳）。
    pub release_date: Option<i64>,
}
