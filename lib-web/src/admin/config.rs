use anyhow::Result;
use framework_core::types::{PaginationParams, PaginationResult};
use framework_web::{Json, web_api_post};
use service_admin::service::ConfigService;
use types_admin::dto::{ConfigQO, ConfigUpdatePO, ConfigVO};

/// 配置分页
#[web_api_post(path = "/___/config/page")]
pub async fn config_page(
    pagination: PaginationParams,
    Json(query): Json<ConfigQO>,
) -> Result<PaginationResult<ConfigVO>> {
    let page = ConfigService::new()?.page(&pagination, &query).await?;

    Ok(PaginationResult {
        total: page.total,
        records: page.records.into_iter().map(ConfigVO::from).collect(),
    })
}

/// 配置写入：config_key 存在则更新，不存在则新增。
#[web_api_post(path = "/___/config/upsert")]
pub async fn config_upsert(Json(params): Json<ConfigUpdatePO>) -> Result<()> {
    ConfigService::new()?.upsert(&params).await
}
