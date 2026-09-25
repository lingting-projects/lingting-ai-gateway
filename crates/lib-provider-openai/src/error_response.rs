//! [OI] 错误响应模型：供应商以 4xx/5xx 返回时的统一结构。
//!
//! ```json
//! {
//!   "error": {
//!     "message": "具体错误信息",
//!     "type": "server_error",
//!     "param": null,
//!     "code": "..."
//!   }
//! }
//! ```

use serde::Deserialize;

/// 错误详情。
///
/// 各字段都是可空的：上游对 `param`、`code` 常直接返回 `null`，
/// 用 `Option` 而不是 `default` 才能接受 `null`（`default` 只处理字段缺失）。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct OpenAiError {
    /// 具体错误信息。
    #[serde(default)]
    pub message: Option<String>,
    /// 错误类别，例如 `server_error`、`invalid_request_error`。
    #[serde(default, rename = "type")]
    pub error_type: Option<String>,
    /// 出错参数名。
    #[serde(default)]
    pub param: Option<String>,
    /// 错误码。
    #[serde(default)]
    pub code: Option<String>,
}

/// 错误响应体。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct OpenAiErrorResponse {
    /// 错误详情；缺失表示不是错误结构。
    #[serde(default)]
    pub error: Option<OpenAiError>,
}

/// 从返回文本解析 [OI] 错误结构；不是该结构时返回 `None`。
///
/// 必须同时具备 `error` 对象与非空 `message`，否则成功响应也能被反序列化成默认值。
pub fn parse_error(content: &str) -> Option<OpenAiError> {
    let response = serde_json::from_str::<OpenAiErrorResponse>(content).ok()?;
    let error = response.error?;
    if !error
        .message
        .as_deref()
        .is_some_and(|message| !message.is_empty())
    {
        return None;
    }

    Some(error)
}
