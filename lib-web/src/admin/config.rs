use super::*;

/// 配置列表

pub async fn list() -> Result<Json<serde_json::Value>> {
    let web_context = use_web().ok_or_else(|| anyhow!("Web context not found"))?;
    let db_context = use_db().ok_or_else(|| anyhow!("Database context not found"))?;

    let repository = KvConfigRepository::new(db_context.pool.clone());
    let configs = repository.find_by_prefix("").await?;

    let records = configs.into_iter().map(|config| {
        serde_json::json!({
            "id": config.id,
            "key": config.key,
            "value": config.value,
            "description": config.description,
            "create_time": config.create_time,
            "update_time": config.update_time,
        })
    }).collect::<Vec<_>>();

    Ok(Json(serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": {
            "total": records.len(),
            "records": records,
        }
    })))
}

/// 配置更新

pub async fn update() -> Result<Json<serde_json::Value>> {
    let web_context = use_web().ok_or_else(|| anyhow!("Web context not found"))?;
    let db_context = use_db().ok_or_else(|| anyhow!("Database context not found"))?;

    // 解析请求体
    let body = web_context.body_json::<serde_json::Value>()
        .await
        .map_err(|e| anyhow!("Failed to parse request body: {}", e))?;

    let id = body.get("id")
        .and_then(|id| id.as_i64())
        .ok_or_else(|| anyhow!("Config ID is required"))?;

    let value = body.get("value")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Config value is required"))?;

    let repository = KvConfigRepository::new(db_context.pool.clone());
    let mut config = repository.find_by_id(id).await?
        .ok_or_else(|| anyhow!("Config not found"))?;

    config.value = value.to_string();
    config.update_time = lib_core::current_millis()?;

    repository.update(&config).await?;

    Ok(Json(serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": {
            "id": config.id,
            "key": config.key,
            "value": config.value,
        }
    })))
}
