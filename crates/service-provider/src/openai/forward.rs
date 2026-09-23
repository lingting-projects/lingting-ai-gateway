//! 转发前置：取供应商、写主 / 子请求日志、装配回调后执行具体转发。

use std::future::Future;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use lib_provider::{ForwardCallback, ForwardOutcome, RouteRequest};
use lib_web_core::{
    WebCancelToken, WebResponse, use_app, use_authorization, use_web, use_web_cancel,
};
use service_admin::service::provider_service::ProviderService;
use service_admin::service::request_service::RequestService;
use types_admin::dto::{RequestMainCreatePO, RequestSubCreatePO};
use types_admin::entity::Provider;

use super::callback::OpenAiForwardCallback;
use super::utils;

/// 按模型取优先级最高的供应商。
pub async fn first_provider(model: &str) -> Result<Provider> {
    ProviderService::new()?
        .first_by_model(model)
        .await?
        .ok_or_else(|| anyhow!("没有匹配到支持模型 {} 的供应商", model))
}

/// 包裹一次转发：写主请求日志、子请求日志，装配回调，再执行传入的转发实例。
pub async fn with_wrapper<F, Fut>(
    provider: Provider,
    request: RouteRequest,
    provider_url: String,
    executor: F,
) -> Result<WebResponse>
where
    F: FnOnce(Provider, RouteRequest, Arc<dyn ForwardCallback>) -> Fut,
    Fut: Future<Output = Result<WebResponse>>,
{
    let web_context = use_web()?;
    let app_context = use_app()?;
    let authorization = use_authorization()?;
    let request_service = RequestService::new()?;

    let debug_mode = app_context.debug_mode();
    // 主请求的收到时间取自框架的请求快照，早于鉴权与供应商查询。
    let receive_time = web_context.request().receive_time;
    let request_params = utils::request_params(&web_context, debug_mode);
    let request_headers = utils::request_headers(&web_context, debug_mode);
    let forwarded_headers = utils::forwarded_headers(&web_context, &provider.api_key, debug_mode)?;

    // 主请求日志
    // trace_id 由框架生成，网关直接取请求快照中的值写入日志。
    let trace_id = web_context.request().trace_id.clone();
    let client_request_id = utils::header(&web_context, "x-client-request-id")
        .unwrap_or_else(|| web_context.request().trace_id.clone());

    let main_request_id = request_service
        .create_main(&RequestMainCreatePO {
            trace_id,
            client_request_id,
            session_id: request.session_id.clone().unwrap_or_default(),
            client_ip: web_context.request().client_ip.clone().unwrap_or_default(),
            user_agent: utils::header(&web_context, "user-agent").unwrap_or_default(),
            credential_id: authorization.credential_id().to_string(),
            model: request.model.clone(),
            stream: request.stream,
            method: web_context.request().method.to_string(),
            path: web_context.request().path.clone(),
            request_params: request_params.clone(),
            request_headers,
            start_time: receive_time,
        })
        .await?;

    // 子请求日志
    let sub_request_id = request_service
        .create_sub(&RequestSubCreatePO {
            main_request_id,
            provider_id: provider.id,
            provider_name: provider.name.clone(),
            model: request.model.clone(),
            provider_url,
            request_params,
            request_headers: forwarded_headers,
            start_time: receive_time,
        })
        .await?;

    // 转发回调
    let callback: Arc<dyn ForwardCallback> = Arc::new(OpenAiForwardCallback::new(
        request_service,
        main_request_id,
        sub_request_id,
        debug_mode,
        request.stream,
    ));
    tracing::debug!(
        "[MOCKTEST] forward-start provider={} model={} stream={} main={main_request_id} sub={sub_request_id}",
        provider.name,
        request.model,
        request.stream
    );

    // 客户端断开时取消上游转发，并补写请求日志收尾。
    // 走取消分支时 executor 的 future 被 drop，reqwest 随之关闭上游连接。
    let cancel_token = use_web_cancel()?;
    let mut cancelled = cancel_token.subscribe();
    let cancel_callback = Arc::clone(&callback);

    let response = tokio::select! {
            response = executor(provider, request, callback) => {
    #[cfg(debug_assertions)]
    tracing::debug!("[MOCKTEST]  forward-return main={main_request_id} sub={sub_request_id}");
                response
            }
            _ = WebCancelToken::wait_cancelled(&mut cancelled) => {
    #[cfg(debug_assertions)]
    tracing::debug!("[MOCKTEST]  forward-cancel main={main_request_id} sub={sub_request_id}");
                // 取消时的日志收尾失败只记录，不再向上传播。
                if let Err(error) = cancel_callback
                    .on_cancel(&ForwardOutcome::default())
                    .await
                {
                    tracing::warn!("取消回调执行失败：{error:#}");
                }
                return Err(anyhow!("客户端已断开"));
            }
        };

    response
}
