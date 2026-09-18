use std::collections::HashMap;

use anyhow::Result;
use framework_core::types::{PaginationParams, PaginationResult};
use types_admin::dto::{ProviderDetailVO, ProviderModelVO, ProviderQO, ProviderVO};

use crate::service::{ProviderModelService, ProviderService};

/// 供应商跨表编排：供应商分页并嵌入各自的模型。
pub struct ProviderManager {
    provider_service: ProviderService,
    provider_model_service: ProviderModelService,
}

impl ProviderManager {
    /// 绑定当前请求的数据库连接池。
    pub fn new() -> Result<Self> {
        Ok(Self {
            provider_service: ProviderService::new()?,
            provider_model_service: ProviderModelService::new()?,
        })
    }

    /// 供应商分页，附带每个供应商下的全部模型。
    ///
    /// 先查供应商拿到主键集合，再一次性查出全部模型后按供应商归组，避免逐条查询。
    pub async fn page_detail(
        &self,
        pagination: &PaginationParams,
        conditions: &ProviderQO,
    ) -> Result<PaginationResult<ProviderDetailVO>> {
        let page = self.provider_service.page(pagination, conditions).await?;
        let ids: Vec<i64> = page.records.iter().map(|provider| provider.id).collect();
        let names: HashMap<i64, String> = page
            .records
            .iter()
            .map(|provider| (provider.id, provider.name.clone()))
            .collect();

        let models = if ids.is_empty() {
            Vec::new()
        } else {
            self.provider_model_service.find_by_providers(&ids).await?
        };

        let mut grouped: HashMap<i64, Vec<ProviderModelVO>> = HashMap::new();
        for model in models {
            let provider_id = model.provider_id;
            let provider_name = names.get(&provider_id).cloned().unwrap_or_default();
            grouped
                .entry(provider_id)
                .or_default()
                .push(ProviderModelVO::of(model, provider_name));
        }

        let records = page
            .records
            .into_iter()
            .map(|provider| {
                let id = provider.id;
                ProviderDetailVO {
                    id,
                    provider: ProviderVO::from(provider),
                    models: grouped.remove(&id).unwrap_or_default(),
                }
            })
            .collect();

        Ok(PaginationResult {
            total: page.total,
            records,
        })
    }
}
