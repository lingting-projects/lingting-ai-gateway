use anyhow::Result;
use framework_core::types::{PaginationParams, PaginationResult};
use framework_web::{Json, web_api_post};
use service_admin::manager::RequestManager;
use service_admin::service::RequestService;
use types_admin::dto::{
    RequestMainDetailVO, RequestMainQO, RequestMainVO, RequestSubQO, RequestSubVO,
};

/// 主请求日志分页
#[web_api_post(path = "/___/request/main/page")]
pub async fn request_main_page(
    pagination: PaginationParams,
    Json(query): Json<RequestMainQO>,
) -> Result<PaginationResult<RequestMainVO>> {
    let page = RequestService::new()?
        .main_page(&pagination, &query)
        .await?;

    Ok(PaginationResult {
        total: page.total,
        records: page.records.into_iter().map(RequestMainVO::from).collect(),
    })
}

/// 子请求日志分页
#[web_api_post(path = "/___/request/sub/page")]
pub async fn request_sub_page(
    pagination: PaginationParams,
    Json(query): Json<RequestSubQO>,
) -> Result<PaginationResult<RequestSubVO>> {
    let page = RequestService::new()?.sub_page(&pagination, &query).await?;

    Ok(PaginationResult {
        total: page.total,
        records: page.records.into_iter().map(RequestSubVO::from).collect(),
    })
}

/// 主请求日志分页，附带每条主日志下的子请求日志。
#[web_api_post(path = "/___/request/main-info/page")]
pub async fn request_main_info_page(
    pagination: PaginationParams,
    Json(query): Json<RequestMainQO>,
) -> Result<PaginationResult<RequestMainDetailVO>> {
    RequestManager::new()?
        .page_detail(&pagination, &query)
        .await
}
