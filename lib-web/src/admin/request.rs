use super::*;

/// 请求日志列表

pub async fn list() -> Result<Json<serde_json::Value>> {
    let web_context = use_web().ok_or_else(|| anyhow!("Web context not found"))?;
    let db_context = use_db().ok_or_else(|| anyhow!("Database context not found"))?;

    // 解析查询参数
    let query_params = web_context.query_json::<RequestMainQO>()
        .await
        .unwrap_or_default();

    let repository = RequestMainRepository::new(db_context.pool.clone());
    let result = repository.page(&query_params.pagination, &query_params).await?;

    Ok(Json(serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": result
    })))
}

/// 请求日志详情

pub async fn detail() -> Result<Json<serde_json::Value>> {
    let web_context = use_web().ok_or_else(|| anyhow!("Web context not found"))?;
    let db_context = use_db().ok_or_else(|| anyhow!("Database context not found"))?;

    // 解析请求体
    let body = web_context.body_json::<serde_json::Value>()
        .await
        .map_err(|e| anyhow!("Failed to parse request body: {}", e))?;

    let id = body.get("id")
        .and_then(|id| id.as_i64())
        .ok_or_else(|| anyhow!("Request ID is required"))?;

    let repository = RequestMainRepository::new(db_context.pool.clone());
    let request = repository.find_by_id(id).await?
        .ok_or_else(|| anyhow!("Request not found"))?;

    // 查询子请求日志
    let sub_repository = RequestSubRepository::new(db_context.pool.clone());
    let sub_requests = sub_repository.find_by_main_request_id(id).await?;

    Ok(Json(serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": {
            "request": request,
            "sub_requests": sub_requests,
        }
    })))
}

/// 子请求日志列表

pub async fn sub_list() -> Result<Json<serde_json::Value>> {
    let web_context = use_web().ok_or_else(|| anyhow!("Web context not found"))?;
    let db_context = use_db().ok_or_else(|| anyhow!("Database context not found"))?;

    // 解析查询参数
    let query_params = web_context.query_json::<RequestSubQO>()
        .await
        .unwrap_or_default();

    let repository = RequestSubRepository::new(db_context.pool.clone());
    let result = repository.page(&query_params.pagination, &query_params).await?;

    Ok(Json(serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": result
    })))
}
