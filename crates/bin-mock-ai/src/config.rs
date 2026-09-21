//! mock 行为配置：由请求头与查询参数共同决定，请求头优先。

use axum::http::HeaderMap;
use serde::Deserialize;

/// mock 行为配置。
#[derive(Debug, Clone, Copy)]
pub(crate) struct MockConfig {
    /// 发送响应头前的延迟（毫秒）。
    pub(crate) delay_ms: u64,
    /// 首个分片前的延迟（毫秒）。
    pub(crate) first_chunk_ms: u64,
    /// 分片之间的间隔（毫秒）。
    pub(crate) chunk_interval_ms: u64,
    /// 分片数量。
    pub(crate) chunks: usize,
    /// 发完分片后是否挂起。
    pub(crate) hang: bool,
}

impl MockConfig {
    /// 解析配置；请求头优先于查询参数。
    pub(crate) fn resolve(headers: &HeaderMap, query: &MockQuery) -> Self {
        Self {
            delay_ms: number(headers, query.delay_ms, "x-mock-delay-ms", 0),
            first_chunk_ms: number(headers, query.first_chunk_ms, "x-mock-first-chunk-ms", 0),
            chunk_interval_ms: number(
                headers,
                query.chunk_interval_ms,
                "x-mock-chunk-interval-ms",
                100,
            ),
            chunks: number(headers, query.chunks, "x-mock-chunks", 3) as usize,
            hang: flag(headers, query.hang, "x-mock-hang"),
        }
    }
}

/// 查询参数形式的配置。
#[derive(Debug, Default, Deserialize)]
pub(crate) struct MockQuery {
    delay_ms: Option<u64>,
    first_chunk_ms: Option<u64>,
    chunk_interval_ms: Option<u64>,
    chunks: Option<u64>,
    hang: Option<bool>,
}

/// 取数值配置：请求头优先，其次查询参数，最后默认值。
fn number(headers: &HeaderMap, query: Option<u64>, name: &str, default: u64) -> u64 {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().parse().ok())
        .or(query)
        .unwrap_or(default)
}

/// 取布尔配置：请求头优先，其次查询参数，最后默认关闭。
fn flag(headers: &HeaderMap, query: Option<bool>, name: &str) -> bool {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(|value| matches!(value.trim(), "1" | "true" | "yes"))
        .or(query)
        .unwrap_or(false)
}
