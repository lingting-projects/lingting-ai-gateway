use super::*;

/// API Key 列表

#[web_api_post(path = "/___/api-key/list")]
pub async fn list() -> Result<Json<serde_json::Value>> {
    let web_context = use_web()?;
    let db_context = use_db()?;

    // TODO: 实现 API Key 列表查询
    let repository = KvConfigRepository::new(db_context.pool.clone());
    let api_keys = repository.find_by_prefix("api_key_").await?;

    let records = api_keys.into_iter().map(|config| {
        serde_json::json!({
            "id": config.id,
            "name": config.key.replace("api_key_", ""),
            "key_hash": config.value,
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

/// API Key 创建

#[web_api_post(path = "/___/api-key/create")]
pub async fn create() -> Result<Json<serde_json::Value>> {
    let web_context = use_web()?;
    let db_context = use_db()?;

    // 解析请求体
    let body = web_context.body_json::<serde_json::Value>()
        .await
        .map_err(|e| anyhow!("Failed to parse request body: {}", e))?;

    let name = body.get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| anyhow!("API Key name is required"))?;

    let key = body.get("key")
        .and_then(|k| k.as_str())
        .ok_or_else(|| anyhow!("API Key is required"))?;

    // 生成 key_hash
    let key_hash = sha1::Sha1::from(key).to_string();

    // 创建配置
    let config = KvConfig {
        id: 0, // 由数据库生成
        key: format!("api_key_{}", name),
        value: key_hash,
        description: Some(format!("API Key: {}", name)),
        create_time: lib_core::current_millis()?,
        update_time: lib_core::current_millis()?,
    };

    let repository = KvConfigRepository::new(db_context.pool.clone());
    repository.create(&config).await?;

    Ok(Json(serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": {
            "name": name,
            "key": "***", // 返回隐藏的 key
            "key_hash": key_hash,
        }
    })))
}

/// API Key 更新

#[web_api_post(path = "/___/api-key/update")]
pub async fn update() -> Result<Json<serde_json::Value>> {
    let web_context = use_web()?;
    let db_context = use_db()?;

    // 解析请求体
    let body = web_context.body_json::<serde_json::Value>()
        .await
        .map_err(|e| anyhow!("Failed to parse request body: {}", e))?;

    let id = body.get("id")
        .and_then(|id| id.as_i64())
        .ok_or_else(|| anyhow!("API Key ID is required"))?;

    let key = body.get("key")
        .and_then(|k| k.as_str());

    let repository = KvConfigRepository::new(db_context.pool.clone());
    let mut config = repository.find_by_id(id).await?
        .ok_or_else(|| anyhow!("API Key not found"))?;

    if let Some(new_key) = key {
        // 更新 key
        config.value = sha1::Sha1::from(new_key).to_string();
        config.update_time = lib_core::current_millis()?;
    }

    repository.update(&config).await?;

    Ok(Json(serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": {
            "id": config.id,
            "name": config.key.replace("api_key_", ""),
            "key_hash": config.value,
        }
    })))
}

/// API Key 删除

#[web_api_post(path = "/___/api-key/delete")]
pub async fn delete() -> Result<Json<serde_json::Value>> {
    let web_context = use_web()?;
    let db_context = use_db()?;

    // 解析请求体
    let body = web_context.body_json::<serde_json::Value>()
        .await
        .map_err(|e| anyhow!("Failed to parse request body: {}", e))?;

    let id = body.get("id")
        .and_then(|id| id.as_i64())
        .ok_or_else(|| anyhow!("API Key ID is required"))?;

    let repository = KvConfigRepository::new(db_context.pool.clone());
    repository.delete(id).await?;

    Ok(Json(serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": null
    })))
}
