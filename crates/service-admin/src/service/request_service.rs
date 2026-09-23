use anyhow::Result;
use framework_core::types::{PaginationParams, PaginationResult};
use lib_db::use_pool;
use types_admin::dto::{
    DashboardQO, DashboardRequestVO, DashboardTokenGroup, DashboardTokenVO, RequestFinishPO,
    RequestMainCreatePO, RequestMainQO, RequestSubCreatePO, RequestSubQO,
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

    /// 记录子请求日志的转发开始时间。
    pub async fn update_sub_start_time(&self, id: i64, start_time: i64) -> Result<()> {
        self.sub_repository.update_start_time(id, start_time).await
    }

    /// 记录子请求日志的首字时间。
    pub async fn update_sub_first_chunk_time(&self, id: i64, first_chunk_time: i64) -> Result<()> {
        self.sub_repository
            .update_first_chunk_time(id, first_chunk_time)
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

    /// 按筛选条件统计主请求日志的请求数量。
    pub async fn dashboard_main_request(
        &self,
        conditions: &DashboardQO,
    ) -> Result<DashboardRequestVO> {
        self.main_repository.dashboard_request(conditions).await
    }

    /// 按筛选条件统计主请求日志的 token。
    pub async fn dashboard_main_token(&self, conditions: &DashboardQO) -> Result<DashboardTokenVO> {
        self.main_repository.dashboard_token(conditions).await
    }

    /// 按筛选条件统计子请求日志的请求数量。
    pub async fn dashboard_sub_request(
        &self,
        conditions: &DashboardQO,
    ) -> Result<DashboardRequestVO> {
        self.sub_repository.dashboard_request(conditions).await
    }

    /// 按筛选条件统计子请求日志的 token。
    pub async fn dashboard_sub_token(&self, conditions: &DashboardQO) -> Result<DashboardTokenVO> {
        self.sub_repository.dashboard_token(conditions).await
    }

    /// 按筛选条件统计子请求日志的 token，并按天（可再按供应商、模型）分组。
    pub async fn dashboard_sub_token_group(
        &self,
        conditions: &DashboardQO,
    ) -> Result<Vec<DashboardTokenGroup>> {
        self.sub_repository.dashboard_token_group(conditions).await
    }
}
