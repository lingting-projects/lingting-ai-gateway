use anyhow::Result;
use framework_core::types::{PaginationParams, PaginationResult};
use lib_db::use_pool;
use types_admin::dto::{ProviderCreatePO, ProviderQO, ProviderUpdatePO};
use types_admin::entity::Provider;

use crate::repository::ProviderRepository;

/// 供应商业务逻辑。
pub struct ProviderService {
    repository: ProviderRepository,
}

impl ProviderService {
    /// 绑定当前请求的数据库连接池。
    pub fn new() -> Result<Self> {
        Ok(Self {
            repository: ProviderRepository::new(use_pool()?),
        })
    }

    /// 分页查询供应商。
    pub async fn page(
        &self,
        pagination: &PaginationParams,
        conditions: &ProviderQO,
    ) -> Result<PaginationResult<Provider>> {
        self.repository.page(pagination, conditions).await
    }

    /// 查询全部启用的供应商。
    pub async fn find_enabled(&self) -> Result<Vec<Provider>> {
        self.repository.find_enabled().await
    }

    /// 按主键查询供应商。
    pub async fn find_by_id(&self, id: i64) -> Result<Option<Provider>> {
        self.repository.find_by_id(id).await
    }

    /// 查询支持指定模型的第一个供应商，按优先级升序、创建时间升序取一个。
    pub async fn first_by_model(&self, model: &str) -> Result<Option<Provider>> {
        self.repository.first_enabled_by_model(model).await
    }

    /// 新增供应商。
    pub async fn create(&self, params: &ProviderCreatePO) -> Result<i64> {
        self.repository.create(params).await
    }

    /// 更新供应商。
    pub async fn update(&self, params: &ProviderUpdatePO) -> Result<()> {
        self.repository.update(params).await
    }

    /// 逻辑删除供应商。
    pub async fn delete(&self, id: i64) -> Result<()> {
        self.repository.delete(id).await
    }
}
