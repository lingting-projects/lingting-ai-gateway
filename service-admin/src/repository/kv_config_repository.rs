use anyhow::{Context, Result};
use framework_core::types::{PaginationParams, PaginationResult};
use lib_db::{PgPool, QueryBuilderExt};
use sqlx::{Postgres, QueryBuilder, Row};
use types_admin::dto::ConfigQO;
use types_admin::entity::KvConfig;

/// 全局配置数据访问。
pub struct KvConfigRepository {
    pool: PgPool,
}

const COLUMNS: &str = "id, config_key, config_value, description, create_time, update_time";

impl KvConfigRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_key(&self, config_key: &str) -> Result<Option<KvConfig>> {
        let query = format!("SELECT {COLUMNS} FROM kv_config WHERE config_key = $1 LIMIT 1");

        let row = sqlx::query(&query)
            .bind(config_key)
            .fetch_optional(&self.pool)
            .await
            .context("查询全局配置失败")?;

        row.map(kv_config_from_row).transpose()
    }

    pub async fn find_all(&self) -> Result<Vec<KvConfig>> {
        let query = format!("SELECT {COLUMNS} FROM kv_config ORDER BY id ASC");

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .context("查询全局配置失败")?;

        rows.into_iter().map(kv_config_from_row).collect()
    }

    /// 按 key 集合批量查询配置，避免逐个 key 查询。
    pub async fn find_keys(&self, keys: &[&str]) -> Result<Vec<KvConfig>> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        let query =
            format!("SELECT {COLUMNS} FROM kv_config WHERE config_key = ANY($1) ORDER BY id ASC");

        let rows = sqlx::query(&query)
            .bind(keys)
            .fetch_all(&self.pool)
            .await
            .context("查询全局配置失败")?;

        rows.into_iter().map(kv_config_from_row).collect()
    }

    pub async fn find_value(&self, config_key: &str) -> Result<Option<String>> {
        let row = sqlx::query("SELECT config_value FROM kv_config WHERE config_key = $1 LIMIT 1")
            .bind(config_key)
            .fetch_optional(&self.pool)
            .await
            .context("查询全局配置失败")?;

        Ok(row.map(|row| row.get("config_value")))
    }

    pub async fn update_value(&self, config_key: &str, config_value: &str) -> Result<()> {
        sqlx::query(
            "UPDATE kv_config SET config_value = $1, update_time = $2 WHERE config_key = $3",
        )
        .bind(config_value)
        .bind(lib_core::current_millis()?)
        .bind(config_key)
        .execute(&self.pool)
        .await
        .context("更新全局配置失败")?;

        Ok(())
    }

    pub async fn page(
        &self,
        pagination: &PaginationParams,
        conditions: &ConfigQO,
    ) -> Result<PaginationResult<KvConfig>> {
        let mut count = QueryBuilder::<Postgres>::new("SELECT COUNT(*) AS total FROM kv_config");
        push_config_conditions(&mut count, conditions);

        let total_row = count.build().fetch_one(&self.pool).await?;
        let total: i64 = total_row.get("total");

        let mut query = QueryBuilder::<Postgres>::new(format!("SELECT {COLUMNS} FROM kv_config"));
        push_config_conditions(&mut query, conditions);
        query.push_pagination(pagination);

        let rows = query.build().fetch_all(&self.pool).await?;
        let records = rows
            .into_iter()
            .filter_map(|row| kv_config_from_row(row).ok())
            .collect();

        Ok(PaginationResult { total, records })
    }
}

fn kv_config_from_row(row: sqlx::postgres::PgRow) -> Result<KvConfig> {
    Ok(KvConfig {
        id: row.get("id"),
        config_key: row.get("config_key"),
        config_value: row.get("config_value"),
        description: row.get("description"),
        create_time: row.get("create_time"),
        update_time: row.get("update_time"),
    })
}

fn push_config_conditions<'a>(
    query: &mut QueryBuilder<'a, Postgres>,
    conditions: &'a ConfigQO,
) {
    query.push(" WHERE 1 = 1");
    query.like("config_key", conditions.config_key.as_deref());
}
