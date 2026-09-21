//! 转发回调：把一次转发的进度与结果写回主请求日志和子请求日志。

use std::sync::atomic::{AtomicI64, Ordering};

use anyhow::Result;
use async_trait::async_trait;
use lib_provider::{ForwardCallback, ForwardFailure, ForwardOutcome};
use service_admin::service::request_service::RequestService;
use types_admin::dto::RequestFinishPO;
use types_admin::entity::RequestStatus;

use super::utils;

/// 首字时间尚未产生的标记值。
const NO_FIRST_CHUNK: i64 = 0;

/// 把一次转发的进度与结果写回主请求日志和子请求日志。
pub struct OpenAiForwardCallback {
    request_service: RequestService,
    main_request_id: i64,
    sub_request_id: i64,
    debug_mode: bool,
    /// 是否流式请求；非流式没有分片回调，首字时间取完成时间。
    stream: bool,
    /// 子请求首字时间，`NO_FIRST_CHUNK` 表示尚未产生分片。
    first_chunk_time: AtomicI64,
}

impl OpenAiForwardCallback {
    /// 绑定请求日志服务与本次转发的主、子请求日志主键。
    pub fn new(
        request_service: RequestService,
        main_request_id: i64,
        sub_request_id: i64,
        debug_mode: bool,
        stream: bool,
    ) -> Self {
        Self {
            request_service,
            main_request_id,
            sub_request_id,
            debug_mode,
            stream,
            first_chunk_time: AtomicI64::new(NO_FIRST_CHUNK),
        }
    }

    /// 子请求首字时间：已产生分片时取首个分片时刻，非流式取完成时间，未产生分片时为空。
    fn first_chunk_time(&self, end_time: i64) -> Option<i64> {
        match self.first_chunk_time.load(Ordering::Acquire) {
            NO_FIRST_CHUNK if self.stream => None,
            NO_FIRST_CHUNK => Some(end_time),
            recorded => Some(recorded),
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
            first_chunk_time: self.first_chunk_time(end_time),
            end_time,
        })
    }

    /// 同时结束子请求日志与主请求日志。
    async fn finish(&self, params: &RequestFinishPO) -> Result<()> {
        tracing::debug!(
            "[MOCKTEST] finish status={} main={} sub={}",
            params.status.as_str(),
            self.main_request_id,
            self.sub_request_id
        );

        // 子日志先写：失败即中断，主日志也不会被标记为已完成，
        // 避免出现「主日志正常、子日志错误」这种不易察觉的状态。
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
    /// 请求即将发出：记录子请求的转发开始时间。
    async fn on_start(&self) -> Result<()> {
        let start_time = lib_core::current_millis()?;
        tracing::debug!("[MOCKTEST] upstream-start sub={}", self.sub_request_id);
        self.request_service
            .update_sub_start_time(self.sub_request_id, start_time)
            .await
    }

    /// 收到新的分片：首次调用时记录首字时间。
    async fn on_progress(&self, _outcome: &ForwardOutcome) -> Result<()> {
        let first_chunk_time = lib_core::current_millis()?;
        let previous = self
            .first_chunk_time
            .swap(first_chunk_time, Ordering::AcqRel);
        if previous != NO_FIRST_CHUNK {
            return Ok(());
        }

        self.request_service
            .update_sub_first_chunk_time(self.sub_request_id, first_chunk_time)
            .await
    }

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
