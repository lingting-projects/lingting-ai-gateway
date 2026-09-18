use anyhow::{Context, Result};
use framework_core::types::{PaginationParams, PaginationResult};
use lib_db::{PgPool, QueryBuilderExt};
use sqlx::{Postgres, QueryBuilder, Row};
use types_admin::dto::{
    ProviderModelRedirectCreatePO, ProviderModelRedirectQO, ProviderModelRedirectUpdatePO,
};
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

    /// 按非官方模型名查询映射；名称映射使用。
    pub async fn find_by_source(&self, source: &str) -> Result<Option<ProviderModelRedirect>> {
        let query =
            format!("SELECT {COLUMNS} FROM provider_model_redirect WHERE source = $1 LIMIT 1");

        let row = sqlx::query(&query)
            .bind(source)
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

    pub async fn page(
        &self,
        pagination: &PaginationParams,
        conditions: &ProviderModelRedirectQO,
    ) -> Result<PaginationResult<ProviderModelRedirect>> {
        let mut count =
            QueryBuilder::<Postgres>::new("SELECT COUNT(*) AS total FROM provider_model_redirect");
        push_provider_model_redirect_conditions(&mut count, conditions);

        let total_row = count.build().fetch_one(&self.pool).await?;
        let total: i64 = total_row.get("total");

        let mut query =
            QueryBuilder::<Postgres>::new(format!("SELECT {COLUMNS} FROM provider_model_redirect"));
        push_provider_model_redirect_conditions(&mut query, conditions);
        query.push_pagination(pagination);

        let rows = query.build().fetch_all(&self.pool).await?;
        let records = rows
            .into_iter()
            .filter_map(|row| provider_model_redirect_from_row(row).ok())
            .collect();

        Ok(PaginationResult { total, records })
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

fn push_provider_model_redirect_conditions<'a>(
    query: &mut QueryBuilder<'a, Postgres>,
    conditions: &'a ProviderModelRedirectQO,
) {
    query.push(" WHERE 1 = 1");
    query.eq("id", conditions.id);
    query.like("source", conditions.source.as_deref());
    query.like("target", conditions.target.as_deref());
}
