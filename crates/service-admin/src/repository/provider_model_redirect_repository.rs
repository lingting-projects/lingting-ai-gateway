use anyhow::{Context, Result};
use lib_db::PgPool;
use sqlx::Row;
use types_admin::dto::{ProviderModelRedirectCreatePO, ProviderModelRedirectUpdatePO};
use types_admin::entity::ProviderModelRedirect;

/// 模型名称映射数据访问。
pub struct ProviderModelRedirectRepository {
    pool: PgPool,
}

const COLUMNS: &str = "id, source, target, create_time";

impl ProviderModelRedirectRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<ProviderModelRedirect>> {
        let query = format!("SELECT {COLUMNS} FROM provider_model_redirect WHERE id = $1 LIMIT 1");

        let row = sqlx::query(&query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .context("查询模型名称映射失败")?;

        row.map(provider_model_redirect_from_row).transpose()
    }

    /// 查询全部模型名称映射；模型同步任务使用。
    pub async fn find_all(&self) -> Result<Vec<ProviderModelRedirect>> {
        let query = format!("SELECT {COLUMNS} FROM provider_model_redirect ORDER BY source ASC");

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .context("查询模型名称映射失败")?;

        rows.into_iter()
            .map(provider_model_redirect_from_row)
            .collect()
    }
    pub async fn create(&self, params: &ProviderModelRedirectCreatePO) -> Result<i64> {
        let now = lib_core::current_millis()?;

        let row = sqlx::query(
            "INSERT INTO provider_model_redirect (source, target, create_time)
             VALUES ($1, $2, $3)
             RETURNING id",
        )
        .bind(&params.source)
        .bind(&params.target)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .context("创建模型名称映射失败")?;

        Ok(row.get("id"))
    }

    pub async fn update(&self, params: &ProviderModelRedirectUpdatePO) -> Result<()> {
        sqlx::query("UPDATE provider_model_redirect SET source = $1, target = $2 WHERE id = $3")
            .bind(&params.source)
            .bind(&params.target)
            .bind(params.id)
            .execute(&self.pool)
            .await
            .context("更新模型名称映射失败")?;

        Ok(())
    }

    pub async fn delete(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM provider_model_redirect WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("删除模型名称映射失败")?;

        Ok(())
    }
}

fn provider_model_redirect_from_row(row: sqlx::postgres::PgRow) -> Result<ProviderModelRedirect> {
    Ok(ProviderModelRedirect {
        id: row.get("id"),
        source: row.get("source"),
        target: row.get("target"),
        create_time: row.get("create_time"),
    })
}
