use anyhow::{Context, Result};
use framework_core::types::{PaginationParams, PaginationResult};
use lib_db::{PgPool, QueryBuilderExt};
use sqlx::{Postgres, QueryBuilder, Row};
use types_admin::dto::{ProviderCreatePO, ProviderQO, ProviderUpdatePO};
use types_admin::entity::Provider;

/// 供应商数据访问。
pub struct ProviderRepository {
    pool: PgPool,
}

const COLUMNS: &str =
    "id, name, display_name, base_url, api_key, priority, enabled, config, create_time, update_time";

impl ProviderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<Provider>> {
        let query = format!("SELECT {COLUMNS} FROM provider WHERE id = $1 LIMIT 1");

        let row = sqlx::query(&query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .context("查询供应商失败")?;

        row.map(provider_from_row).transpose()
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<Provider>> {
        let query = format!("SELECT {COLUMNS} FROM provider WHERE name = $1 LIMIT 1");

        let row = sqlx::query(&query)
            .bind(name)
            .fetch_optional(&self.pool)
            .await
            .context("查询供应商失败")?;

        row.map(provider_from_row).transpose()
    }

    /// 查询所有启用的供应商，按优先级升序、加入时间升序排列。
    pub async fn find_enabled(&self) -> Result<Vec<Provider>> {
        let query = format!(
            "SELECT {COLUMNS} FROM provider WHERE enabled = true ORDER BY priority ASC, create_time ASC"
        );

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .context("查询启用的供应商失败")?;

        rows.into_iter().map(provider_from_row).collect()
    }

    /// 查询提供指定原始模型名（无别名）的启用供应商，顺序即路由尝试顺序。
    pub async fn find_enabled_by_model(&self, model: &str) -> Result<Vec<Provider>> {
        let query = format!(
            "SELECT p.id, p.name, p.display_name, p.base_url, p.api_key, p.priority, p.enabled,
                    p.config, p.create_time, p.update_time
             FROM provider p
             JOIN provider_model pm ON pm.provider_id = p.id
             WHERE p.enabled = true AND pm.enabled = true AND pm.model = $1
             ORDER BY p.priority ASC, p.create_time ASC"
        );

        let rows = sqlx::query(&query)
            .bind(model)
            .fetch_all(&self.pool)
            .await
            .context("查询支持指定模型的供应商失败")?;

        rows.into_iter().map(provider_from_row).collect()
    }

    pub async fn create(&self, params: &ProviderCreatePO) -> Result<i64> {
        let now = lib_core::current_millis()?;

        let row = sqlx::query(
            "INSERT INTO provider
                (name, display_name, base_url, api_key, priority, enabled, config, create_time, update_time)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)
             RETURNING id",
        )
        .bind(&params.name)
        .bind(&params.display_name)
        .bind(&params.base_url)
        .bind(&params.api_key)
        .bind(params.priority)
        .bind(params.enabled)
        .bind(&params.config)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .context("创建供应商失败")?;

        Ok(row.get("id"))
    }

    pub async fn update(&self, params: &ProviderUpdatePO) -> Result<()> {
        let mut query = QueryBuilder::<Postgres>::new("UPDATE provider SET ");
        query
            .push("display_name = ")
            .push_bind(&params.display_name)
            .push(", base_url = ")
            .push_bind(&params.base_url)
            .push(", priority = ")
            .push_bind(params.priority)
            .push(", enabled = ")
            .push_bind(params.enabled);

        // 为空表示不修改已保存的 key。
        if let Some(api_key) = &params.api_key {
            query.push(", api_key = ").push_bind(api_key);
        }

        if let Some(config) = &params.config {
            query.push(", config = ").push_bind(config);
        }

        query
            .push(", update_time = ")
            .push_bind(lib_core::current_millis()?)
            .push(" WHERE id = ")
            .push_bind(params.id);

        query
            .build()
            .execute(&self.pool)
            .await
            .context("更新供应商失败")?;

        Ok(())
    }

    pub async fn page(
        &self,
        pagination: &PaginationParams,
        conditions: &ProviderQO,
    ) -> Result<PaginationResult<Provider>> {
        let mut count = QueryBuilder::<Postgres>::new("SELECT COUNT(*) AS total FROM provider");
        push_provider_conditions(&mut count, conditions);

        let total_row = count.build().fetch_one(&self.pool).await?;
        let total: i64 = total_row.get("total");

        let mut query = QueryBuilder::<Postgres>::new(format!("SELECT {COLUMNS} FROM provider"));
        push_provider_conditions(&mut query, conditions);
        query.push(" ORDER BY priority ASC, create_time ASC ");

        let offset = (pagination.current - 1) * pagination.size;
        query
            .push(" LIMIT ")
            .push_bind(pagination.size)
            .push(" OFFSET ")
            .push_bind(offset);

        let rows = query.build().fetch_all(&self.pool).await?;
        let records = rows
            .into_iter()
            .filter_map(|row| provider_from_row(row).ok())
            .collect();

        Ok(PaginationResult { total, records })
    }
}

fn provider_from_row(row: sqlx::postgres::PgRow) -> Result<Provider> {
    Ok(Provider {
        id: row.get("id"),
        name: row.get("name"),
        display_name: row.get("display_name"),
        base_url: row.get("base_url"),
        api_key: row.get("api_key"),
        priority: row.get("priority"),
        enabled: row.get("enabled"),
        config: row.get("config"),
        create_time: row.get("create_time"),
        update_time: row.get("update_time"),
    })
}

fn push_provider_conditions<'a>(
    query: &mut QueryBuilder<'a, Postgres>,
    conditions: &'a ProviderQO,
) {
    query.push(" WHERE 1 = 1");
    query.like("name", conditions.name.as_deref());
    query.eq("id", conditions.id);
    query.eq("enabled", conditions.enabled);
}
