use anyhow::{Result, anyhow};
use framework_web::{Json, WebError, catch_panic, use_web};
use lib_db::{DbContext, use_db, RequestMainRepository, RequestSubRepository, ProviderRepository, KvConfigRepository};
use lib_web_core::{AppContext, use_app, Authorization, AuthType};
use types_admin::entity::{RequestMain, RequestSub, Provider, KvConfig};
use types_admin::dto::{RequestMainQO, RequestSubQO, ProviderQO, KvConfigQO};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod request;
pub mod api_key;
pub mod config;
pub mod provider;

/// 请求日志管理
pub mod request {
    use super::*;

    /// 请求日志列表
    #[catch_panic]
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
    #[catch_panic]
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
    #[catch_panic]
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
}

/// API Key 管理
pub mod api_key {
    use super::*;

    /// API Key 列表
    #[catch_panic]
    pub async fn list() -> Result<Json<serde_json::Value>> {
        let web_context = use_web().ok_or_else(|| anyhow!("Web context not found"))?;
        let db_context = use_db().ok_or_else(|| anyhow!("Database context not found"))?;
        
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
    #[catch_panic]
    pub async fn create() -> Result<Json<serde_json::Value>> {
        let web_context = use_web().ok_or_else(|| anyhow!("Web context not found"))?;
        let db_context = use_db().ok_or_else(|| anyhow!("Database context not found"))?;
        
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
    #[catch_panic]
    pub async fn update() -> Result<Json<serde_json::Value>> {
        let web_context = use_web().ok_or_else(|| anyhow!("Web context not found"))?;
        let db_context = use_db().ok_or_else(|| anyhow!("Database context not found"))?;
        
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
    #[catch_panic]
    pub async fn delete() -> Result<Json<serde_json::Value>> {
        let web_context = use_web().ok_or_else(|| anyhow!("Web context not found"))?;
        let db_context = use_db().ok_or_else(|| anyhow!("Database context not found"))?;
        
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
}

/// 配置管理
pub mod config {
    use super::*;

    /// 配置列表
    #[catch_panic]
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
    #[catch_panic]
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
}

/// 供应商管理
pub mod provider {
    use super::*;

    /// 供应商列表
    #[catch_panic]
    pub async fn list() -> Result<Json<serde_json::Value>> {
        let web_context = use_web().ok_or_else(|| anyhow!("Web context not found"))?;
        let db_context = use_db().ok_or_else(|| anyhow!("Database context not found"))?;
        
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
    #[catch_panic]
    pub async fn create() -> Result<Json<serde_json::Value>> {
        let web_context = use_web().ok_or_else(|| anyhow!("Web context not found"))?;
        let db_context = use_db().ok_or_else(|| anyhow!("Database context not found"))?;
        
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
    #[catch_panic]
    pub async fn update() -> Result<Json<serde_json::Value>> {
        let web_context = use_web().ok_or_else(|| anyhow!("Web context not found"))?;
        let db_context = use_db().ok_or_else(|| anyhow!("Database context not found"))?;
        
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
    #[catch_panic]
    pub async fn delete() -> Result<Json<serde_json::Value>> {
        let web_context = use_web().ok_or_else(|| anyhow!("Web context not found"))?;
        let db_context = use_db().ok_or_else(|| anyhow!("Database context not found"))?;
        
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
    #[catch_panic]
    pub async fn model_update() -> Result<Json<serde_json::Value>> {
        // TODO: 实现供应商模型更新逻辑
        Ok(Json(serde_json::json!({
            "code": 200,
            "message": "Success",
            "data": null
        })))
    }
}