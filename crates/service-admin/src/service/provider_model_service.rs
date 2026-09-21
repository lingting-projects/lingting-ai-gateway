use anyhow::Result;
use framework_core::types::{PaginationParams, PaginationResult};
use lib_db::{PgTransaction, use_pool};
use types_admin::dto::{ProviderModelCreatePO, ProviderModelQO, ProviderModelUpdatePO};
use types_admin::entity::ProviderModel;

use crate::repository::ProviderModelRepository;

/// 供应商模型业务逻辑。
pub struct ProviderModelService {
    repository: ProviderModelRepository,
}

impl ProviderModelService {
    /// 绑定当前请求的数据库连接池。
    pub fn new() -> Result<Self> {
        Ok(Self {
            repository: ProviderModelRepository::new(use_pool()?),
        })
    }

    /// 分页查询供应商模型。
    pub async fn page(
        &self,
        pagination: &PaginationParams,
        conditions: &ProviderModelQO,
    ) -> Result<PaginationResult<ProviderModel>> {
        self.repository.page(pagination, conditions).await
    }

    /// 查询指定供应商的全部模型。
    pub async fn find_by_provider(&self, provider_id: i64) -> Result<Vec<ProviderModel>> {
        self.repository.find_by_provider_id(provider_id).await
    }

    /// 批量查询多个供应商的模型。
    pub async fn find_by_providers(&self, provider_ids: &[i64]) -> Result<Vec<ProviderModel>> {
        self.repository.find_by_provider_ids(provider_ids).await
    }

    /// 查询全部启用的模型。
    pub async fn find_enabled(&self) -> Result<Vec<ProviderModel>> {
        self.repository.find_enabled().await
    }

    /// 查询全部启用且按名称去重的模型，同名取路由优先级最高的一条；用于可用模型列表。
    pub async fn find_enabled_distinct(&self) -> Result<Vec<ProviderModel>> {
        self.repository.find_enabled_distinct().await
    }

    /// 查询全部启用且去重后的模型名称。
    pub async fn find_enabled_names(&self) -> Result<Vec<String>> {
        self.repository.find_enabled_names().await
    }

    /// 按供应商与模型名查询。
    pub async fn find_by_provider_and_model(
        &self,
        provider_id: i64,
        model: &str,
    ) -> Result<Option<ProviderModel>> {
        self.repository
            .find_by_provider_and_model(provider_id, model)
            .await
    }

    /// 查询默认模型数据；默认数据仅作为模型基础配置，不参与路由与展示。
    pub async fn find_default(&self) -> Result<Vec<ProviderModel>> {
        self.repository.find_default().await
    }

    /// 查询指定供应商已入库的模型名称；模型同步任务使用。
    pub async fn find_models(&self, provider_id: i64) -> Result<Vec<String>> {
        self.repository.find_models(provider_id).await
    }

    /// 批量写入或更新供应商模型，用于模型同步任务。
    pub async fn upsert(
        &self,
        params: &[ProviderModelCreatePO],
        transaction: &mut PgTransaction<'_>,
    ) -> Result<()> {
        self.repository.upsert(params, transaction).await
    }

    /// 关闭供应商已不再返回的模型。
    pub async fn disable_missing(
        &self,
        provider_id: i64,
        models: &[String],
        transaction: &mut PgTransaction<'_>,
    ) -> Result<u64> {
        self.repository
            .disable_missing(provider_id, models, transaction)
            .await
    }

    /// 关闭供应商全部已启用模型；远程返回空模型列表时使用。
    pub async fn disable_all(
        &self,
        provider_id: i64,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<u64> {
        self.repository.disable_all(provider_id, transaction).await
    }

    /// 更新供应商模型的可编辑字段。
    pub async fn update(&self, params: &ProviderModelUpdatePO) -> Result<()> {
        self.repository.update(params).await
    }
}
