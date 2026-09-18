use anyhow::Result;
use framework_core::types::{IdPO, PaginationParams, PaginationResult};
use framework_web::{Json, web_api_post};
use service_admin::service::ApiKeyService;
use types_admin::dto::{ApiKeyCreatePO, ApiKeyQO, ApiKeyUpdatePO, ApiKeyVO};

/// API Key 分页
#[web_api_post(path = "/___/api-key/page")]
pub async fn api_key_page(
    pagination: PaginationParams,
    Json(query): Json<ApiKeyQO>,
) -> Result<PaginationResult<ApiKeyVO>> {
    let page = ApiKeyService::new()?.page(&pagination, &query).await?;

    Ok(PaginationResult {
        total: page.total,
        records: page.records.into_iter().map(ApiKeyVO::from).collect(),
    })
}

/// API Key 创建，返回仅此一次可见的明文 key。
#[web_api_post(path = "/___/api-key/create")]
pub async fn api_key_create(Json(params): Json<ApiKeyCreatePO>) -> Result<String> {
    let service = ApiKeyService::new()?;
    let (_, key) = service.create(&params).await?;
    Ok(key)
}

/// API Key 更新，仅允许修改名称、启用状态与备注。
#[web_api_post(path = "/___/api-key/update")]
pub async fn api_key_update(Json(params): Json<ApiKeyUpdatePO>) -> Result<()> {
    ApiKeyService::new()?.update(&params).await
}

/// API Key 逻辑删除。
#[web_api_post(path = "/___/api-key/delete")]
pub async fn api_key_delete(Json(params): Json<IdPO>) -> Result<()> {
    ApiKeyService::new()?.delete(params.id).await
}
