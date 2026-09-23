use anyhow::{Context, Result};
use framework_core::types::{PaginationParams, PaginationResult};
use lib_db::{PgPool, QueryBuilderExt};
use sqlx::{Postgres, QueryBuilder, Row};
use types_admin::dto::{
    DashboardQO, DashboardRequestVO, DashboardTokenGroup, DashboardTokenVO, RequestFinishPO,
    RequestSubCreatePO, RequestSubQO,
};
use types_admin::entity::{RequestStatus, RequestSub};

use super::dashboard_row::{TOKEN_COLUMNS, dashboard_token_vo};

/// 子请求日志数据访问。
pub struct RequestSubRepository {
    pool: PgPool,
}

const COLUMNS: &str = "id, main_request_id, provider_id, provider_name, model, provider_url,
    request_params, request_headers, status, error_type, error_code, error_message, return_model,
    input_tokens,
    output_tokens, cache_read_tokens, cache_write_tokens, inference_tokens, total_tokens,
    http_status, finish_reason, provider_request_id, response_content, response_headers,
    start_time, first_chunk_time, end_time, duration_ms, create_time";

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
                 request_headers, status, start_time, create_time)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             RETURNING id",
        )
        .bind(params.main_request_id)
        .bind(params.provider_id)
        .bind(&params.provider_name)
        .bind(&params.model)
        .bind(&params.provider_url)
        .bind(&params.request_params)
        .bind(&params.request_headers)
        .bind(RequestStatus::Processing.as_str())
        .bind(params.start_time)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .context("写入子请求日志失败")?;

        Ok(row.get("id"))
    }

    /// 记录转发开始时间；由转发回调在请求发出前调用。
    pub async fn update_start_time(&self, id: i64, start_time: i64) -> Result<()> {
        sqlx::query("UPDATE request_sub SET start_time = $1 WHERE id = $2")
            .bind(start_time)
            .bind(id)
            .execute(&self.pool)
            .await
            .context("更新子请求开始时间失败")?;

        Ok(())
    }

    /// 记录首字时间；由转发回调在首个分片到达时调用。
    pub async fn update_first_chunk_time(&self, id: i64, first_chunk_time: i64) -> Result<()> {
        sqlx::query("UPDATE request_sub SET first_chunk_time = $1 WHERE id = $2")
            .bind(first_chunk_time)
            .bind(id)
            .execute(&self.pool)
            .await
            .context("更新子请求首字时间失败")?;

        Ok(())
    }

    /// 请求结束时写入状态、错误、token 与返回信息。
    pub async fn finish(&self, id: i64, params: &RequestFinishPO) -> Result<()> {
        sqlx::query(
            "UPDATE request_sub SET
                status = $1, error_type = COALESCE($2, ''), error_code = COALESCE($3, ''),
                error_message = COALESCE($4, ''),
                return_model = COALESCE($5, ''), input_tokens = $6, output_tokens = $7,
                cache_read_tokens = $8, cache_write_tokens = $9, inference_tokens = $10,
                total_tokens = $11, http_status = $12,
                finish_reason = COALESCE($13, ''), provider_request_id = COALESCE($14, ''),
                response_content = $15,
                response_headers = $16, end_time = $17,
                first_chunk_time = COALESCE($18, first_chunk_time), duration_ms = $17 - start_time
             WHERE id = $19",
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
        .bind(params.total_tokens)
        .bind(params.http_status)
        .bind(&params.finish_reason)
        .bind(&params.provider_request_id)
        .bind(&params.response_content)
        .bind(&params.response_headers)
        .bind(params.end_time)
        .bind(params.first_chunk_time)
        .bind(id)
        .execute(&self.pool)
        .await
        .context("更新子请求日志失败")?;

        Ok(())
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

    /// 按筛选条件统计子请求数量：总数、进行中、失败、取消。
    pub async fn dashboard_request(&self, conditions: &DashboardQO) -> Result<DashboardRequestVO> {
        let mut query = QueryBuilder::<Postgres>::new("SELECT COUNT(*) AS total,");
        query
            .push(" COUNT(*) FILTER (WHERE status = ")
            .push_bind(RequestStatus::Processing.as_str())
            .push(") AS processing, COUNT(*) FILTER (WHERE status = ")
            .push_bind(RequestStatus::Failed.as_str())
            .push(") AS failed, COUNT(*) FILTER (WHERE status = ")
            .push_bind(RequestStatus::Cancelled.as_str())
            .push(") AS cancelled FROM request_sub");
        push_dashboard_sub_conditions(&mut query, conditions);

        let row = query.build().fetch_one(&self.pool).await?;
        Ok(DashboardRequestVO {
            total: row.get("total"),
            processing: row.get("processing"),
            failed: row.get("failed"),
            cancelled: row.get("cancelled"),
        })
    }

    /// 按筛选条件统计子请求日志的 token。
    pub async fn dashboard_token(&self, conditions: &DashboardQO) -> Result<DashboardTokenVO> {
        let mut query = QueryBuilder::<Postgres>::new("SELECT ");
        query.push(TOKEN_COLUMNS);
        query.push(" FROM request_sub");
        push_dashboard_sub_conditions(&mut query, conditions);

        let row = query.build().fetch_one(&self.pool).await?;
        Ok(dashboard_token_vo(&row))
    }

    /// 按筛选条件统计子请求日志的 token，按所在电脑时区的天分组，
    /// 并可继续按供应商、模型分组。
    pub async fn dashboard_token_group(
        &self,
        conditions: &DashboardQO,
    ) -> Result<Vec<DashboardTokenGroup>> {
        let timezone_offset = timezone_offset_millis();
        let mut query = QueryBuilder::<Postgres>::new(format!(
            "SELECT to_char(to_timestamp((start_time + {timezone_offset}) / 1000), 'YYYY-MM-DD') AS day"
        ));
        query.push(", ").push(TOKEN_COLUMNS);
        if conditions.with_provider {
            query.push(", provider_id");
        }
        if conditions.with_model {
            query.push(", model");
        }
        query.push(" FROM request_sub");
        push_dashboard_sub_conditions(&mut query, conditions);

        // 分组与排序按 SELECT 中的列序号引用，依次为天、供应商、模型。
        let group_count =
            1 + usize::from(conditions.with_provider) + usize::from(conditions.with_model);
        let group_columns = (1..=group_count)
            .map(|index| index.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        query
            .push(format!(" GROUP BY {group_columns}"))
            .push(format!(" ORDER BY {group_columns}"));

        let rows = query.build().fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|row| {
                let token = dashboard_token_vo(&row);
                Ok(DashboardTokenGroup {
                    day: row.get("day"),
                    provider_id: conditions.with_provider.then(|| row.get("provider_id")),
                    model: conditions.with_model.then(|| row.get("model")),
                    total: token.total,
                    cache_read: token.cache_read,
                    cache_write: token.cache_write,
                    input: token.input,
                    output: token.output,
                })
            })
            .collect()
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
        request_headers: row.get("request_headers"),
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
        total_tokens: row.get("total_tokens"),
        http_status: row.get("http_status"),
        finish_reason: row.get("finish_reason"),
        provider_request_id: row.get("provider_request_id"),
        response_content: row.get("response_content"),
        response_headers: row.get("response_headers"),
        start_time: row.get("start_time"),
        first_chunk_time: row.get("first_chunk_time"),
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

/// 追加仪表盘统计筛选条件；筛选值一律参数绑定，数组为空时不追加条件。
fn push_dashboard_sub_conditions(query: &mut QueryBuilder<Postgres>, conditions: &DashboardQO) {
    query.push(" WHERE 1 = 1");
    query.ge("start_time", conditions.start_time);
    query.le("start_time", conditions.end_time);
    query.in_array("provider_id", conditions.provider_ids.as_deref());
    query.in_array("model", conditions.models.as_deref());
}

/// 当前电脑时区相对 UTC 的偏移毫秒数；pglite 无时区数据，因此由本地时钟提供。
fn timezone_offset_millis() -> i32 {
    use chrono::Offset;

    chrono::Local::now().offset().fix().local_minus_utc() * 1000
}
