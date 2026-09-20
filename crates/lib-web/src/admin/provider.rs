use anyhow::{Result, anyhow};
use framework_core::types::IdPO;
use framework_web::{Json, web_api_post};
use lib_db::use_db;
use service_admin::manager::ProviderManager;
use service_admin::service::ProviderService;
use types_admin::dto::{ProviderCreatePO, ProviderDetailVO, ProviderUpdatePO, ProviderVO};

/// 供应商列表，附带每个供应商下的全部模型，按供应商路由策略排序。
#[web_api_post(path = "/___/provider/list")]
pub async fn provider_list() -> Result<Vec<ProviderDetailVO>> {
    ProviderManager::new()?.list_detail().await
}

/// 供应商创建，返回仅此一次可见的明文 key。
#[web_api_post(path = "/___/provider/create")]
pub async fn provider_create(Json(params): Json<ProviderCreatePO>) -> Result<ProviderVO> {
    let service = ProviderService::new()?;
    let id = service.create(&params).await?;
    let provider = service
        .find_by_id(id)
        .await?
        .ok_or_else(|| anyhow!("供应商创建后查询不到记录：{id}"))?;

    let mut vo = ProviderVO::from(provider);
    vo.api_key = params.api_key;
    Ok(vo)
}

/// 供应商更新，仅允许修改展示名、地址、key、优先级、启用状态与扩展配置。
#[web_api_post(path = "/___/provider/update")]
pub async fn provider_update(Json(params): Json<ProviderUpdatePO>) -> Result<()> {
    ProviderService::new()?.update(&params).await
}

/// 供应商逻辑删除。
#[web_api_post(path = "/___/provider/delete")]
pub async fn provider_delete(Json(params): Json<IdPO>) -> Result<()> {
    ProviderService::new()?.delete(params.id).await
}

/// 供应商模型同步：启动异步任务拉取供应商模型并更新本地配置，立即返回。
#[web_api_post(path = "/___/provider/update-model")]
pub async fn provider_update_model(Json(params): Json<IdPO>) -> Result<()> {
    service_provider::models_fetch::async_run(use_db()?, Some(params.id)).await;
    Ok(())
}
