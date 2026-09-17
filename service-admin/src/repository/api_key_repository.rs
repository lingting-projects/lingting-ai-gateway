use anyhow::{Context, Result};
use framework_core::types::{PaginationParams, PaginationResult};
use lib_db::{PgPool, QueryBuilderExt};
use sqlx::{Postgres, QueryBuilder, Row};
use types_admin::dto::{ApiKeyCreatePO, ApiKeyQO, ApiKeyUpdatePO};
use types_admin::entity::ApiKey;

/// API Key 数据访问；库中只保存原始 key 的 sha1 值。
pub struct ApiKeyRepository {
    pool: PgPool,
}

const COLUMNS: &str = "id, name, key_hash, enabled, deleted, remark, create_time, update_time";

impl ApiKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 按 sha1 值查找可用密钥；鉴权流程使用。
    pub async fn find_by_key_hash(&self, key_hash: &str) -> Result<Option<ApiKey>> {
        let query = format!(
            "SELECT {COLUMNS} FROM api_key
             WHERE key_hash = $1 AND enabled = true AND deleted = false LIMIT 1"
        );

        let row = sqlx::query(&query)
            .bind(key_hash)
            .fetch_optional(&self.pool)
            .await
            .context("查询 API Key 失败")?;

        row.map(api_key_from_row).transpose()
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<ApiKey>> {
        let query = format!("SELECT {COLUMNS} FROM api_key WHERE id = $1 LIMIT 1");

        let row = sqlx::query(&query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .context("查询 API Key 失败")?;

        row.map(api_key_from_row).transpose()
    }

    pub async fn create(&self, params: &ApiKeyCreatePO, key_hash: &str) -> Result<i64> {
        let now = lib_core::current_millis()?;

        let row = sqlx::query(
            "INSERT INTO api_key (name, key_hash, enabled, deleted, remark, create_time, update_time)
             VALUES ($1, $2, true, false, $3, $4, $4)
             RETURNING id",
        )
            .bind(&params.name)
            .bind(key_hash)
            .bind(&params.remark)
            .bind(now)
            .fetch_one(&self.pool)
            .await
            .context("创建 API Key 失败")?;

        Ok(row.get("id"))
    }

    pub async fn update(&self, params: &ApiKeyUpdatePO) -> Result<()> {
        sqlx::query(
            "UPDATE api_key SET name = $1, enabled = $2, remark = $3, update_time = $4
             WHERE id = $5",
        )
            .bind(&params.name)
            .bind(params.enabled)
            .bind(&params.remark)
            .bind(lib_core::current_millis()?)
            .bind(params.id)
            .execute(&self.pool)
            .await
            .context("更新 API Key 失败")?;

        Ok(())
    }

    /// 逻辑删除，保留记录以便追溯历史请求日志中的凭据标识。
    pub async fn delete(&self, id: i64) -> Result<()> {
        sqlx::query("UPDATE api_key SET deleted = true, update_time = $1 WHERE id = $2")
            .bind(lib_core::current_millis()?)
            .bind(id)
            .execute(&self.pool)
            .await
            .context("删除 API Key 失败")?;

        Ok(())
    }

    /// 查询全部启用且未删除的密钥；装载鉴权数据使用。
    pub async fn find_enabled(&self) -> Result<Vec<ApiKey>> {
        let query = format!(
            "SELECT {COLUMNS} FROM api_key WHERE enabled = true AND deleted = false ORDER BY id ASC"
        );

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .context("查询启用的 API Key 失败")?;

        rows.into_iter().map(api_key_from_row).collect()
    }

    pub async fn page(
        &self,
        pagination: &PaginationParams,
        conditions: &ApiKeyQO,
    ) -> Result<PaginationResult<ApiKey>> {
        let mut count = QueryBuilder::<Postgres>::new("SELECT COUNT(*) AS total FROM api_key");
        push_api_key_conditions(&mut count, conditions);

        let total_row = count.build().fetch_one(&self.pool).await?;
        let total: i64 = total_row.get("total");

        let mut query = QueryBuilder::<Postgres>::new(format!("SELECT {COLUMNS} FROM api_key"));
        push_api_key_conditions(&mut query, conditions);
        query.push_pagination(pagination);

        let rows = query.build().fetch_all(&self.pool).await?;
        let records = rows
            .into_iter()
            .filter_map(|row| api_key_from_row(row).ok())
            .collect();

        Ok(PaginationResult { total, records })
    }
}

fn api_key_from_row(row: sqlx::postgres::PgRow) -> Result<ApiKey> {
    Ok(ApiKey {
        id: row.get("id"),
        name: row.get("name"),
        key_hash: row.get("key_hash"),
        enabled: row.get("enabled"),
        deleted: row.get("deleted"),
        remark: row.get("remark"),
        create_time: row.get("create_time"),
        update_time: row.get("update_time"),
    })
}

fn push_api_key_conditions<'a>(query: &mut QueryBuilder<'a, Postgres>, conditions: &'a ApiKeyQO) {
    query.push(" WHERE deleted = false");
    query.like("name", conditions.name.as_deref());
    query.eq("id", conditions.id);
    query.eq("enabled", conditions.enabled);
}
