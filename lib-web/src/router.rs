use anyhow::{Result, anyhow};
use framework_web::{
    WebRoute, WebRouteFuture, WebResponse, AuthRule,
    web_api_post, scope_db, scope_web, catch_panic, 
    use_web, WebError, Json, Query
};
use lib_db::{DbContext, use_db};
use lib_web_core::{AppContext, use_app, Authorization, AuthType};
use lib_provider::ProviderServiceClient;
use types_admin::entity::KvConfig;
use std::sync::Arc;

/// Web 路由器
pub struct WebRouter {
    app_context: Arc<AppContext>,
    auth_service: Arc<lib_web_core::auth::AuthorizationService>,
    provider_factory: Arc<dyn crate::provider::ProviderFactory>,
}

impl WebRouter {
    pub fn new(
        app_context: Arc<AppContext>,
        auth_service: Arc<lib_web_core::auth::AuthorizationService>,
        provider_factory: Arc<dyn crate::provider::ProviderFactory>,
    ) -> Self {
        Self {
            app_context,
            auth_service,
            provider_factory,
        }
    }

    /// 构建路由器
    pub fn build() -> Vec<WebRoute> {
        let mut routes = Vec::new();

        // AI 接口路由
        routes.extend(Self::build_ai_routes());

        // 管理接口路由
        routes.extend(Self::build_admin_routes());

        routes
    }

    /// 构建 AI 接口路由
    fn build_ai_routes() -> Vec<WebRoute> {
        vec![
            // 聊天接口
            web_api_post!("/chat", AuthRule::anonymous(), chat_handler),
            // 模型列表接口
            web_api_post!("/models", AuthRule::anonymous(), models_handler),
        ]
    }

    /// 构建管理接口路由
    fn build_admin_routes() -> Vec<WebRoute> {
        vec![
            // 请求日志管理
            web_api_post!("/___request/list", AuthRule::admin(), request_list_handler),
            web_api_post!("/___request/detail", AuthRule::admin(), request_detail_handler),
            web_api_post!("/___request/sub-list", AuthRule::admin(), request_sub_list_handler),
            
            // API Key 管理
            web_api_post!("/___api-key/list", AuthRule::admin(), api_key_list_handler),
            web_api_post!("/___api-key/create", AuthRule::admin(), api_key_create_handler),
            web_api_post!("/___api-key/update", AuthRule::admin(), api_key_update_handler),
            web_api_post!("/___api-key/delete", AuthRule::admin(), api_key_delete_handler),
            
            // 配置管理
            web_api_post!("/___config/list", AuthRule::admin(), config_list_handler),
            web_api_post!("/___config/update", AuthRule::admin(), config_update_handler),
            
            // 供应商管理
            web_api_post!("/___provider/list", AuthRule::admin(), provider_list_handler),
            web_api_post!("/___provider/create", AuthRule::admin(), provider_create_handler),
            web_api_post!("/___provider/update", AuthRule::admin(), provider_update_handler),
            web_api_post!("/___provider/delete", AuthRule::admin(), provider_delete_handler),
            web_api_post!("/___provider/model-update", AuthRule::admin(), provider_model_update_handler),
        ]
    }

    /// 处理请求
    pub async fn handle_request(&self, route: &WebRoute) -> WebResponse {
        let web_context = use_web().ok_or_else(|| WebError::unauthorized("Web context not found"))?;
        let db_context = use_db().ok_or_else(|| WebError::internal("Database context not found"))?;

        // 设置应用上下文
        lib_web_core::set_app_context(self.app_context.clone());

        // 设置请求上下文
        let request_context = lib_web_core::RequestContext::new(web_context.request_id.clone())
            .with_client_ip(web_context.client_ip.clone())
            .with_user_agent(web_context.headers.get("user-agent").cloned())
            .with_auth(self.parse_authorization(&web_context)?);

        // 执行路由处理
        match route.invoke {
            Some(ref invoke) => {
                let result = invoke().await;
                WebResponse::from_result(result)
            }
            None => WebResponse::internal("No handler found"),
        }
    }

    /// 解析鉴权信息
    fn parse_authorization(&self, web_context: &framework_web::WebContext) -> Result<Option<Authorization>> {
        let auth_header = web_context.headers.get("authorization")
            .map(|h| h.clone())
            .unwrap_or_else(|| "".to_string());

        let auth_query = lib_web_core::auth::AuthorizationQuery::new(self.auth_service.clone());

        if self.app_context.get_allow_anonymous().await {
            // 允许匿名访问
            Ok(Some(Authorization::anonymous()))
        } else {
            // 需要鉴权
            match auth_query.parse_and_validate(&auth_header, AuthType::API).await {
                Ok(auth) => Ok(Some(auth)),
                Err(_) => Ok(None),
            }
        }
    }
}

/// AI 聊天接口处理器

async fn chat_handler() -> Result<Json<serde_json::Value>> {
    let web_context = use_web().ok_or_else(|| anyhow!("Web context not found"))?;
    let db_context = use_db().ok_or_else(|| anyhow!("Database context not found"))?;
    
    // 解析请求参数
    let request_body = web_context.body_json::<serde_json::Value>()
        .await
        .map_err(|e| anyhow!("Failed to parse request body: {}", e))?;
    
    // 获取模型名称
    let model = request_body.get("model")
        .and_then(|m| m.as_str())
        .ok_or_else(|| anyhow!("Model is required"))?;
    
    // 获取流式标志
    let stream = request_body.get("stream")
        .and_then(|s| s.as_bool())
        .unwrap_or(false);
    
    // TODO: 实现主请求流程
    // 1. 记录主请求日志
    // 2. 匹配供应商
    // 3. 创建客户端
    // 4. 发送请求
    // 5. 记录响应
    
    Ok(Json(serde_json::json!({
        "id": "chatcmpl-" + &lib_core::next_id()?.to_string(),
        "object": "chat.completion",
        "created": lib_core::current_millis()?,
        "model": model,
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello! I'm a test response."
                },
                "finish_reason": "stop"
            }
        ],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 10,
            "total_tokens": 20
        }
    })))
}

/// AI 模型列表接口处理器

async fn models_handler() -> Result<Json<serde_json::Value>> {
    let web_context = use_web().ok_or_else(|| anyhow!("Web context not found"))?;
    
    // TODO: 实现模型列表查询
    // 1. 查询启用的模型
    // 2. 去重和过滤
    // 3. 返回模型列表
    
    Ok(Json(serde_json::json!({
        "object": "list",
        "data": [
            {
                "id": "gpt-3.5-turbo",
                "object": "model",
                "created": 1677610602,
                "owned_by": "openai"
            },
            {
                "id": "gpt-4",
                "object": "model",
                "created": 1687882411,
                "owned_by": "openai"
            }
        ]
    })))
}

/// 请求日志列表处理器

async fn request_list_handler() -> Result<Json<serde_json::Value>> {
    // TODO: 实现请求日志列表查询
    Ok(Json(serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": {
            "total": 0,
            "records": []
        }
    })))
}

/// 请求日志详情处理器

async fn request_detail_handler() -> Result<Json<serde_json::Value>> {
    // TODO: 实现请求日志详情查询
    Ok(Json(serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": null
    })))
}

/// 子请求日志列表处理器

async fn request_sub_list_handler() -> Result<Json<serde_json::Value>> {
    // TODO: 实现子请求日志列表查询
    Ok(Json(serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": {
            "total": 0,
            "records": []
        }
    })))
}

/// API Key 列表处理器

async fn api_key_list_handler() -> Result<Json<serde_json::Value>> {
    // TODO: 实现 API Key 列表查询
    Ok(Json(serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": {
            "total": 0,
            "records": []
        }
    })))
}

/// API Key 创建处理器

async fn api_key_create_handler() -> Result<Json<serde_json::Value>> {
    // TODO: 实现 API Key 创建
    Ok(Json(serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": null
    })))
}

/// API Key 更新处理器

async fn api_key_update_handler() -> Result<Json<serde_json::Value>> {
    // TODO: 实现 API Key 更新
    Ok(Json(serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": null
    })))
}

/// API Key 删除处理器

async fn api_key_delete_handler() -> Result<Json<serde_json::Value>> {
    // TODO: 实现 API Key 删除
    Ok(Json(serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": null
    })))
}

/// 配置列表处理器

async fn config_list_handler() -> Result<Json<serde_json::Value>> {
    // TODO: 实现配置列表查询
    Ok(Json(serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": {
            "total": 0,
            "records": []
        }
    })))
}

/// 配置更新处理器

async fn config_update_handler() -> Result<Json<serde_json::Value>> {
    // TODO: 实现配置更新
    Ok(Json(serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": null
    })))
}

/// 供应商列表处理器

async fn provider_list_handler() -> Result<Json<serde_json::Value>> {
    // TODO: 实现供应商列表查询
    Ok(Json(serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": {
            "total": 0,
            "records": []
        }
    })))
}

/// 供应商创建处理器

async fn provider_create_handler() -> Result<Json<serde_json::Value>> {
    // TODO: 实现供应商创建
    Ok(Json(serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": null
    })))
}

/// 供应商更新处理器

async fn provider_update_handler() -> Result<Json<serde_json::Value>> {
    // TODO: 实现供应商更新
    Ok(Json(serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": null
    })))
}

/// 供应商删除处理器

async fn provider_delete_handler() -> Result<Json<serde_json::Value>> {
    // TODO: 实现供应商删除
    Ok(Json(serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": null
    })))
}

/// 供应商模型更新处理器

async fn provider_model_update_handler() -> Result<Json<serde_json::Value>> {
    // TODO: 实现供应商模型更新
    Ok(Json(serde_json::json!({
        "code": 200,
        "message": "Success",
        "data": null
    })))
}