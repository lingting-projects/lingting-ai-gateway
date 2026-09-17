//! 聊天接口：取供应商、写请求日志、装配回调后执行具体转发。

use std::future::Future;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use lib_provider::{ChatRequest, ForwardCallback, ForwardFailure, ForwardOutcome};
use lib_web_core::{WebResponse, use_app, use_authorization, use_web};
use service_admin::service::provider_service::ProviderService;
use service_admin::service::request_service::RequestService;
use types_admin::dto::{RequestFinishPO, RequestMainCreatePO, RequestSubCreatePO};
use types_admin::entity::{Provider, RequestStatus};

use super::OpenaiService;
use super::call_chat::call_chat;
use super::call_chat_stream::call_chat_stream;
use super::utils;

impl OpenaiService {
    /// 聊天接口：按模型取优先级最高的供应商，普通请求走 `call_chat`，
    /// 流式请求走 `call_chat_stream`。
    pub async fn chat(request: ChatRequest) -> Result<WebResponse> {
        let provider = first_provider(&request.model).await?;

        if request.is_stream() {
            with_wrapper(provider, request, call_chat_stream).await
        } else {
            with_wrapper(provider, request, call_chat).await
        }
    }
}

/// 取该模型下优先级最高的供应商。
async fn first_provider(model: &str) -> Result<Provider> {
    ProviderService::new()?
        .find_by_model(model)
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("没有匹配到支持模型 {model} 的供应商"))
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
    let callback: Arc<dyn ForwardCallback> = Arc::new(OpenAiForwardCallback {
        request_service,
        main_request_id,
        sub_request_id,
        debug_mode,
        start_time,
    });

    executor(provider, request, callback).await
}

/// 把一次转发的进度与结果写回主请求日志和子请求日志。
struct OpenAiForwardCallback {
    request_service: RequestService,
    main_request_id: i64,
    sub_request_id: i64,
    debug_mode: bool,
    start_time: i64,
}

impl OpenAiForwardCallback {
    /// 组装请求结束数据；失败时额外带上异常信息。
    fn finish_params(
        &self,
        status: RequestStatus,
        outcome: &ForwardOutcome,
        failure: Option<&ForwardFailure>,
    ) -> Result<RequestFinishPO> {
        let end_time = lib_core::current_millis()?;
        let token_info = &outcome.token_info;

        Ok(RequestFinishPO {
            status,
            error_type: failure.map(|item| item.error_type.clone()),
            error_code: failure.map(|item| item.error_code.clone()),
            error_message: failure.map(|item| item.message.clone()),
            return_model: Some(outcome.return_model.clone()),
            input_tokens: token_info.input_tokens,
            output_tokens: token_info.output_tokens,
            cache_read_tokens: token_info.cache_read_tokens,
            cache_write_tokens: token_info.cache_write_tokens,
            inference_tokens: token_info.inference_tokens,
            read_tokens: token_info.read_tokens,
            write_tokens: token_info.write_tokens,
            total_tokens: token_info.total_tokens,
            http_status: outcome.http_status,
            finish_reason: Some(outcome.finish_reason.clone()),
            provider_request_id: Some(outcome.provider_request_id.clone()),
            response_content: utils::response_content(&outcome.content, self.debug_mode),
            end_time,
            duration_ms: end_time - self.start_time,
        })
    }

    /// 同时结束子请求日志与主请求日志。
    async fn finish(&self, params: &RequestFinishPO) -> Result<()> {
        self.request_service
            .finish_sub(self.sub_request_id, params)
            .await?;
        self.request_service
            .finish_main(self.main_request_id, params)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl ForwardCallback for OpenAiForwardCallback {
    async fn on_success(&self, outcome: &ForwardOutcome) -> Result<()> {
        let params = self.finish_params(RequestStatus::Success, outcome, None)?;
        self.finish(&params).await
    }

    async fn on_failure(&self, failure: &ForwardFailure) -> Result<()> {
        let params = self.finish_params(RequestStatus::Failed, &failure.outcome, Some(failure))?;
        self.finish(&params).await
    }
}
