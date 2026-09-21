use std::collections::HashMap;

use anyhow::Result;
use types_admin::dto::{
    DashboardProviderVO, DashboardQO, DashboardRequestVO, DashboardTokenFilterVO,
    DashboardTokenGroup, DashboardTokenVO,
};

use crate::service::{ProviderService, RequestService};

/// 仪表盘统计编排：跨表统计请求日志并补齐供应商信息。
pub struct DashboardManager {
    request_service: RequestService,
    provider_service: ProviderService,
}

impl DashboardManager {
    /// 绑定当前请求的数据库连接池。
    pub fn new() -> Result<Self> {
        Ok(Self {
            request_service: RequestService::new()?,
            provider_service: ProviderService::new()?,
        })
    }

    /// 统计主请求日志的请求数量。
    pub async fn request_main(&self, conditions: &DashboardQO) -> Result<DashboardRequestVO> {
        self.request_service
            .dashboard_main_request(conditions)
            .await
    }

    /// 统计子请求日志的请求数量。
    pub async fn request_sub(&self, conditions: &DashboardQO) -> Result<DashboardRequestVO> {
        self.request_service.dashboard_sub_request(conditions).await
    }

    /// 统计主请求日志的 token。
    pub async fn token_main(&self, conditions: &DashboardQO) -> Result<DashboardTokenVO> {
        self.request_service.dashboard_main_token(conditions).await
    }

    /// 统计子请求日志的 token。
    pub async fn token_sub(&self, conditions: &DashboardQO) -> Result<DashboardTokenVO> {
        self.request_service.dashboard_sub_token(conditions).await
    }

    /// 统计子请求日志的 token，按天（可再按供应商、模型）分组并补齐供应商信息。
    ///
    /// 先查分组结果拿到供应商主键集合，再一次性查出供应商后按主键补齐，避免逐条查询。
    pub async fn token_sub_filter(
        &self,
        conditions: &DashboardQO,
    ) -> Result<Vec<DashboardTokenFilterVO>> {
        let groups = self
            .request_service
            .dashboard_sub_token_group(conditions)
            .await?;
        let providers = self.find_providers(&groups).await?;

        Ok(groups
            .into_iter()
            .map(|group| DashboardTokenFilterVO {
                day: group.day,
                provider: group.provider_id.map(|id| provider_of(&providers, id)),
                model: group.model,
                total: group.total,
                cache_read: group.cache_read,
                cache_write: group.cache_write,
                input: group.input,
                output: group.output,
            })
            .collect())
    }

    /// 按分组结果中的供应商主键批量查询供应商，返回主键到供应商信息的映射。
    async fn find_providers(
        &self,
        groups: &[DashboardTokenGroup],
    ) -> Result<HashMap<i64, DashboardProviderVO>> {
        let mut ids: Vec<i64> = groups
            .iter()
            .filter_map(|group| group.provider_id)
            .collect();
        ids.sort_unstable();
        ids.dedup();
        if ids.is_empty() {
            return Ok(HashMap::new());
        }

        let providers = self.provider_service.find_by_ids_with_deleted(&ids).await?;

        Ok(providers
            .into_iter()
            .map(|provider| {
                let vo = DashboardProviderVO {
                    id: provider.id,
                    name: provider.name,
                    display_name: provider.display_name,
                    deleted_at: provider.deleted_at,
                };
                (vo.id, vo)
            })
            .collect())
    }
}

/// 供应商信息；记录不存在时只保留主键，避免统计结果缺行。
fn provider_of(providers: &HashMap<i64, DashboardProviderVO>, id: i64) -> DashboardProviderVO {
    providers.get(&id).cloned().unwrap_or(DashboardProviderVO {
        id,
        name: String::new(),
        display_name: String::new(),
        deleted_at: 0,
    })
}
