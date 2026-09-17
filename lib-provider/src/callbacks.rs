use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use types_admin::TokenInfo;

use crate::error::ProviderError;

/// 子请求开始信息，回调实现据此写入子请求日志。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubRequestStart {
    /// 供应商 ID。
    pub provider_id: i64,
    /// 供应商标识。
    pub provider_name: String,
    /// 请求模型。
    pub model: String,
    /// 请求地址。
    pub provider_url: String,
    /// 请求参数摘要。
    pub request_params: Value,
    /// 请求时间。
    pub start_time: i64,
}

/// 一次转发的结果，回调实现据此更新主/子请求日志。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ForwardResult {
    /// 返回模型。
    pub model: String,
    /// token 计数。
    pub usage: TokenInfo,
    /// 返回内容，仅调试模式下填充。
    pub content: Option<String>,
    /// 供应商返回的 HTTP 状态码。
    pub http_status: Option<u16>,
    /// 结束原因。
    pub finish_reason: Option<String>,
    /// 供应商请求 ID。
    pub provider_request_id: Option<String>,
    /// 已转发分块数量。
    pub chunk_count: u64,
    /// 请求开始时间。
    pub start_time: i64,
    /// 结束时间。
    pub end_time: i64,
    /// 耗时毫秒。
    pub duration_ms: i64,
}

/// 供应商客户端回调。
///
/// 客户端只负责在合适的时机回调，日志写入由实现方完成；
/// 回调内部异常不允许影响请求本身的完成。
#[async_trait]
pub trait ProviderCallbacks: Send + Sync {
    /// 请求发起前。
    async fn on_start(&self, start: SubRequestStart) {
        let _ = start;
    }

    /// 请求正常结束。
    async fn on_success(&self, result: ForwardResult) {
        let _ = result;
    }

    /// 请求失败。
    async fn on_failure(&self, error: ProviderError, result: ForwardResult) {
        let _ = (error, result);
    }

    /// 客户端取消请求。
    async fn on_cancel(&self, result: ForwardResult) {
        let _ = result;
    }
}
