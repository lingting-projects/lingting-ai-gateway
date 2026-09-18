//! 聊天接口：取供应商、写请求日志、装配回调后执行具体转发。

use std::future::Future;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use lib_provider::{ChatRequest, ForwardCallback};
use lib_web_core::{WebResponse, use_app, use_authorization, use_web};
use service_admin::service::provider_service::ProviderService;
use service_admin::service::request_service::RequestService;
use types_admin::dto::{RequestMainCreatePO, RequestSubCreatePO};
use types_admin::entity::Provider;

use super::OpenaiService;
use super::call_chat::call_chat;
use super::call_chat_stream::call_chat_stream;
use super::callback::OpenAiForwardCallback;
use super::utils;

impl OpenaiService {
    /// 聊天接口：按模型取优先级最高的供应商，普通请求走 `call_chat`，
    /// 流式请求走 `call_chat_stream`。
    pub async fn chat(request: ChatRequest) -> Result<WebResponse> {
        let provider = ProviderService::new()?
            .first_by_model(&request.model)
            .await?
            .ok_or_else(|| anyhow!("没有匹配到支持模型 {} 的供应商", request.model))?;

        if request.is_stream() {
            with_wrapper(provider, request, call_chat_stream).await
        } else {
            with_wrapper(provider, request, call_chat).await
        }
    }
}

/// 包裹一次转发：写主请求日志、子请求日志，装配回调，再执行传入的转发实例。
async fn with_wrapper<F, Fut>(
    provider: Provider,
    request: ChatRequest,
    executor: F,
) -> Result<WebResponse>
where
    F: FnOnce(Provider, ChatRequest, Arc<dyn ForwardCallback>) -> Fut,
    Fut: Future<Output = Result<WebResponse>>,
{
    let web_context = use_web()?;
    let app_context = use_app()?;
    let authorization = use_authorization()?;
    let request_service = RequestService::new()?;

    let debug_mode = app_context.debug_mode();
    let start_time = lib_core::current_millis()?;
    let request_params = utils::request_params(&request, debug_mode);

    // 主请求日志
    let request_id = lib_core::next_id()?.to_string();
    let trace_id = utils::header(&web_context, "x-trace-id").unwrap_or_else(|| request_id.clone());
    let client_request_id = utils::header(&web_context, "x-request-id")
        .unwrap_or_else(|| web_context.request().request_id.clone());

    let main_request_id = request_service
        .create_main(&RequestMainCreatePO {
            request_id,
            trace_id,
            client_request_id,
            session_id: request.session_id().unwrap_or_default(),
            client_ip: web_context.request().client_ip.clone().unwrap_or_default(),
            user_agent: utils::header(&web_context, "user-agent").unwrap_or_default(),
            credential_id: authorization.credential_id().to_string(),
            model: request.model.clone(),
            stream: request.is_stream(),
            method: web_context.request().method.to_string(),
            path: web_context.request().path.clone(),
            request_params: request_params.clone(),
            start_time,
        })
        .await?;

    // 子请求日志
    let sub_request_id = request_service
        .create_sub(&RequestSubCreatePO {
            main_request_id,
            provider_id: provider.id,
            provider_name: provider.name.clone(),
            model: request.model.clone(),
            provider_url: utils::chat_url(&provider),
            request_params,
            start_time,
        })
        .await?;

    // 转发回调
    let callback: Arc<dyn ForwardCallback> = Arc::new(OpenAiForwardCallback::new(
        request_service,
        main_request_id,
        sub_request_id,
        debug_mode,
        start_time,
    ));

    executor(provider, request, callback).await
}
