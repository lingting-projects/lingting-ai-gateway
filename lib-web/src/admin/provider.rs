use super::*;

/// 供应商列表

#[web_api_post(path = "/___/provider/list")]
pub async fn list() -> Result<Json<serde_json::Value>> {
    let web_context = use_web()?;
    let db_context = use_db()?;

    // 解析查询参数
    let query_params = web_context.query_json::<ProviderQO>()
        .await
        .unwrap_or_default();

    let repository = ProviderRepository::new(db_context.pool.clone());
    let result = repository.page(&query_params.pagination, &query_params).await?;

    Ok(Json(serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": result
    })))
}

/// 供应商创建

#[web_api_post(path = "/___/provider/create")]
pub async fn create() -> Result<Json<serde_json::Value>> {
    let web_context = use_web()?;
    let db_context = use_db()?;

    // 解析请求体
    let body = web_context.body_json::<serde_json::Value>()
        .await
        .map_err(|e| anyhow!("Failed to parse request body: {}", e))?;

    let create_data = serde_json::from_value::<types_admin::dto::ProviderCreatePO>(body)
        .map_err(|e| anyhow!("Failed to parse provider data: {}", e))?;

    let repository = ProviderRepository::new(db_context.pool.clone());
    let id = repository.create(&create_data).await?;

    Ok(Json(serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": {
            "id": id,
        }
    })))
}

/// 供应商更新

#[web_api_post(path = "/___/provider/update")]
pub async fn update() -> Result<Json<serde_json::Value>> {
    let web_context = use_web()?;
    let db_context = use_db()?;

    // 解析请求体
    let body = web_context.body_json::<serde_json::Value>()
        .await
        .map_err(|e| anyhow!("Failed to parse request body: {}", e))?;

    let id = body.get("id")
        .and_then(|id| id.as_i64())
        .ok_or_else(|| anyhow!("Provider ID is required"))?;

    let update_data = serde_json::from_value::<types_admin::dto::ProviderUpdatePO>(body)
        .map_err(|e| anyhow!("Failed to parse provider data: {}", e))?;

    let repository = ProviderRepository::new(db_context.pool.clone());
    repository.update(id, &update_data).await?;

    // 触发供应商模型更新
    // TODO: 实现供应商模型更新逻辑

    Ok(Json(serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": {
            "id": id,
        }
    })))
}

/// 供应商删除

#[web_api_post(path = "/___/provider/delete")]
pub async fn delete() -> Result<Json<serde_json::Value>> {
    let web_context = use_web()?;
    let db_context = use_db()?;

    // 解析请求体
    let body = web_context.body_json::<serde_json::Value>()
        .await
        .map_err(|e| anyhow!("Failed to parse request body: {}", e))?;

    let id = body.get("id")
        .and_then(|id| id.as_i64())
        .ok_or_else(|| anyhow!("Provider ID is required"))?;

    let repository = ProviderRepository::new(db_context.pool.clone());
    repository.delete(id).await?;

    Ok(Json(serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": null
    })))
}

/// 供应商模型更新

#[web_api_post(path = "/___/provider/model-update")]
pub async fn model_update() -> Result<Json<serde_json::Value>> {
    // TODO: 实现供应商模型更新逻辑
    Ok(Json(serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": null
    })))
}
