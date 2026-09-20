use std::collections::HashMap;

use anyhow::Result;
use types_admin::dto::{ProviderDetailVO, ProviderModelVO, ProviderVO};

use crate::service::{ProviderModelService, ProviderService};

/// 供应商跨表编排：查询供应商并嵌入各自的模型。
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

    /// 查询全部未删除供应商，附带每个供应商下的全部模型，按供应商路由策略排序。
    ///
    /// 先查供应商拿到主键集合，再一次性查出全部模型后按供应商归组，避免逐条查询。
    pub async fn list_detail(&self) -> Result<Vec<ProviderDetailVO>> {
        let providers = self.provider_service.find_all().await?;
        let ids: Vec<i64> = providers.iter().map(|provider| provider.id).collect();
        let names: HashMap<i64, String> = providers
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

        let vec = providers
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
        Ok(vec)
    }
}
