use anyhow::Result;
use framework_core::types::{PaginationParams, PaginationResult};
use types_admin::dto::{ConfigQO, ConfigUpdatePO};
use types_admin::entity::KvConfig;

use super::pool;
use crate::repository::KvConfigRepository;

/// 全局配置业务逻辑。
pub struct ConfigService;

impl ConfigService {
    /// 分页查询全局配置。
    pub async fn page(
        pagination: &PaginationParams,
        conditions: &ConfigQO,
    ) -> Result<PaginationResult<KvConfig>> {
        KvConfigRepository::new(pool()?)
            .page(pagination, conditions)
            .await
    }

    /// 查询全部全局配置。
    pub async fn find_all() -> Result<Vec<KvConfig>> {
        KvConfigRepository::new(pool()?).find_all().await
    }

    /// 读取单个配置值。
    pub async fn find_value(config_key: &str) -> Result<Option<String>> {
        KvConfigRepository::new(pool()?)
            .find_value(config_key)
            .await
    }

    /// 更新配置值。
    pub async fn update(params: &ConfigUpdatePO) -> Result<()> {
        KvConfigRepository::new(pool()?)
            .update_value(&params.config_key, &params.config_value)
            .await
    }
}
