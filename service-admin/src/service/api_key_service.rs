use anyhow::Result;
use framework_core::types::{PaginationParams, PaginationResult};
use lib_core::{current_millis, hash, next_id};
use types_admin::dto::{ApiKeyCreatePO, ApiKeyQO, ApiKeyUpdatePO};
use types_admin::entity::ApiKey;

use crate::repository::ApiKeyRepository;
use lib_db::use_pool;

/// API Key 业务逻辑；库中只保存原始 key 的 sha1。
pub struct ApiKeyService {
    repository: ApiKeyRepository,
}

impl ApiKeyService {
    /// 绑定当前请求的数据库连接池。
    pub fn new() -> Result<Self> {
        Ok(Self {
            repository: ApiKeyRepository::new(use_pool()?),
        })
    }

    /// 分页查询 API Key。
    pub async fn page(
        &self,
        pagination: &PaginationParams,
        conditions: &ApiKeyQO,
    ) -> Result<PaginationResult<ApiKey>> {
        self.repository.page(pagination, conditions).await
    }

    /// 创建 API Key，返回主键与仅此一次可见的原始 key。
    pub async fn create(&self, params: &ApiKeyCreatePO) -> Result<(i64, String)> {
        let key = generate_key()?;
        let id = self.repository.create(params, &hash(&key)).await?;
        Ok((id, key))
    }

    /// 更新 API Key。
    pub async fn update(&self, params: &ApiKeyUpdatePO) -> Result<()> {
        self.repository.update(params).await
    }

    /// 逻辑删除 API Key。
    pub async fn delete(&self, id: i64) -> Result<()> {
        self.repository.delete(id).await
    }

    /// 查询全部启用且未删除的 API Key。
    pub async fn find_enabled(&self) -> Result<Vec<ApiKey>> {
        self.repository.find_enabled().await
    }

    /// 按摘要查询启用且未删除的 API Key；鉴权流程使用。
    pub async fn find_by_key_hash(&self, key_hash: &str) -> Result<Option<ApiKey>> {
        self.repository.find_by_key_hash(key_hash).await
    }
}

/// 生成 API Key 原始值，格式为 sk- 加 32 位摘要。
fn generate_key() -> Result<String> {
    let seed = format!("{}-{}-{}", next_id()?, current_millis()?, next_id()?);
    Ok(format!("sk-{}", &hash(&seed)[..32]))
}
