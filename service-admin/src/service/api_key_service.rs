use anyhow::Result;
use framework_core::types::{PaginationParams, PaginationResult};
use lib_core::{current_millis, next_id};
use sha1::{Digest, Sha1};
use types_admin::dto::{ApiKeyCreatePO, ApiKeyQO, ApiKeyUpdatePO};
use types_admin::entity::ApiKey;

use super::pool;
use crate::repository::ApiKeyRepository;

/// API Key 业务逻辑；库中只保存原始 key 的 sha1。
pub struct ApiKeyService;

impl ApiKeyService {
    /// 分页查询 API Key。
    pub async fn page(
        pagination: &PaginationParams,
        conditions: &ApiKeyQO,
    ) -> Result<PaginationResult<ApiKey>> {
        ApiKeyRepository::new(pool()?)
            .page(pagination, conditions)
            .await
    }

    /// 创建 API Key，返回主键与仅此一次可见的原始 key。
    pub async fn create(params: &ApiKeyCreatePO) -> Result<(i64, String)> {
        let key = generate_key()?;
        let id = ApiKeyRepository::new(pool()?)
            .create(params, &sha1_hex(&key))
            .await?;
        Ok((id, key))
    }

    /// 更新 API Key。
    pub async fn update(params: &ApiKeyUpdatePO) -> Result<()> {
        ApiKeyRepository::new(pool()?).update(params).await
    }

    /// 逻辑删除 API Key。
    pub async fn delete(id: i64) -> Result<()> {
        ApiKeyRepository::new(pool()?).delete(id).await
    }

    /// 查询全部启用且未删除的 API Key，用于装载鉴权数据。
    pub async fn find_enabled() -> Result<Vec<ApiKey>> {
        ApiKeyRepository::new(pool()?).find_enabled().await
    }
}

/// 生成 API Key 原始值，格式为 sk- 加 32 位摘要。
fn generate_key() -> Result<String> {
    let seed = format!("{}-{}-{}", next_id()?, current_millis()?, next_id()?);
    Ok(format!("sk-{}", &sha1_hex(&seed)[..32]))
}

/// 计算字符串的 SHA1 十六进制摘要。
fn sha1_hex(value: &str) -> String {
    format!("{:x}", Sha1::new().chain_update(value).finalize())
}
