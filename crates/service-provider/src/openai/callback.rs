//! 转发回调：把一次转发的进度与结果写回主请求日志和子请求日志。

use anyhow::Result;
use async_trait::async_trait;
use lib_provider::{ForwardCallback, ForwardFailure, ForwardOutcome};
use service_admin::service::request_service::RequestService;
use types_admin::dto::RequestFinishPO;
use types_admin::entity::RequestStatus;

use super::utils;

/// 把一次转发的进度与结果写回主请求日志和子请求日志。
pub struct OpenAiForwardCallback {
    request_service: RequestService,
    main_request_id: i64,
    sub_request_id: i64,
    debug_mode: bool,
    start_time: i64,
}

impl OpenAiForwardCallback {
    /// 绑定请求日志服务与本次转发的主、子请求日志主键。
    pub fn new(
        request_service: RequestService,
        main_request_id: i64,
        sub_request_id: i64,
        debug_mode: bool,
        start_time: i64,
    ) -> Self {
        Self {
            request_service,
            main_request_id,
            sub_request_id,
            debug_mode,
            start_time,
        }
    }

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
            response_headers: utils::response_headers(
                outcome.response_headers.as_ref(),
                self.debug_mode,
            ),
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

    /// 客户端取消请求：按取消状态结束日志，取消前已累积的用量与内容一并写入。
    async fn on_cancel(&self, outcome: &ForwardOutcome) -> Result<()> {
        let params = self.finish_params(RequestStatus::Cancelled, outcome, None)?;
        self.finish(&params).await
    }

    fn is_debug(&self) -> bool {
        self.debug_mode
    }
}
