use anyhow::Result;
use framework_core::types::{PaginationParams, PaginationResult};
use lib_db::use_pool;
use types_admin::dto::{
    RequestFinishPO, RequestMainCreatePO, RequestMainQO, RequestSubCreatePO, RequestSubQO,
};
use types_admin::entity::{RequestMain, RequestSub};

use crate::repository::{RequestMainRepository, RequestSubRepository};

/// 请求日志业务逻辑。
pub struct RequestService {
    main_repository: RequestMainRepository,
    sub_repository: RequestSubRepository,
}

impl RequestService {
    /// 绑定当前请求的数据库连接池。
    pub fn new() -> Result<Self> {
        let pool = use_pool()?;
        Ok(Self {
            main_repository: RequestMainRepository::new(pool.clone()),
            sub_repository: RequestSubRepository::new(pool),
        })
    }

    /// 分页查询主请求日志。
    pub async fn main_page(
        &self,
        pagination: &PaginationParams,
        conditions: &RequestMainQO,
    ) -> Result<PaginationResult<RequestMain>> {
        self.main_repository.page(pagination, conditions).await
    }

    /// 分页查询子请求日志。
    pub async fn sub_page(
        &self,
        pagination: &PaginationParams,
        conditions: &RequestSubQO,
    ) -> Result<PaginationResult<RequestSub>> {
        self.sub_repository.page(pagination, conditions).await
    }

    /// 按主键查询主请求日志。
    pub async fn find_main_by_id(&self, id: i64) -> Result<Option<RequestMain>> {
        self.main_repository.find_by_id(id).await
    }

    /// 查询单条主请求日志下的全部子请求日志。
    pub async fn find_sub_by_main(&self, main_request_id: i64) -> Result<Vec<RequestSub>> {
        self.sub_repository
            .find_by_main_request_id(main_request_id)
            .await
    }

    /// 批量查询多条主请求日志下的子请求日志。
    pub async fn find_sub_by_mains(&self, main_request_ids: &[i64]) -> Result<Vec<RequestSub>> {
        self.sub_repository
            .find_by_main_request_ids(main_request_ids)
            .await
    }

    /// 写入主请求日志，返回主键。
    pub async fn create_main(&self, params: &RequestMainCreatePO) -> Result<i64> {
        self.main_repository.create(params).await
    }

    /// 写入子请求日志，返回主键。
    pub async fn create_sub(&self, params: &RequestSubCreatePO) -> Result<i64> {
        self.sub_repository.create(params).await
    }

    /// 更新主请求日志的处理进度描述。
    pub async fn update_main_progress(&self, id: i64, current_status: &str) -> Result<()> {
        self.main_repository
            .update_current_status(id, current_status)
            .await
    }

    /// 更新主请求日志命中的供应商数量。
    pub async fn update_main_provider_count(&self, id: i64, provider_count: i32) -> Result<()> {
        self.main_repository
            .update_provider_count(id, provider_count)
            .await
    }

    /// 结束主请求日志。
    pub async fn finish_main(&self, id: i64, params: &RequestFinishPO) -> Result<()> {
        self.main_repository.finish(id, params).await
    }

    /// 结束子请求日志。
    pub async fn finish_sub(&self, id: i64, params: &RequestFinishPO) -> Result<()> {
        self.sub_repository.finish(id, params).await
    }
}
