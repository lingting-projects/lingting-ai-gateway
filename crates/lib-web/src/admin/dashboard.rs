use anyhow::Result;
use framework_web::{Json, web_api_post};
use service_admin::manager::DashboardManager;
use types_admin::dto::{DashboardQO, DashboardRequestVO, DashboardTokenFilterVO, DashboardTokenVO};

/// 主请求数量统计：总数、进行中、失败、取消。
#[web_api_post(path = "/___/dashboard/request-main")]
pub async fn dashboard_request_main(Json(query): Json<DashboardQO>) -> Result<DashboardRequestVO> {
    DashboardManager::new()?.request_main(&query).await
}

/// 子请求数量统计：总数、进行中、失败、取消。
#[web_api_post(path = "/___/dashboard/request-sub")]
pub async fn dashboard_request_sub(Json(query): Json<DashboardQO>) -> Result<DashboardRequestVO> {
    DashboardManager::new()?.request_sub(&query).await
}

/// 主请求 Token 统计。
#[web_api_post(path = "/___/dashboard/token-main")]
pub async fn dashboard_token_main(Json(query): Json<DashboardQO>) -> Result<DashboardTokenVO> {
    DashboardManager::new()?.token_main(&query).await
}

/// 子请求 Token 统计。
#[web_api_post(path = "/___/dashboard/token-sub")]
pub async fn dashboard_token_sub(Json(query): Json<DashboardQO>) -> Result<DashboardTokenVO> {
    DashboardManager::new()?.token_sub(&query).await
}

/// 子请求 Token 统计，按天（可再按供应商、模型）分组。
#[web_api_post(path = "/___/dashboard/token-sub-filter")]
pub async fn dashboard_token_sub_filter(
    Json(query): Json<DashboardQO>,
) -> Result<Vec<DashboardTokenFilterVO>> {
    DashboardManager::new()?.token_sub_filter(&query).await
}
