//! 模型列表接口：拉取上游模型列表并转换为远程模型。

use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use lib_provider::build_client;
use lib_provider::models::RemoteModel;
use serde::{Deserialize, Deserializer};
use types_admin::entity::Provider;

/// 模型列表接口的路径后缀。
const MODELS_SUFFIX: &str = "/models";

/// 图片模态标识。
const IMAGE_MODALITY: &str = "image";

/// 拉取供应商模型列表，请求失败、响应异常或解析失败均返回错误。
///
/// 兼容两类响应字段：本网关 `[OI]` 模型列表的 `reasoning` / `levels` / `max_tokens` /
/// `support_*` / 日期字段，与外部供应商的 `name` / `max_output_tokens` /
/// `input_modalities` / `effort` 风格。语义相同的字段以显式字段为准，两者都未返回的
/// 字段保持 `None`，由默认模型配置补足。
pub async fn fetch_models(provider: &Provider) -> Result<Vec<RemoteModel>> {
    let url = format!("{}{MODELS_SUFFIX}", provider.base_url.trim_end_matches('/'));
    let response = build_client(provider)
        .get(&url)
        .bearer_auth(&provider.api_key)
        .send()
        .await
        .with_context(|| format!("请求模型列表失败：{url}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        bail!("模型列表响应状态码异常：{status}，响应体 {body}");
    }

    let body = response.text().await.context("读取模型列表响应体失败")?;
    let list = serde_json::from_str::<OpenaiModelList>(&body)
        .with_context(|| format!("解析模型列表响应失败，响应体 {body}"))?;

    Ok(list
        .data
        .into_iter()
        .map(OpenaiModel::into_remote)
        .collect())
}

/// 模型列表响应，`data` 缺失时视为空列表。
#[derive(Debug, Clone, Deserialize)]
struct OpenaiModelList {
    #[serde(default)]
    data: Vec<OpenaiModel>,
}

/// 模型列表中的单个模型，兼容两类上游的字段命名。
#[derive(Debug, Clone, Deserialize)]
struct OpenaiModel {
    id: String,
    /// 外部供应商的模型展示名。
    name: Option<String>,
    /// 上下文窗口上限。
    context_window: Option<i64>,
    /// 最大输出 token；本网关返回 `max_tokens`。
    #[serde(alias = "max_tokens")]
    max_output_tokens: Option<i64>,
    /// 输入模态列表，含 `image` 表示支持图片输入。
    input_modalities: Option<Vec<String>>,
    /// 外部供应商的推理配置。
    effort: Option<Effort>,
    /// 是否推理模型；本网关返回该字段。
    reasoning: Option<bool>,
    /// 支持的推理级别列表；本网关返回该字段。
    levels: Option<Vec<String>>,
    /// 默认推理级别；本网关返回该字段。
    level_default: Option<String>,
    support_tools: Option<bool>,
    support_vision: Option<bool>,
    support_stream: Option<bool>,
    support_json: Option<bool>,
    support_cache: Option<bool>,
    #[serde(default, deserialize_with = "date_millis")]
    knowledge_cutoff: Option<i64>,
    #[serde(default, deserialize_with = "date_millis")]
    release_date: Option<i64>,
}

/// 外部供应商的推理配置。
#[derive(Debug, Clone, Deserialize)]
struct Effort {
    supported_levels: Option<Vec<String>>,
    default_level: Option<String>,
}

impl OpenaiModel {
    /// 转换为远程模型：显式字段优先，其次取外部供应商字段，都未返回时为 `None`。
    fn into_remote(self) -> RemoteModel {
        // 外部供应商用 effort 表达推理配置：存在即视为推理模型，级别也取自其中。
        let (effort_reasoning, effort_levels, effort_default) = match self.effort {
            Some(effort) => (true, effort.supported_levels, effort.default_level),
            None => (false, None, None),
        };

        RemoteModel {
            id: self.id,
            display_name: self.name,
            reasoning: self.reasoning.or(effort_reasoning.then_some(true)),
            levels: self.levels.or(effort_levels),
            level_default: self.level_default.or(effort_default),
            context_window: self.context_window,
            max_tokens: self.max_output_tokens,
            support_tools: self.support_tools,
            support_vision: self.support_vision.or(vision(&self.input_modalities)),
            support_stream: self.support_stream,
            support_json: self.support_json,
            support_cache: self.support_cache,
            knowledge_cutoff: self.knowledge_cutoff,
            release_date: self.release_date,
        }
    }
}

/// 由输入模态推断是否支持图片：未返回模态列表时为 `None`。
fn vision(input_modalities: &Option<Vec<String>>) -> Option<bool> {
    input_modalities
        .as_ref()
        .map(|modalities| modalities.iter().any(|modality| modality == IMAGE_MODALITY))
}

/// 日期字段反序列化：接受毫秒时间戳数字，或 `YYYY-MM-DD` / `YYYY-MM` 字符串。
///
/// 字符串按 UTC 零点解析，`YYYY-MM` 取当月 1 日；无法识别时视为未返回。
fn date_millis<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(value.and_then(|value| match value {
        serde_json::Value::Number(number) => number.as_i64(),
        serde_json::Value::String(text) => parse_date(&text),
        _ => None,
    }))
}

/// 解析 `YYYY-MM-DD` / `YYYY-MM` 为 UTC 零点毫秒时间戳。
fn parse_date(text: &str) -> Option<i64> {
    let text = text.trim();
    ["%Y-%m-%d", "%Y-%m"].iter().find_map(|format| {
        NaiveDate::parse_from_str(text, format)
            .ok()
            .and_then(|date| date.and_hms_opt(0, 0, 0))
            .map(|time| time.and_utc().timestamp_millis())
    })
}
