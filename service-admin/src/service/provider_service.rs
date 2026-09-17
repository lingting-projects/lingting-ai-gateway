use anyhow::Result;
use framework_core::types::{PaginationParams, PaginationResult};
use types_admin::dto::{ProviderCreatePO, ProviderQO, ProviderUpdatePO};
use types_admin::entity::Provider;

use super::pool;
use crate::repository::ProviderRepository;

/// 供应商业务逻辑。
pub struct ProviderService;

impl ProviderService {
    /// 分页查询供应商。
    pub async fn page(
        pagination: &PaginationParams,
        conditions: &ProviderQO,
    ) -> Result<PaginationResult<Provider>> {
        ProviderRepository::new(pool()?)
            .page(pagination, conditions)
            .await
    }

    /// 查询全部启用的供应商。
    pub async fn find_enabled() -> Result<Vec<Provider>> {
        ProviderRepository::new(pool()?).find_enabled().await
    }

    /// 按主键查询供应商。
    pub async fn find_by_id(id: i64) -> Result<Option<Provider>> {
        ProviderRepository::new(pool()?).find_by_id(id).await
    }

    /// 查询支持指定模型的供应商，按优先级升序、创建时间升序排列。
    pub async fn find_by_model(model: &str) -> Result<Vec<Provider>> {
        ProviderRepository::new(pool()?)
            .find_enabled_by_model(model)
            .await
    }

    /// 新增供应商。
    pub async fn create(params: &ProviderCreatePO) -> Result<i64> {
        ProviderRepository::new(pool()?).create(params).await
    }

    /// 更新供应商。
    pub async fn update(params: &ProviderUpdatePO) -> Result<()> {
        ProviderRepository::new(pool()?).update(params).await
    }
}
