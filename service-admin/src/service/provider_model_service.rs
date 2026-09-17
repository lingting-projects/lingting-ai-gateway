use anyhow::Result;
use framework_core::types::{PaginationParams, PaginationResult};
use types_admin::dto::{ProviderModelQO, ProviderModelUpdatePO};
use types_admin::entity::{InferenceLevel, ProviderModel};

use super::pool;
use crate::repository::ProviderModelRepository;

/// 供应商模型业务逻辑。
pub struct ProviderModelService;

impl ProviderModelService {
    /// 分页查询供应商模型。
    pub async fn page(
        pagination: &PaginationParams,
        conditions: &ProviderModelQO,
    ) -> Result<PaginationResult<ProviderModel>> {
        ProviderModelRepository::new(pool()?)
            .page(pagination, conditions)
            .await
    }

    /// 查询指定供应商的全部模型。
    pub async fn find_by_provider(provider_id: i64) -> Result<Vec<ProviderModel>> {
        ProviderModelRepository::new(pool()?)
            .find_by_provider_id(provider_id)
            .await
    }

    /// 查询全部启用的模型。
    pub async fn find_enabled() -> Result<Vec<ProviderModel>> {
        ProviderModelRepository::new(pool()?).find_enabled().await
    }

    /// 查询全部启用且去重后的模型名称。
    pub async fn find_enabled_names() -> Result<Vec<String>> {
        ProviderModelRepository::new(pool()?)
            .find_enabled_names()
            .await
    }

    /// 按供应商与模型名查询。
    pub async fn find_by_provider_and_model(
        provider_id: i64,
        model: &str,
    ) -> Result<Option<ProviderModel>> {
        ProviderModelRepository::new(pool()?)
            .find_by_provider_and_model(provider_id, model)
            .await
    }

    /// 写入或更新供应商模型，用于模型同步任务。
    pub async fn upsert(
        provider_id: i64,
        model: &str,
        display_name: &str,
        inference_level: &InferenceLevel,
    ) -> Result<()> {
        ProviderModelRepository::new(pool()?)
            .upsert(provider_id, model, display_name, inference_level)
            .await
    }

    /// 关闭供应商已不再返回的模型。
    pub async fn disable_missing(provider_id: i64, models: &[String]) -> Result<u64> {
        ProviderModelRepository::new(pool()?)
            .disable_missing(provider_id, models)
            .await
    }

    /// 更新供应商模型的可编辑字段。
    pub async fn update(params: &ProviderModelUpdatePO) -> Result<()> {
        ProviderModelRepository::new(pool()?).update(params).await
    }
}
