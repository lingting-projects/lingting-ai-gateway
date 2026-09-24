use anyhow::{Context, Result};
use framework_core::types::{PaginationParams, PaginationResult};
use lib_db::{PgPool, QueryBuilderExt};
use sqlx::{Postgres, QueryBuilder, Row};
use types_admin::dto::{
    DashboardQO, DashboardRequestVO, DashboardTokenVO, RequestFinishPO, RequestMainCreatePO,
    RequestMainQO,
};
use types_admin::entity::{RequestMain, RequestStatus};

use super::dashboard_row::{TOKEN_COLUMNS, dashboard_token_vo};

/// 主请求日志数据访问。
pub struct RequestMainRepository {
    pool: PgPool,
}

const COLUMNS: &str = "id, trace_id, client_request_id, session_id, client_ip,
    user_agent, credential_id, model, stream, method, path, request_params, request_headers,
    status,
    error_type, error_code, error_message, return_model, input_tokens,
    output_tokens, cache_read_tokens, cache_write_tokens, inference_tokens, total_tokens,
    http_status, finish_reason, provider_request_id, response_content, response_headers,
    start_time, first_chunk_time, end_time, duration_ms, create_time";

impl RequestMainRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<RequestMain>> {
        let query = format!("SELECT {COLUMNS} FROM request_main WHERE id = $1 LIMIT 1");

        let row = sqlx::query(&query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .context("查询主请求日志失败")?;

        row.map(request_main_from_row).transpose()
    }

    /// 写入主请求日志，返回日志主键。
    pub async fn create(&self, params: &RequestMainCreatePO) -> Result<i64> {
        let now = lib_core::current_millis()?;

        let row = sqlx::query(
            "INSERT INTO request_main
                (trace_id, client_request_id, session_id, client_ip, user_agent,
                 credential_id, model, stream, method, path, request_params, request_headers,
                 status, start_time, create_time)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
             RETURNING id",
        )
        .bind(&params.trace_id)
        .bind(&params.client_request_id)
        .bind(&params.session_id)
        .bind(&params.client_ip)
        .bind(&params.user_agent)
        .bind(&params.credential_id)
        .bind(&params.model)
        .bind(params.stream)
        .bind(&params.method)
        .bind(&params.path)
        .bind(&params.request_params)
        .bind(&params.request_headers)
        .bind(RequestStatus::Processing.as_str())
        .bind(params.start_time)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .context("写入主请求日志失败")?;

        Ok(row.get("id"))
    }

    /// 请求结束时写入状态、错误、token 与返回信息。
    pub async fn finish(&self, id: i64, params: &RequestFinishPO) -> Result<()> {
        sqlx::query(
            "UPDATE request_main SET
                status = $1, error_type = COALESCE($2, ''), error_code = COALESCE($3, ''),
                error_message = COALESCE($4, ''),
                return_model = COALESCE($5, ''), input_tokens = $6, output_tokens = $7,
                cache_read_tokens = $8, cache_write_tokens = $9, inference_tokens = $10,
                total_tokens = $11, http_status = $12,
                finish_reason = COALESCE($13, ''), provider_request_id = COALESCE($14, ''),
                response_content = $15,
                response_headers = $16, end_time = $17,
                first_chunk_time = COALESCE($18, first_chunk_time),
                duration_ms = $17 - start_time
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
        .context("更新主请求日志失败")?;

        Ok(())
    }

    pub async fn page(
        &self,
        pagination: &PaginationParams,
        conditions: &RequestMainQO,
    ) -> Result<PaginationResult<RequestMain>> {
        let mut count = QueryBuilder::<Postgres>::new("SELECT COUNT(*) AS total FROM request_main");
        push_request_main_conditions(&mut count, conditions);

        let total_row = count.build().fetch_one(&self.pool).await?;
        let total: i64 = total_row.get("total");

        let mut query =
            QueryBuilder::<Postgres>::new(format!("SELECT {COLUMNS} FROM request_main"));
        push_request_main_conditions(&mut query, conditions);
        query.push_pagination(pagination);

        let rows = query.build().fetch_all(&self.pool).await?;
        let records = rows
            .into_iter()
            .filter_map(|row| request_main_from_row(row).ok())
            .collect();

        Ok(PaginationResult { total, records })
    }

    /// 按筛选条件统计请求数量：总数、进行中、失败、取消。
    pub async fn dashboard_request(&self, conditions: &DashboardQO) -> Result<DashboardRequestVO> {
        let mut query = QueryBuilder::<Postgres>::new("SELECT COUNT(*) AS total,");
        query
            .push(" COUNT(*) FILTER (WHERE status = ")
            .push_bind(RequestStatus::Processing.as_str())
            .push(") AS processing, COUNT(*) FILTER (WHERE status = ")
            .push_bind(RequestStatus::Failed.as_str())
            .push(") AS failed, COUNT(*) FILTER (WHERE status = ")
            .push_bind(RequestStatus::Cancelled.as_str())
            .push(") AS cancelled FROM request_main");
        push_dashboard_main_conditions(&mut query, conditions);

        let row = query.build().fetch_one(&self.pool).await?;
        Ok(DashboardRequestVO {
            total: row.get("total"),
            processing: row.get("processing"),
            failed: row.get("failed"),
            cancelled: row.get("cancelled"),
        })
    }

    /// 按筛选条件统计主请求日志的 token。
    pub async fn dashboard_token(&self, conditions: &DashboardQO) -> Result<DashboardTokenVO> {
        let mut query = QueryBuilder::<Postgres>::new("SELECT ");
        query.push(TOKEN_COLUMNS);
        query.push(" FROM request_main");
        push_dashboard_main_conditions(&mut query, conditions);

        let row = query.build().fetch_one(&self.pool).await?;
        Ok(dashboard_token_vo(&row))
    }
}

fn request_main_from_row(row: sqlx::postgres::PgRow) -> Result<RequestMain> {
    Ok(RequestMain {
        id: row.get("id"),
        trace_id: row.get("trace_id"),
        client_request_id: row.get("client_request_id"),
        session_id: row.get("session_id"),
        client_ip: row.get("client_ip"),
        user_agent: row.get("user_agent"),
        credential_id: row.get("credential_id"),
        model: row.get("model"),
        stream: row.get("stream"),
        method: row.get("method"),
        path: row.get("path"),
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

fn push_request_main_conditions<'a>(
    query: &mut QueryBuilder<'a, Postgres>,
    conditions: &'a RequestMainQO,
) {
    query.push(" WHERE 1 = 1");
    query.eq("id", conditions.id);
    query.like("client_request_id", conditions.client_request_id.as_deref());
    query.eq("session_id", conditions.session_id.as_deref());
    query.like("model", conditions.model.as_deref());
    query.eq("credential_id", conditions.credential_id.as_deref());
    query.like("client_ip", conditions.client_ip.as_deref());
    query.eq(
        "status",
        conditions.status.as_ref().map(|status| status.as_str()),
    );
    query.ge("start_time", conditions.start_time_begin);
    query.le("start_time", conditions.start_time_end);
}

/// 追加仪表盘统计筛选条件；主请求表没有供应商字段，忽略 `provider_ids`。
fn push_dashboard_main_conditions(query: &mut QueryBuilder<Postgres>, conditions: &DashboardQO) {
    query.push(" WHERE 1 = 1");
    query.ge("start_time", conditions.start_time);
    query.le("start_time", conditions.end_time);
    query.in_array("model", conditions.models.as_deref());
}
