use anyhow::Result;
use framework_core::types::{IdPO, PaginationParams, PaginationResult};
use framework_web::{Json, web_api_post};
use service_admin::service::ProviderModelRedirectService;
use types_admin::dto::{
    ProviderModelRedirectCreatePO, ProviderModelRedirectQO, ProviderModelRedirectUpdatePO,
    ProviderModelRedirectVO,
};

/// 模型名称映射分页
#[web_api_post(path = "/___/provider-model-redirect/page")]
pub async fn provider_model_redirect_page(
    pagination: PaginationParams,
    Json(query): Json<ProviderModelRedirectQO>,
) -> Result<PaginationResult<ProviderModelRedirectVO>> {
    let page = ProviderModelRedirectService::new()?
        .page(&pagination, &query)
        .await?;

    Ok(PaginationResult {
        total: page.total,
        records: page
            .records
            .into_iter()
            .map(ProviderModelRedirectVO::from)
            .collect(),
    })
}

/// 模型名称映射创建
#[web_api_post(path = "/___/provider-model-redirect/create")]
pub async fn provider_model_redirect_create(
    Json(params): Json<ProviderModelRedirectCreatePO>,
) -> Result<i64> {
    ProviderModelRedirectService::new()?.create(&params).await
}

/// 模型名称映射更新
#[web_api_post(path = "/___/provider-model-redirect/update")]
pub async fn provider_model_redirect_update(
    Json(params): Json<ProviderModelRedirectUpdatePO>,
) -> Result<()> {
    ProviderModelRedirectService::new()?.update(&params).await
}

/// 模型名称映射删除
#[web_api_post(path = "/___/provider-model-redirect/delete")]
pub async fn provider_model_redirect_delete(Json(params): Json<IdPO>) -> Result<()> {
    ProviderModelRedirectService::new()?.delete(params.id).await
}
