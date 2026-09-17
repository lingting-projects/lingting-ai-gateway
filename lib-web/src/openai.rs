use anyhow::{Result, anyhow};
use framework_web::{Json, WebError, catch_panic, use_web};
use lib_db::{DbContext, use_db, RequestMainRepository, RequestSubRepository};
use lib_web_core::{AppContext, use_app, Authorization, AuthType};
use lib_provider::{ChatRequest, ChatRequestBuilder, ProviderServiceClient, ProviderClientCallbacks};
use lib_provider::response::{ChatResponse, ErrorResponse};
use types_admin::entity::{RequestMain, RequestSub, RequestMainStatus, RequestSubStatus, TokenInfo};
use types_admin::dto::{RequestMainCreatePO, RequestSubCreatePO};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 聊天接口

pub async fn chat() -> Result<Json<serde_json::Value>> {
    let web_context = use_web().ok_or_else(|| anyhow!("Web context not found"))?;
    let db_context = use_db().ok_or_else(|| anyhow!("Database context not found"))?;
    let app_context = use_app().ok_or_else(|| anyhow!("App context not found"))?;
    
    // 解析请求参数
    let request_body = web_context.body_json::<ChatRequest>()
        .await
        .map_err(|e| anyhow!("Failed to parse request body: {}", e))?;
    
    // 获取模型名称
    let model = &request_body.model;
    let stream = request_body.stream.unwrap_or(false);
    
    // 记录主请求日志
    let main_request_id = record_main_request(
        &web_context,
        &app_context,
        model,
        stream,
        &request_body,
    ).await?;
    
    // 匹配供应商
    let provider_client = match match_provider(model, &app_context).await {
        Ok(client) => client,
        Err(error) => {
            // 更新主请求日志为失败状态
            update_main_request_status(
                &db_context,
                main_request_id,
                RequestMainStatus::Failed,
                Some(error.to_string()),
                None,
                None,
                None,
                None,
            ).await?;
            
            return Err(error);
        }
    };
    
    // 记录子请求日志
    let sub_request_id = record_sub_request(
        &db_context,
        main_request_id,
        &provider_client,
        model,
        &request_body,
    ).await?;
    
    // 发送请求
    let response = if stream {
        handle_streaming_request(
            &db_context,
            main_request_id,
            sub_request_id,
            provider_client,
            request_body,
        ).await
    } else {
        handle_normal_request(
            &db_context,
            main_request_id,
            sub_request_id,
            provider_client,
            request_body,
        ).await
    };
    
    // 更新主请求日志
    update_main_request_from_response(
        &db_context,
        main_request_id,
        &response,
    ).await?;
    
    Ok(Json(response))
}

/// 模型列表接口

pub async fn models() -> Result<Json<serde_json::Value>> {
    let web_context = use_web().ok_or_else(|| anyhow!("Web context not found"))?;
    let app_context = use_app().ok_or_else(|| anyhow!("App context not found"))?;
    
    // TODO: 从数据库查询启用的模型
    // 这里返回硬编码的模型列表
    let models = vec![
        "gpt-3.5-turbo",
        "gpt-3.5-turbo-16k",
        "gpt-4",
        "gpt-4-turbo",
        "gpt-4o",
    ];
    
    Ok(Json(serde_json::json!({
        "object": "list",
        "data": models.into_iter().map(|id| {
            serde_json::json!({
                "id": id,
                "object": "model",
                "created": 1677610602,
                "owned_by": "openai"
            })
        }).collect::<Vec<_>>()
    })))
}

/// 记录主请求日志
async fn record_main_request(
    web_context: &framework_web::WebContext,
    app_context: &AppContext,
    model: &str,
    stream: bool,
    request: &ChatRequest,
) -> Result<i64> {
    let client_ip = web_context.client_ip.clone();
    let user_agent = web_context.headers.get("user-agent").cloned();
    let auth = parse_authorization(web_context, app_context).await?;
    
    let request_params = if app_context.get_debug_mode().await {
        serde_json::to_value(request).unwrap_or(serde_json::Value::Null)
    } else {
        serde_json::json!({
            "messages": request.messages.len(),
            "temperature": request.temperature,
            "max_tokens": request.max_tokens,
            "top_p": request.top_p,
            "stream": request.stream,
            "tools": request.tools.as_ref().map(|t| t.len()),
            "tool_choice": request.tool_choice.as_ref().map(|_| "specified"),
        })
    };
    
    let main_request = RequestMainCreatePO {
        client_ip,
        user_agent,
        credential_id: auth.map(|a| a.value).unwrap_or_else(|| "".to_string()),
        model: model.to_string(),
        stream,
        method: "POST".to_string(),
        path: "/chat".to_string(),
        request_params,
        status: RequestMainStatus::Processing,
        current_status: "正在处理请求".to_string(),
        provider_count: 0, // TODO: 从匹配结果中获取
        return_model: None,
        input_tokens: None,
        output_tokens: None,
        cache_read_tokens: None,
        cache_write_tokens: None,
        inference_tokens: None,
        read_tokens: None,
        write_tokens: None,
        total_tokens: None,
        http_status: None,
        finish_reason: None,
        provider_request_id: None,
        response_content: None,
        start_time: lib_core::current_millis()?,
        end_time: None,
        duration_ms: None,
    };
    
    let repository = RequestMainRepository::new(use_db().ok_or_else(|| anyhow!("Database context not found"))?);
    repository.create(&main_request.into()).await
        .map_err(|e| anyhow!("Failed to create main request: {}", e))?;
    
    Ok(0) // TODO: 返回实际的 ID
}

/// 匹配供应商
async fn match_provider(
    model: &str,
    app_context: &AppContext,
) -> Result<ProviderServiceClient> {
    // TODO: 实现供应商匹配逻辑
    // 1. 查询支持该模型的供应商
    // 2. 按优先级排序
    // 3. 创建客户端
    
    // 这里返回一个模拟的客户端
    let callbacks = ProviderClientCallbacks::new()
        .on_success(|response| {
            tracing::info!("Provider request successful: model={}, tokens={}", 
                response.model, response.usage.total_tokens);
        })
        .on_error(|error| {
            tracing::error!("Provider request error: {}", error.message);
        });
    
    Ok(ProviderServiceClient::new(
        "openai".to_string(),
        model.to_string(),
        Box::new(MockProviderClient::new()),
        Some(callbacks),
    ))
}

/// 记录子请求日志
async fn record_sub_request(
    db_context: &DbContext,
    main_request_id: i64,
    provider_client: &ProviderServiceClient,
    model: &str,
    request: &ChatRequest,
) -> Result<i64> {
    let sub_request = RequestSubCreatePO {
        main_request_id,
        provider_id: 1, // TODO: 从供应商信息中获取
        provider_name: provider_client.provider_name.clone(),
        model: model.to_string(),
        provider_url: "".to_string(), // TODO: 从供应商信息中获取
        request_params: serde_json::to_value(request).unwrap_or(serde_json::Value::Null),
        status: RequestSubStatus::Processing,
        return_model: None,
        input_tokens: None,
        output_tokens: None,
        cache_read_tokens: None,
        cache_write_tokens: None,
        inference_tokens: None,
        read_tokens: None,
        write_tokens: None,
        total_tokens: None,
        http_status: None,
        finish_reason: None,
        provider_request_id: None,
        response_content: None,
        start_time: lib_core::current_millis()?,
        end_time: None,
        duration_ms: None,
    };
    
    let repository = RequestSubRepository::new(db_context.pool.clone());
    repository.create(&sub_request.into()).await
        .map_err(|e| anyhow!("Failed to create sub request: {}", e))?;
    
    Ok(0) // TODO: 返回实际的 ID
}

/// 处理普通请求
async fn handle_normal_request(
    db_context: &DbContext,
    main_request_id: i64,
    sub_request_id: i64,
    mut provider_client: ProviderServiceClient,
    request: ChatRequest,
) -> Result<serde_json::Value> {
    let response = provider_client.chat(&request).await?;
    
    // 更新子请求日志
    update_sub_request(
        db_context,
        sub_request_id,
        &response,
    ).await?;
    
    Ok(serde_json::json!({
        "id": response.id,
        "object": response.object,
        "created": response.created,
        "model": response.model,
        "choices": response.choices,
        "usage": response.usage,
        "system_fingerprint": response.system_fingerprint,
    }))
}

/// 处理流式请求
async fn handle_streaming_request(
    db_context: &DbContext,
    main_request_id: i64,
    sub_request_id: i64,
    mut provider_client: ProviderServiceClient,
    request: ChatRequest,
) -> Result<serde_json::Value> {
    let mut accumulated_content = String::new();
    let mut accumulated_tokens = TokenInfo::zero();
    
    let mut stream = provider_client.chat_stream(&request).await?;
    
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(chunk) => {
                // 累积内容
                if let Some(content) = &chunk.content {
                    accumulated_content.push_str(content);
                }
                
                // 累积 Token 信息
                if let Some(ref token_info) = chunk.token_info {
                    accumulated_tokens.input_tokens += token_info.input_tokens;
                    accumulated_tokens.output_tokens += token_info.output_tokens;
                    accumulated_tokens.cache_read_tokens += token_info.cache_read_tokens;
                    accumulated_tokens.cache_write_tokens += token_info.cache_write_tokens;
                    accumulated_tokens.inference_tokens += token_info.inference_tokens;
                    accumulated_tokens.read_tokens += token_info.read_tokens;
                    accumulated_tokens.write_tokens += token_info.write_tokens;
                    accumulated_tokens.total_tokens += token_info.total_tokens;
                }
                
                // TODO: 发送 chunk 给客户端
                // 这里需要实现流式响应
            }
            Err(error) => {
                return Err(error);
            }
        }
    }
    
    // 更新子请求日志
    update_sub_request(
        db_context,
        sub_request_id,
        &ChatResponse {
            id: "".to_string(), // TODO: 从流中获取
            object: "chat.completion".to_string(),
            created: 0, // TODO: 从流中获取
            model: request.model,
            choices: vec![], // 流式响应没有 choices
            usage: accumulated_tokens,
            system_fingerprint: None,
        },
    ).await?;
    
    Ok(serde_json::json!({
        "id": "chatcmpl-" + &lib_core::next_id()?.to_string(),
        "object": "chat.completion",
        "created": lib_core::current_millis()?,
        "model": request.model,
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": accumulated_content,
                },
                "finish_reason": "stop"
            }
        ],
        "usage": accumulated_tokens,
    }))
}

/// 更新主请求日志状态
async fn update_main_request_status(
    db_context: &DbContext,
    main_request_id: i64,
    status: RequestMainStatus,
    current_status: Option<String>,
    return_model: Option<String>,
    token_info: Option<TokenInfo>,
    http_status: Option<i32>,
    finish_reason: Option<String>,
) -> Result<()> {
    let repository = RequestMainRepository::new(db_context.pool.clone());
    
    repository.update_status(main_request_id, status, current_status).await?;
    
    if let Some(ref token_info) = token_info {
        repository.update_token_info(main_request_id, token_info).await?;
    }
    
    if status == RequestMainStatus::Success || status == RequestMainStatus::Failed {
        let end_time = lib_core::current_millis()?;
        let duration_ms = None; // TODO: 计算耗时
        
        repository.update_finish_info(
            main_request_id,
            return_model,
            http_status,
            finish_reason,
            None, // provider_request_id
            None, // response_content
            end_time,
            duration_ms,
        ).await?;
    }
    
    Ok(())
}

/// 更新子请求日志
async fn update_sub_request(
    db_context: &DbContext,
    sub_request_id: i64,
    response: &ChatResponse,
) -> Result<()> {
    let repository = RequestSubRepository::new(db_context.pool.clone());
    
    // 更新状态和 Token 信息
    repository.update_status(
        sub_request_id,
        RequestSubStatus::Success,
        Some("请求成功".to_string()),
    ).await?;
    
    repository.update_token_info(sub_request_id, &response.usage).await?;
    
    // 更新完成信息
    let end_time = lib_core::current_millis()?;
    let duration_ms = None; // TODO: 计算耗时
    
    repository.update_finish_info(
        sub_request_id,
        Some(response.model.clone()),
        Some(200), // HTTP 200
        Some("stop".to_string()),
        None, // provider_request_id
        None, // response_content
        end_time,
        duration_ms,
    ).await?;
    
    Ok(())
}

/// 从响应更新主请求日志
async fn update_main_request_from_response(
    db_context: &DbContext,
    main_request_id: i64,
    response: &serde_json::Value,
) -> Result<()> {
    // TODO: 从响应中提取信息并更新主请求日志
    Ok(())
}

/// 解析鉴权信息
async fn parse_authorization(
    web_context: &framework_web::WebContext,
    app_context: &AppContext,
) -> Result<Option<Authorization>> {
    let auth_header = web_context.headers.get("authorization")
        .map(|h| h.clone())
        .unwrap_or_else(|| "".to_string());
    
    if app_context.get_allow_anonymous().await {
        Ok(Some(Authorization::anonymous()))
    } else {
        // TODO: 实现鉴权逻辑
        Ok(None)
    }
}

/// 模拟供应商客户端
struct MockProviderClient {
    model: String,
}

impl MockProviderClient {
    fn new() -> Self {
        Self {
            model: "gpt-3.5-turbo".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl lib_provider::provider::ProviderClient for MockProviderClient {
    async fn chat(&mut self, request: &ChatRequest) -> Result<ChatResponse> {
        // 模拟延迟
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        Ok(ChatResponse {
            id: "chatcmpl-" + &lib_core::next_id()?.to_string(),
            object: "chat.completion".to_string(),
            created: lib_core::current_millis()?,
            model: request.model.clone(),
            choices: vec![
                lib_provider::response::Choice {
                    index: 0,
                    message: lib_provider::response::ChoiceMessage {
                        role: lib_provider::response::MessageRole::Assistant,
                        content: Some("Hello! I'm a mock response.".to_string()),
                        tool_calls: None,
                        refusal: None,
                    },
                    finish_reason: Some(lib_provider::response::FinishReason::Stop),
                    logprobs: None,
                }
            ],
            usage: TokenInfo {
                input_tokens: 10,
                output_tokens: 20,
                cache_read_tokens: 0,
                cache_write_tokens: 0,
                inference_tokens: 0,
                read_tokens: 0,
                write_tokens: 0,
                total_tokens: 30,
            },
            system_fingerprint: None,
        })
    }

    async fn chat_stream(&mut self, request: &ChatRequest) -> Result<lib_provider::response::ChatStreamResponse> {
        // 模拟流式响应
        Ok(lib_provider::response::ChatStreamResponse::new(
            "chatcmpl-" + &lib_core::next_id()?.to_string(),
            request.model.clone(),
            lib_core::current_millis()?,
        ))
    }

    fn name(&self) -> &str {
        "mock-provider"
    }

    fn model(&self) -> &str {
        &self.model
    }
}