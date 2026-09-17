use anyhow::{Context, Result};
use framework_core::types::{PaginationParams, PaginationResult};
use lib_db::{PgPool, QueryBuilderExt};
use sqlx::{Postgres, QueryBuilder, Row};
use types_admin::dto::{RequestFinishPO, RequestSubCreatePO, RequestSubQO};
use types_admin::entity::{RequestStatus, RequestSub};

/// 子请求日志数据访问。
pub struct RequestSubRepository {
    pool: PgPool,
}

const COLUMNS: &str = "id, main_request_id, provider_id, provider_name, model, provider_url,
    request_params, status, error_type, error_code, error_message, return_model, input_tokens,
    output_tokens, cache_read_tokens, cache_write_tokens, inference_tokens, read_tokens,
    write_tokens, total_tokens, http_status, finish_reason, provider_request_id, response_content,
    start_time, end_time, duration_ms, create_time";

impl RequestSubRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 写入子请求日志，返回日志主键。
    pub async fn create(&self, params: &RequestSubCreatePO) -> Result<i64> {
        let now = lib_core::current_millis()?;

        let row = sqlx::query(
            "INSERT INTO request_sub
                (main_request_id, provider_id, provider_name, model, provider_url, request_params,
                 status, start_time, create_time)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             RETURNING id",
        )
            .bind(params.main_request_id)
            .bind(params.provider_id)
            .bind(&params.provider_name)
            .bind(&params.model)
            .bind(&params.provider_url)
            .bind(&params.request_params)
            .bind(RequestStatus::Processing.as_str())
            .bind(params.start_time)
            .bind(now)
            .fetch_one(&self.pool)
            .await
            .context("写入子请求日志失败")?;

        Ok(row.get("id"))
    }

    /// 请求结束时写入状态、错误、token 与返回信息。
    pub async fn finish(&self, id: i64, params: &RequestFinishPO) -> Result<()> {
        sqlx::query(
            "UPDATE request_sub SET
                status = $1, error_type = $2, error_code = $3, error_message = $4,
                return_model = $5, input_tokens = $6, output_tokens = $7,
                cache_read_tokens = $8, cache_write_tokens = $9, inference_tokens = $10,
                read_tokens = $11, write_tokens = $12, total_tokens = $13, http_status = $14,
                finish_reason = $15, provider_request_id = $16, response_content = $17,
                end_time = $18, duration_ms = $19
             WHERE id = $20",
        )
            .bind(params.status.as_str())
            .bind(&params.error_type)
            .bind(&params.error_code)
            .bind(&params.error_message)
            .bind(&params.return_model)
            .bind(params.input_tokens)
            .bind(params.output_tokens)
            .bind(params.cache_read_tokens)
            .bind(params.cache_write_tokens)
            .bind(params.inference_tokens)
            .bind(params.read_tokens)
            .bind(params.write_tokens)
            .bind(params.total_tokens)
            .bind(params.http_status)
            .bind(&params.finish_reason)
            .bind(&params.provider_request_id)
            .bind(&params.response_content)
            .bind(params.end_time)
            .bind(params.duration_ms)
            .bind(id)
            .execute(&self.pool)
            .await
            .context("更新子请求日志失败")?;

        Ok(())
    }

    pub async fn find_by_main_request_id(&self, main_request_id: i64) -> Result<Vec<RequestSub>> {
        self.find_by_main_request_ids(&[main_request_id]).await
    }

    /// 批量查询多个主请求下的子请求，用于主请求分页时一次性装配子日志。
    pub async fn find_by_main_request_ids(
        &self,
        main_request_ids: &[i64],
    ) -> Result<Vec<RequestSub>> {
        if main_request_ids.is_empty() {
            return Ok(Vec::new());
        }

        let query = format!(
            "SELECT {COLUMNS} FROM request_sub
             WHERE main_request_id = ANY($1) ORDER BY create_time ASC"
        );

        let rows = sqlx::query(&query)
            .bind(main_request_ids)
            .fetch_all(&self.pool)
            .await
            .context("查询子请求日志失败")?;

        rows.into_iter().map(request_sub_from_row).collect()
    }

    pub async fn page(
        &self,
        pagination: &PaginationParams,
        conditions: &RequestSubQO,
    ) -> Result<PaginationResult<RequestSub>> {
        let mut count = QueryBuilder::<Postgres>::new("SELECT COUNT(*) AS total FROM request_sub");
        push_request_sub_conditions(&mut count, conditions);

        let total_row = count.build().fetch_one(&self.pool).await?;
        let total: i64 = total_row.get("total");

        let mut query = QueryBuilder::<Postgres>::new(format!("SELECT {COLUMNS} FROM request_sub"));
        push_request_sub_conditions(&mut query, conditions);
        query.push_pagination(pagination);

        let rows = query.build().fetch_all(&self.pool).await?;
        let records = rows
            .into_iter()
            .filter_map(|row| request_sub_from_row(row).ok())
            .collect();

        Ok(PaginationResult { total, records })
    }
}

fn request_sub_from_row(row: sqlx::postgres::PgRow) -> Result<RequestSub> {
    Ok(RequestSub {
        id: row.get("id"),
        main_request_id: row.get("main_request_id"),
        provider_id: row.get("provider_id"),
        provider_name: row.get("provider_name"),
        model: row.get("model"),
        provider_url: row.get("provider_url"),
        request_params: row.get("request_params"),
        status: RequestStatus::from_db(&row.get::<String, _>("status")),
        error_type: row.get("error_type"),
        error_code: row.get("error_code"),
        error_message: row.get("error_message"),
        return_model: row.get("return_model"),
        input_tokens: row.get("input_tokens"),
        output_tokens: row.get("output_tokens"),
        cache_read_tokens: row.get("cache_read_tokens"),
        cache_write_tokens: row.get("cache_write_tokens"),
        inference_tokens: row.get("inference_tokens"),
        read_tokens: row.get("read_tokens"),
        write_tokens: row.get("write_tokens"),
        total_tokens: row.get("total_tokens"),
        http_status: row.get("http_status"),
        finish_reason: row.get("finish_reason"),
        provider_request_id: row.get("provider_request_id"),
        response_content: row.get("response_content"),
        start_time: row.get("start_time"),
        end_time: row.get("end_time"),
        duration_ms: row.get("duration_ms"),
        create_time: row.get("create_time"),
    })
}

fn push_request_sub_conditions<'a>(
    query: &mut QueryBuilder<'a, Postgres>,
    conditions: &'a RequestSubQO,
) {
    query.push(" WHERE 1 = 1");
    query.eq("id", conditions.id);
    query.eq("main_request_id", conditions.main_request_id);
    query.eq("provider_id", conditions.provider_id);
    query.like("provider_name", conditions.provider_name.as_deref());
    query.like("model", conditions.model.as_deref());
    query.eq(
        "status",
        conditions.status.as_ref().map(|status| status.as_str()),
    );
}
