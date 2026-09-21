use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use anyhow::{Result, anyhow};
use bytes::Bytes;
use dashmap::DashMap;
use framework_core::MultiStringValue;
use framework_web::WebContext;
use http::Response as HttpResponse;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, RequestBuilder, Response, StatusCode};
use types_admin::entity::Provider;

/// 供应商 HTTP 客户端缓存：同一个供应商始终复用同一个客户端。
static CLIENTS: LazyLock<DashMap<i64, Arc<Client>>> = LazyLock::new(DashMap::new);

/// 取供应商对应的 HTTP 客户端，首次调用时创建并缓存。
pub fn build_client(provider: &Provider) -> Arc<Client> {
    let entry = CLIENTS
        .entry(provider.id)
        .or_insert_with(|| Arc::new(Client::new()));
    Arc::clone(entry.value())
}

/// 转发时跳过的请求头与响应头。
///
/// 逐跳头只对单条连接有效；`host` 与 `content-length` 必须按实际连接和请求体重新生成，
/// 原样转发会与 reqwest 冲突。
pub const SKIPPED_HEADERS: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
    "host",
    "content-length",
];

/// 一次转发请求：请求构造器在构造时生成，发送与响应头处理统一在这里完成。
pub struct ForwardRequest {
    /// 供应商 api_key，转发时覆盖客户端自带的鉴权头。
    api_key: String,
    /// 请求构造器，外部可继续追加超时、查询参数等内容。
    pub builder: RequestBuilder,
}

impl ForwardRequest {
    /// 绑定供应商与请求后缀；后缀直接拼在 `base_url` 之后，例如 `/chat/completions`。
    pub fn new(provider: &Provider, suffix: &str) -> Self {
        let client = build_client(provider);
        let url = format!("{}{}", provider.base_url.trim_end_matches('/'), suffix);
        Self {
            api_key: provider.api_key.clone(),
            builder: client.post(url),
        }
    }

    /// 按当前请求上下文填充请求头与请求体，并返回填充后的请求。
    pub fn forward(mut self, web_context: &WebContext) -> Result<Self> {
        let headers = self.forward_headers(web_context)?;
        self.builder = self.builder.headers(headers).body(web_context.body());
        Ok(self)
    }

    /// 发起请求；传输层异常转换为 502 响应，保证调用方始终拿到响应。
    pub async fn call(self) -> Response {
        match self.builder.send().await {
            Ok(mut response) => {
                let headers = response.headers_mut();
                for name in SKIPPED_HEADERS {
                    headers.remove(*name);
                }
                response
            }
            Err(error) => {
                tracing::warn!("转发请求失败：{error:#}");
                transport_response(&error)
            }
        }
    }

    /// 原样转发客户端请求头，鉴权换成供应商自己的 api_key。
    fn forward_headers(&self, web_context: &WebContext) -> Result<HeaderMap> {
        build_forward_headers(web_context, &self.api_key)
    }
}

/// 按转发规则构造供应商请求头：移除逐跳头，鉴权换成供应商自己的 api_key。
///
/// 请求日志需要复现实际转发内容时也走这里，保证与真实转发完全一致。
pub fn build_forward_headers(web_context: &WebContext, api_key: &str) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    web_context.request().headers.for_each(|name, values| {
        if is_skipped_header(name) {
            return;
        }
        let Ok(name) = HeaderName::from_bytes(name.as_bytes()) else {
            return;
        };
        for value in values {
            if let Ok(value) = HeaderValue::from_str(value) {
                headers.append(name.clone(), value);
            }
        }
    });

    let authorization = HeaderValue::from_str(&format!("Bearer {api_key}"))
        .map_err(|error| anyhow!("供应商 api_key 无法作为请求头使用：{error:#}"))?;
    headers.insert(AUTHORIZATION, authorization);
    Ok(headers)
}

/// 请求头 / 响应头转为键值集合；同名多值全部保留。
pub fn headers_to_multi(headers: &HeaderMap) -> MultiStringValue {
    let mut values = HashMap::<String, Vec<String>>::new();
    for (name, value) in headers {
        let Ok(value) = value.to_str() else {
            continue;
        };
        values
            .entry(name.as_str().to_string())
            .or_default()
            .push(value.to_string());
    }
    MultiStringValue::create(true, values)
}

/// 传输层异常：合成 502 响应，失败原因放在 JSON 响应体中。
fn transport_response(error: &reqwest::Error) -> Response {
    let body = serde_json::json!({
        "code": StatusCode::BAD_GATEWAY.as_u16(),
        "message": error.to_string(),
    })
    .to_string();

    let mut response = HttpResponse::new(Bytes::from(body));
    *response.status_mut() = StatusCode::BAD_GATEWAY;
    response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/json; charset=utf-8"),
    );
    response.into()
}

/// 是否为需要跳过的请求头 / 响应头。
pub fn is_skipped_header(name: &str) -> bool {
    SKIPPED_HEADERS
        .iter()
        .any(|skipped| name.eq_ignore_ascii_case(skipped))
}
