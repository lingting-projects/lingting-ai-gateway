use anyhow::Result;
use framework_core::types::{PaginationParams, PaginationResult};
use lib_db::use_pool;
use types_admin::dto::{ProviderModelQO, ProviderModelUpdatePO};
use types_admin::entity::{InferenceLevel, ProviderModel};

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

    /// 查询全部启用的模型。
    pub async fn find_enabled(&self) -> Result<Vec<ProviderModel>> {
        self.repository.find_enabled().await
    }

    /// 查询全部启用且按名称去重的模型，同名取 id 最大的一条；用于可用模型列表。
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

    /// 写入或更新供应商模型，用于模型同步任务。
    pub async fn upsert(
        &self,
        provider_id: i64,
        model: &str,
        display_name: &str,
        inference_level: &InferenceLevel,
    ) -> Result<()> {
        self.repository
            .upsert(provider_id, model, display_name, inference_level)
            .await
    }

    /// 关闭供应商已不再返回的模型。
    pub async fn disable_missing(&self, provider_id: i64, models: &[String]) -> Result<u64> {
        self.repository.disable_missing(provider_id, models).await
    }

    /// 更新供应商模型的可编辑字段。
    pub async fn update(&self, params: &ProviderModelUpdatePO) -> Result<()> {
        self.repository.update(params).await
    }
}
