//! 仪表盘统计的公共 SQL 片段与结果映射；由各请求日志 Repository 共用。

use sqlx::Row;
use types_admin::dto::DashboardTokenVO;

/// 统计结果中的 token 列；总数与缓存读写、输入输出统一别名。
///
/// `SUM(bigint)` 在 PostgreSQL 中返回 `numeric`，必须显式转回 `BIGINT`，
/// 否则解码到 `i64` 会报类型不匹配。
pub const TOKEN_COLUMNS: &str = "CAST(COALESCE(SUM(total_tokens), 0) AS BIGINT) AS total,
    CAST(COALESCE(SUM(cache_read_tokens), 0) AS BIGINT) AS cache_read,
    CAST(COALESCE(SUM(cache_write_tokens), 0) AS BIGINT) AS cache_write,
    CAST(COALESCE(SUM(input_tokens), 0) AS BIGINT) AS input,
    CAST(COALESCE(SUM(output_tokens), 0) AS BIGINT) AS output";

/// 读取查询结果中的 token 统计列，统一 `dashboard_token` 与分组统计的取值方式。
pub fn dashboard_token_vo(row: &sqlx::postgres::PgRow) -> DashboardTokenVO {
    DashboardTokenVO {
        total: row.get("total"),
        cache_read: row.get("cache_read"),
        cache_write: row.get("cache_write"),
        input: row.get("input"),
        output: row.get("output"),
    }
}
