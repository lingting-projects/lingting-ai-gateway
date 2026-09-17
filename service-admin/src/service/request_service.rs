use anyhow::Result;
use framework_core::types::{PaginationParams, PaginationResult};
use types_admin::dto::{
    RequestFinishPO, RequestMainCreatePO, RequestMainQO, RequestSubCreatePO, RequestSubQO,
};
use types_admin::entity::{RequestMain, RequestSub};

use super::pool;
use crate::repository::{RequestMainRepository, RequestSubRepository};

/// 请求日志业务逻辑。
pub struct RequestService;

impl RequestService {
    /// 分页查询主请求日志。
    pub async fn main_page(
        pagination: &PaginationParams,
        conditions: &RequestMainQO,
    ) -> Result<PaginationResult<RequestMain>> {
        RequestMainRepository::new(pool()?)
            .page(pagination, conditions)
            .await
    }

    /// 分页查询子请求日志。
    pub async fn sub_page(
        pagination: &PaginationParams,
        conditions: &RequestSubQO,
    ) -> Result<PaginationResult<RequestSub>> {
        RequestSubRepository::new(pool()?)
            .page(pagination, conditions)
            .await
    }

    /// 按主键查询主请求日志。
    pub async fn find_main_by_id(id: i64) -> Result<Option<RequestMain>> {
        RequestMainRepository::new(pool()?).find_by_id(id).await
    }

    /// 查询单条主请求日志下的全部子请求日志。
    pub async fn find_sub_by_main(main_request_id: i64) -> Result<Vec<RequestSub>> {
        RequestSubRepository::new(pool()?)
            .find_by_main_request_id(main_request_id)
            .await
    }

    /// 批量查询多条主请求日志下的子请求日志。
    pub async fn find_sub_by_mains(main_request_ids: &[i64]) -> Result<Vec<RequestSub>> {
        RequestSubRepository::new(pool()?)
            .find_by_main_request_ids(main_request_ids)
            .await
    }

    /// 写入主请求日志，返回主键。
    pub async fn create_main(params: &RequestMainCreatePO) -> Result<i64> {
        RequestMainRepository::new(pool()?).create(params).await
    }

    /// 写入子请求日志，返回主键。
    pub async fn create_sub(params: &RequestSubCreatePO) -> Result<i64> {
        RequestSubRepository::new(pool()?).create(params).await
    }

    /// 更新主请求日志的处理进度描述。
    pub async fn update_main_progress(id: i64, current_status: &str) -> Result<()> {
        RequestMainRepository::new(pool()?)
            .update_current_status(id, current_status)
            .await
    }

    /// 更新主请求日志命中的供应商数量。
    pub async fn update_main_provider_count(id: i64, provider_count: i32) -> Result<()> {
        RequestMainRepository::new(pool()?)
            .update_provider_count(id, provider_count)
            .await
    }

    /// 结束主请求日志。
    pub async fn finish_main(id: i64, params: &RequestFinishPO) -> Result<()> {
        RequestMainRepository::new(pool()?).finish(id, params).await
    }

    /// 结束子请求日志。
    pub async fn finish_sub(id: i64, params: &RequestFinishPO) -> Result<()> {
        RequestSubRepository::new(pool()?).finish(id, params).await
    }
}
