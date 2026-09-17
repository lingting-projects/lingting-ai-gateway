use anyhow::{Context, Result};
use framework_core::types::{PaginationParams, PaginationResult};
use lib_db::{PgPool, QueryBuilderExt};
use sqlx::{Postgres, QueryBuilder, Row};
use types_admin::dto::{ProviderModelQO, ProviderModelUpdatePO};
use types_admin::entity::{InferenceLevel, ProviderModel};

/// 供应商模型数据访问。
pub struct ProviderModelRepository {
    pool: PgPool,
}

const COLUMNS: &str =
    "id, provider_id, model, display_name, inference_level, enabled, create_time, update_time";

impl ProviderModelRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<ProviderModel>> {
        let query = format!("SELECT {COLUMNS} FROM provider_model WHERE id = $1 LIMIT 1");

        let row = sqlx::query(&query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .context("查询供应商模型失败")?;

        row.map(provider_model_from_row).transpose()
    }

    pub async fn find_by_provider_id(&self, provider_id: i64) -> Result<Vec<ProviderModel>> {
        let query = format!(
            "SELECT {COLUMNS} FROM provider_model WHERE provider_id = $1 ORDER BY model ASC"
        );

        let rows = sqlx::query(&query)
            .bind(provider_id)
            .fetch_all(&self.pool)
            .await
            .context("查询供应商模型失败")?;

        rows.into_iter().map(provider_model_from_row).collect()
    }

    /// 查询所有启用模型；用于可用模型列表与路由匹配。
    pub async fn find_enabled(&self) -> Result<Vec<ProviderModel>> {
        let query = format!(
            "SELECT {COLUMNS} FROM provider_model WHERE enabled = true ORDER BY model ASC"
        );

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .context("查询启用供应商模型失败")?;

        rows.into_iter().map(provider_model_from_row).collect()
    }

    /// 查询启用的模型名称，已去重并排序；用于可用模型列表接口。
    pub async fn find_enabled_names(&self) -> Result<Vec<String>> {
        let rows = sqlx::query(
            "SELECT DISTINCT pm.model
             FROM provider_model pm
             JOIN provider p ON p.id = pm.provider_id
             WHERE pm.enabled = true AND p.enabled = true
             ORDER BY pm.model ASC",
        )
        .fetch_all(&self.pool)
        .await
        .context("查询可用模型失败")?;

        Ok(rows.into_iter().map(|row| row.get("model")).collect())
    }

    pub async fn find_by_provider_and_model(
        &self,
        provider_id: i64,
        model: &str,
    ) -> Result<Option<ProviderModel>> {
        let query = format!(
            "SELECT {COLUMNS} FROM provider_model WHERE provider_id = $1 AND model = $2 LIMIT 1"
        );

        let row = sqlx::query(&query)
            .bind(provider_id)
            .bind(model)
            .fetch_optional(&self.pool)
            .await
            .context("查询供应商模型失败")?;

        row.map(provider_model_from_row).transpose()
    }

    /// 模型同步任务使用；已存在时更新展示名与推理级别，不改变启用状态。
    pub async fn upsert(
        &self,
        provider_id: i64,
        model: &str,
        display_name: &str,
        inference_level: &InferenceLevel,
    ) -> Result<()> {
        let now = lib_core::current_millis()?;

        sqlx::query(
            "INSERT INTO provider_model
                (provider_id, model, display_name, inference_level, enabled, create_time, update_time)
             VALUES ($1, $2, $3, $4, true, $5, $5)
             ON CONFLICT (provider_id, model) DO UPDATE
             SET display_name = EXCLUDED.display_name,
                 update_time = EXCLUDED.update_time",
        )
        .bind(provider_id)
        .bind(model)
        .bind(display_name)
        .bind(inference_level.as_str())
        .bind(now)
        .execute(&self.pool)
        .await
        .context("写入供应商模型失败")?;

        Ok(())
    }

    /// 关闭供应商已不再返回的模型；同步任务使用。
    pub async fn disable_missing(&self, provider_id: i64, models: &[String]) -> Result<u64> {
        let result = sqlx::query(
            "UPDATE provider_model SET enabled = false, update_time = $1
             WHERE provider_id = $2 AND enabled = true AND model <> ALL($3)",
        )
        .bind(lib_core::current_millis()?)
        .bind(provider_id)
        .bind(models)
        .execute(&self.pool)
        .await
        .context("关闭失效供应商模型失败")?;

        Ok(result.rows_affected())
    }

    pub async fn update(&self, params: &ProviderModelUpdatePO) -> Result<()> {
        sqlx::query(
            "UPDATE provider_model SET display_name = $1, inference_level = $2, enabled = $3,
                update_time = $4 WHERE id = $5",
        )
        .bind(&params.display_name)
        .bind(params.inference_level.as_str())
        .bind(params.enabled)
        .bind(lib_core::current_millis()?)
        .bind(params.id)
        .execute(&self.pool)
        .await
        .context("更新供应商模型失败")?;

        Ok(())
    }

    pub async fn page(
        &self,
        pagination: &PaginationParams,
        conditions: &ProviderModelQO,
    ) -> Result<PaginationResult<ProviderModel>> {
        let mut count =
            QueryBuilder::<Postgres>::new("SELECT COUNT(*) AS total FROM provider_model");
        push_provider_model_conditions(&mut count, conditions);

        let total_row = count.build().fetch_one(&self.pool).await?;
        let total: i64 = total_row.get("total");

        let mut query =
            QueryBuilder::<Postgres>::new(format!("SELECT {COLUMNS} FROM provider_model"));
        push_provider_model_conditions(&mut query, conditions);
        query.push_pagination(pagination);

        let rows = query.build().fetch_all(&self.pool).await?;
        let records = rows
            .into_iter()
            .filter_map(|row| provider_model_from_row(row).ok())
            .collect();

        Ok(PaginationResult { total, records })
    }
}

fn provider_model_from_row(row: sqlx::postgres::PgRow) -> Result<ProviderModel> {
    Ok(ProviderModel {
        id: row.get("id"),
        provider_id: row.get("provider_id"),
        model: row.get("model"),
        display_name: row.get("display_name"),
        inference_level: InferenceLevel::from_db(&row.get::<String, _>("inference_level")),
        enabled: row.get("enabled"),
        create_time: row.get("create_time"),
        update_time: row.get("update_time"),
    })
}

fn push_provider_model_conditions<'a>(
    query: &mut QueryBuilder<'a, Postgres>,
    conditions: &'a ProviderModelQO,
) {
    query.push(" WHERE 1 = 1");
    query.eq("id", conditions.id);
    query.eq("provider_id", conditions.provider_id);
    query.like("model", conditions.model.as_deref());
    query.eq(
        "inference_level",
        conditions.inference_level.as_ref().map(|level| level.as_str()),
    );
    query.eq("enabled", conditions.enabled);
}
