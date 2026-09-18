use anyhow::Result;
use framework_core::types::{PaginationParams, PaginationResult};
use lib_db::use_pool;
use types_admin::dto::{
    ProviderModelRedirectCreatePO, ProviderModelRedirectQO, ProviderModelRedirectUpdatePO,
};
use types_admin::entity::ProviderModelRedirect;

use crate::repository::ProviderModelRedirectRepository;

/// 模型名称映射业务逻辑。
pub struct ProviderModelRedirectService {
    repository: ProviderModelRedirectRepository,
}

impl ProviderModelRedirectService {
    /// 绑定当前请求的数据库连接池。
    pub fn new() -> Result<Self> {
        Ok(Self {
            repository: ProviderModelRedirectRepository::new(use_pool()?),
        })
    }

    /// 分页查询模型名称映射。
    pub async fn page(
        &self,
        pagination: &PaginationParams,
        conditions: &ProviderModelRedirectQO,
    ) -> Result<PaginationResult<ProviderModelRedirect>> {
        self.repository.page(pagination, conditions).await
    }

    /// 按主键查询模型名称映射。
    pub async fn find_by_id(&self, id: i64) -> Result<Option<ProviderModelRedirect>> {
        self.repository.find_by_id(id).await
    }

    /// 按非官方模型名查询映射。
    pub async fn find_by_source(&self, source: &str) -> Result<Option<ProviderModelRedirect>> {
        self.repository.find_by_source(source).await
    }

    /// 查询全部模型名称映射；模型同步任务使用。
    pub async fn find_all(&self) -> Result<Vec<ProviderModelRedirect>> {
        self.repository.find_all().await
    }
    /// 新增模型名称映射。
    pub async fn create(&self, params: &ProviderModelRedirectCreatePO) -> Result<i64> {
        self.repository.create(params).await
    }

    /// 更新模型名称映射。
    pub async fn update(&self, params: &ProviderModelRedirectUpdatePO) -> Result<()> {
        self.repository.update(params).await
    }

    /// 删除模型名称映射。
    pub async fn delete(&self, id: i64) -> Result<()> {
        self.repository.delete(id).await
    }
}
