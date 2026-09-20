use anyhow::Result;
use framework_core::types::IdPO;
use framework_web::{Json, web_api_post};
use service_admin::service::{ProviderModelRedirectService, ProviderModelService};
use types_admin::dto::{
    ProviderModelRedirectCreatePO, ProviderModelRedirectCreateVO, ProviderModelRedirectUpdatePO,
    ProviderModelRedirectVO, ProviderModelVO,
};

/// 默认模型列表：返回全部默认模型数据，不含真实供应商。
#[web_api_post(path = "/___/provider-model/default")]
pub async fn provider_model_default() -> Result<Vec<ProviderModelVO>> {
    let models = ProviderModelService::new()?.find_default().await?;

    Ok(models
        .into_iter()
        .map(|model| ProviderModelVO::of(model, String::new()))
        .collect())
}

/// 模型名称映射列表，返回全部映射。
#[web_api_post(path = "/___/provider-model-redirect/list")]
pub async fn provider_model_redirect_list() -> Result<Vec<ProviderModelRedirectVO>> {
    let redirects = ProviderModelRedirectService::new()?.find_all().await?;

    Ok(redirects
        .into_iter()
        .map(ProviderModelRedirectVO::from)
        .collect())
}

/// 模型名称映射创建
#[web_api_post(path = "/___/provider-model-redirect/create")]
pub async fn provider_model_redirect_create(
    Json(params): Json<ProviderModelRedirectCreatePO>,
) -> Result<ProviderModelRedirectCreateVO> {
    let id = ProviderModelRedirectService::new()?.create(&params).await?;
    Ok(ProviderModelRedirectCreateVO { id })
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
