use anyhow::{Context, Result};
use framework_core::types::{PaginationParams, PaginationResult};
use lib_db::{PgPool, PgTransaction, QueryBuilderExt};
use sqlx::{Postgres, QueryBuilder, Row};
use types_admin::DEFAULT_PROVIDER_ID;
use types_admin::dto::{ProviderModelCreatePO, ProviderModelQO, ProviderModelUpdatePO};
use types_admin::entity::ProviderModel;

/// 供应商模型数据访问。
pub struct ProviderModelRepository {
    pool: PgPool,
}

const COLUMNS: &str = "pm.id, pm.provider_id, pm.model, pm.display_name, pm.reasoning, pm.levels,
    pm.level_default, pm.context_window, pm.max_tokens, pm.support_tools, pm.support_vision,
    pm.support_stream, pm.support_json, pm.support_cache, pm.knowledge_cutoff, pm.release_date,
    pm.enabled, pm.create_time, pm.update_time";

impl ProviderModelRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<ProviderModel>> {
        let query = format!(
            "SELECT {COLUMNS} FROM provider_model pm
             WHERE pm.provider_id <> {DEFAULT_PROVIDER_ID} AND pm.id = $1 LIMIT 1"
        );

        let row = sqlx::query(&query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .context("查询供应商模型失败")?;

        row.map(provider_model_from_row).transpose()
    }

    pub async fn find_by_provider_id(&self, provider_id: i64) -> Result<Vec<ProviderModel>> {
        let query = format!(
            "SELECT {COLUMNS} FROM provider_model pm
             WHERE pm.provider_id <> {DEFAULT_PROVIDER_ID} AND pm.provider_id = $1
             ORDER BY pm.model ASC"
        );

        let rows = sqlx::query(&query)
            .bind(provider_id)
            .fetch_all(&self.pool)
            .await
            .context("查询供应商模型失败")?;

        rows.into_iter().map(provider_model_from_row).collect()
    }

    /// 批量查询多个供应商的模型，用于供应商分页时一次性装配模型。
    pub async fn find_by_provider_ids(&self, provider_ids: &[i64]) -> Result<Vec<ProviderModel>> {
        if provider_ids.is_empty() {
            return Ok(Vec::new());
        }

        let query = format!(
            "SELECT {COLUMNS} FROM provider_model pm
             WHERE pm.provider_id <> {DEFAULT_PROVIDER_ID} AND pm.provider_id = ANY($1)
             ORDER BY pm.provider_id ASC, pm.model ASC"
        );

        let rows = sqlx::query(&query)
            .bind(provider_ids)
            .fetch_all(&self.pool)
            .await
            .context("查询供应商模型失败")?;

        rows.into_iter().map(provider_model_from_row).collect()
    }

    /// 查询所有启用模型；用于可用模型列表与路由匹配。
    pub async fn find_enabled(&self) -> Result<Vec<ProviderModel>> {
        let query = format!(
            "SELECT {COLUMNS} FROM provider_model pm
             WHERE pm.provider_id <> {DEFAULT_PROVIDER_ID} AND pm.enabled = true
             ORDER BY pm.model ASC"
        );

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .context("查询启用供应商模型失败")?;

        rows.into_iter().map(provider_model_from_row).collect()
    }

    /// 查询所有启用模型，按模型名去重；同名取路由优先级最高的一条。
    /// 排序键与供应商路由规则一致：优先级升序、创建时间升序。
    pub async fn find_enabled_distinct(&self) -> Result<Vec<ProviderModel>> {
        let query = format!(
            "SELECT DISTINCT ON (pm.model) {COLUMNS}
             FROM provider_model pm
             JOIN provider p ON p.id = pm.provider_id
             WHERE pm.provider_id <> {DEFAULT_PROVIDER_ID}
               AND pm.enabled = true AND p.enabled = true AND p.deleted_at = 0
             ORDER BY pm.model ASC, p.priority ASC, p.create_time ASC"
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
             WHERE pm.provider_id <> {DEFAULT_PROVIDER_ID} AND pm.enabled = true AND p.enabled = true
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
            "SELECT {COLUMNS} FROM provider_model pm
             WHERE pm.provider_id <> {DEFAULT_PROVIDER_ID} AND pm.provider_id = $1 AND pm.model = $2
             LIMIT 1"
        );

        let row = sqlx::query(&query)
            .bind(provider_id)
            .bind(model)
            .fetch_optional(&self.pool)
            .await
            .context("查询供应商模型失败")?;

        row.map(provider_model_from_row).transpose()
    }

    /// 查询默认模型数据，唯一读取该数据的入口；其余查询均已排除默认 provider_id。
    pub async fn find_default(&self) -> Result<Vec<ProviderModel>> {
        let query = format!(
            "SELECT {COLUMNS} FROM provider_model pm
             WHERE pm.provider_id = {DEFAULT_PROVIDER_ID} ORDER BY pm.model ASC"
        );

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .context("查询默认模型数据失败")?;

        rows.into_iter().map(provider_model_from_row).collect()
    }

    /// 查询指定供应商已入库的模型名称；模型同步任务使用。
    pub async fn find_models(&self, provider_id: i64) -> Result<Vec<String>> {
        let rows = sqlx::query(
            "SELECT model FROM provider_model
             WHERE provider_id = $1 AND provider_id <> {DEFAULT_PROVIDER_ID}
             ORDER BY model ASC",
        )
        .bind(provider_id)
        .fetch_all(&self.pool)
        .await
        .context("查询供应商模型名称失败")?;

        Ok(rows.into_iter().map(|row| row.get("model")).collect())
    }

    /// 模型同步任务使用；批量写入，已存在时更新除启用状态外的全部字段。
    pub async fn upsert(
        &self,
        params: &[ProviderModelCreatePO],
        transaction: &mut PgTransaction<'_>,
    ) -> Result<()> {
        if params.is_empty() {
            return Ok(());
        }

        let now = lib_core::current_millis()?;
        let mut query = QueryBuilder::<Postgres>::new(
            "INSERT INTO provider_model
                (provider_id, model, display_name, reasoning, levels, level_default,
                 context_window, max_tokens, support_tools, support_vision, support_stream,
                 support_json, support_cache, knowledge_cutoff, release_date,
                 enabled, create_time, update_time) ",
        );
        query.push_values(params, |mut row, param| {
            row.push_bind(param.provider_id)
                .push_bind(&param.model)
                .push_bind(&param.display_name)
                .push_bind(param.reasoning)
                .push_bind(sqlx::types::Json(&param.levels))
                .push_bind(&param.level_default)
                .push_bind(param.context_window)
                .push_bind(param.max_tokens)
                .push_bind(param.support_tools)
                .push_bind(param.support_vision)
                .push_bind(param.support_stream)
                .push_bind(param.support_json)
                .push_bind(param.support_cache)
                .push_bind(param.knowledge_cutoff)
                .push_bind(param.release_date)
                .push_bind(true)
                .push_bind(now)
                .push_bind(now);
        });
        query.push(
            " ON CONFLICT (provider_id, model) DO UPDATE
             SET display_name = EXCLUDED.display_name,
                 reasoning = EXCLUDED.reasoning,
                 levels = EXCLUDED.levels,
                 level_default = EXCLUDED.level_default,
                 context_window = EXCLUDED.context_window,
                 max_tokens = EXCLUDED.max_tokens,
                 support_tools = EXCLUDED.support_tools,
                 support_vision = EXCLUDED.support_vision,
                 support_stream = EXCLUDED.support_stream,
                 support_json = EXCLUDED.support_json,
                 support_cache = EXCLUDED.support_cache,
                 knowledge_cutoff = EXCLUDED.knowledge_cutoff,
                 release_date = EXCLUDED.release_date,
                 update_time = EXCLUDED.update_time",
        );

        query
            .build()
            .execute(&mut **transaction)
            .await
            .context("写入供应商模型失败")?;

        Ok(())
    }

    /// 关闭供应商已不再返回的模型；同步任务使用。
    pub async fn disable_missing(
        &self,
        provider_id: i64,
        models: &[String],
        transaction: &mut PgTransaction<'_>,
    ) -> Result<u64> {
        let result = sqlx::query(
            "UPDATE provider_model SET enabled = false, update_time = $1
             WHERE provider_id = $2 AND provider_id <> {DEFAULT_PROVIDER_ID}
               AND enabled = true AND model <> ALL($3)",
        )
        .bind(lib_core::current_millis()?)
        .bind(provider_id)
        .bind(models)
        .execute(&mut **transaction)
        .await
        .context("关闭失效供应商模型失败")?;

        Ok(result.rows_affected())
    }

    pub async fn update(&self, params: &ProviderModelUpdatePO) -> Result<()> {
        sqlx::query(
            "UPDATE provider_model SET display_name = $1, levels = $2, level_default = $3,
                enabled = $4, update_time = $5
             WHERE id = $6 AND provider_id <> {DEFAULT_PROVIDER_ID}",
        )
        .bind(&params.display_name)
        .bind(sqlx::types::Json(&params.levels))
        .bind(&params.level_default)
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
            QueryBuilder::<Postgres>::new("SELECT COUNT(*) AS total FROM provider_model pm");
        push_provider_model_conditions(&mut count, conditions);

        let total_row = count.build().fetch_one(&self.pool).await?;
        let total: i64 = total_row.get("total");

        let mut query =
            QueryBuilder::<Postgres>::new(format!("SELECT {COLUMNS} FROM provider_model pm"));
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
    let levels = row.get::<sqlx::types::Json<Vec<String>>, _>("levels").0;

    Ok(ProviderModel {
        id: row.get("id"),
        provider_id: row.get("provider_id"),
        model: row.get("model"),
        display_name: row.get("display_name"),
        reasoning: row.get("reasoning"),
        levels,
        level_default: row.get("level_default"),
        context_window: row.get("context_window"),
        max_tokens: row.get("max_tokens"),
        support_tools: row.get("support_tools"),
        support_vision: row.get("support_vision"),
        support_stream: row.get("support_stream"),
        support_json: row.get("support_json"),
        support_cache: row.get("support_cache"),
        knowledge_cutoff: row.get("knowledge_cutoff"),
        release_date: row.get("release_date"),
        enabled: row.get("enabled"),
        create_time: row.get("create_time"),
        update_time: row.get("update_time"),
    })
}

fn push_provider_model_conditions<'a>(
    query: &mut QueryBuilder<'a, Postgres>,
    conditions: &'a ProviderModelQO,
) {
    query.push(format!(" WHERE pm.provider_id <> {DEFAULT_PROVIDER_ID}"));
    query.eq("pm.id", conditions.id);
    query.eq("pm.provider_id", conditions.provider_id);
    query.like("pm.model", conditions.model.as_deref());
    query.eq("pm.reasoning", conditions.reasoning);
    query.eq("pm.enabled", conditions.enabled);
}
