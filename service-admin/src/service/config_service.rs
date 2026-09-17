use anyhow::Result;
use framework_core::types::{PaginationParams, PaginationResult};
use types_admin::dto::{ConfigQO, ConfigUpdatePO};
use types_admin::entity::KvConfig;

use crate::repository::KvConfigRepository;
use lib_db::use_pool;

/// 全局配置业务逻辑。
pub struct ConfigService {
    repository: KvConfigRepository,
}

impl ConfigService {
    /// 绑定当前请求的数据库连接池。
    pub fn new() -> Result<Self> {
        Ok(Self {
            repository: KvConfigRepository::new(use_pool()?),
        })
    }

    /// 分页查询全局配置。
    pub async fn page(
        &self,
        pagination: &PaginationParams,
        conditions: &ConfigQO,
    ) -> Result<PaginationResult<KvConfig>> {
        self.repository.page(pagination, conditions).await
    }

    /// 查询全部全局配置。
    pub async fn find_all(&self) -> Result<Vec<KvConfig>> {
        self.repository.find_all().await
    }

    /// 按 key 集合批量查询配置，避免逐个 key 查询。
    pub async fn find_keys(&self, keys: &[&str]) -> Result<Vec<KvConfig>> {
        self.repository.find_keys(keys).await
    }

    /// 读取单个配置值。
    pub async fn find_value(&self, config_key: &str) -> Result<Option<String>> {
        self.repository.find_value(config_key).await
    }

    /// 更新配置值。
    pub async fn update(&self, params: &ConfigUpdatePO) -> Result<()> {
        self.repository
            .update_value(&params.config_key, &params.config_value)
            .await
    }
}
