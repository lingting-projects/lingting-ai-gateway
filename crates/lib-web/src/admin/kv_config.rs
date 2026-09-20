use anyhow::Result;
use framework_core::types::{PaginationParams, PaginationResult};
use framework_web::{Json, web_api_post};
use service_admin::service::KvConfigService;
use types_admin::dto::{KvConfigQO, KvConfigUpdatePO, KvConfigVO};

/// 全局配置分页
#[web_api_post(path = "/___/kv-config/page")]
pub async fn kv_config_page(
    pagination: PaginationParams,
    Json(query): Json<KvConfigQO>,
) -> Result<PaginationResult<KvConfigVO>> {
    let page = KvConfigService::new()?.page(&pagination, &query).await?;

    Ok(PaginationResult {
        total: page.total,
        records: page.records.into_iter().map(KvConfigVO::from).collect(),
    })
}

/// 全局配置写入：config_key 存在则更新，不存在则新增。
#[web_api_post(path = "/___/kv-config/upsert")]
pub async fn kv_config_upsert(Json(params): Json<KvConfigUpdatePO>) -> Result<()> {
    KvConfigService::new()?.upsert(&params).await
}
