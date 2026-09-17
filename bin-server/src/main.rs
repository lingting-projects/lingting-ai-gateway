use anyhow::{Result, anyhow};
use framework_core::{ApplicationDirectory, current_millis};
use lib_db::{DbContext, DbConfig, init_db};
use lib_web::{WebRouter, WebContext};
use lib_web_core::AppContext;
use tracing::{info, error, warn};
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use std::sync::Arc;
use axum::{routing::Router, response::Json, http::StatusCode};
use serde_json::json;

/// 应用程序
pub struct GatewayApplication {
    app_context: Arc<AppContext>,
    db_context: Option<DbContext>,
    router: WebRouter,
}

impl GatewayApplication {
    /// 创建新的应用程序实例
    pub async fn new() -> Result<Self> {
        // 创建应用目录
        let app_dir = ApplicationDirectory::normal("lingting-ai-gateway", "")?;
        info!("应用目录: {:?}", app_dir);
        
        // 初始化数据库
        let db_config = Self::load_db_config().await?;
        let db_context = init_db(&app_dir.data, &db_config).await?;
        
        // 创建应用上下文
        let app_context = Arc::new(AppContext::new());
        
        // 加载配置
        Self::load_config(&db_context, &app_context).await?;
        
        // 创建路由器
        let router = WebRouter::new(
            app_context.clone(),
            Arc::new(MockProviderFactory::new()),
        );
        
        Ok(Self {
            app_context,
            db_context: Some(db_context),
            router,
        })
    }
    
    /// 启动应用程序
    pub async fn run(&self) -> Result<()> {
        info!("启动 lingting-ai-gateway...");
        
        // 构建 Axum 路由器
        let axum_router = Router::new()
            .route("/health", axum::routing::get(health_check))
            .route("/chat", axum::routing::post(chat_handler))
            .route("/models", axum::routing::post(models_handler))
            .fallback(fallback_handler);
        
        // 启动 Axum 服务器
        let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await
            .map_err(|e| anyhow!("无法绑定端口: {}", e))?;
        
        info!("服务器启动成功，监听地址: 0.0.0.0:8080");
        info!("健康检查: http://localhost:8080/health");
        info!("AI 聊天接口: http://localhost:8080/chat");
        info!("AI 模型列表: http://localhost:8080/models");
        
        axum::serve(listener, axum_router)
            .await
            .map_err(|e| anyhow!("服务器运行错误: {}", e))?;
        
        Ok(())
    }
    
    /// 加载数据库配置
    async fn load_db_config() -> Result<DbConfig> {
        // 从环境变量读取配置，如果不存在则使用默认值
        let db_address = std::env::var("DB_ADDRESS")
            .unwrap_or_else(|_| "localhost".to_string());
        let db_database = std::env::var("DB_DATABASE")
            .unwrap_or_else(|_| "lingting_ai_gateway".to_string());
        let db_username = std::env::var("DB_USERNAME")
            .unwrap_or_else(|_| "postgres".to_string());
        let db_password = std::env::var("DB_PASSWORD")
            .unwrap_or_else(|_| "password".to_string());
        
        Ok(DbConfig {
            address: db_address,
            database: db_database,
            username: db_username,
            password: db_password,
        })
    }
    
    /// 加载配置
    async fn load_config(
        db_context: &DbContext,
        app_context: &AppContext,
    ) -> Result<()> {
        // TODO: 由 WebManager 装载鉴权与全局配置，这里暂时使用默认值
        info!("配置加载完成");
        Ok(())
    }
}

/// 健康检查接口
async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "timestamp": current_millis().unwrap_or(0),
        "version": "1.0.0"
    }))
}

/// 聊天接口处理器
async fn chat_handler() -> Json<serde_json::Value> {
    // 这里应该调用 lib-web 的聊天接口
    // 现在返回一个简单的响应
    Json(json!({
        "id": "chatcmpl-" + &lib_core::next_id().unwrap_or(0).to_string(),
        "object": "chat.completion",
        "created": current_millis().unwrap_or(0),
        "model": "gpt-3.5-turbo",
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello! I'm a mock response."
                },
                "finish_reason": "stop"
            }
        ],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 10,
            "total_tokens": 20
        }
    }))
}

/// 模型列表接口处理器
async fn models_handler() -> Json<serde_json::Value> {
    Json(json!({
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
    }))
}

/// 404 处理器
async fn fallback_handler() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::NOT_FOUND, Json(json!({
        "code": 404,
        "message": "Not Found"
    })))
}

/// 模拟供应商工厂
struct MockProviderFactory {}

impl MockProviderFactory {
    fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl lib_provider::ProviderFactory for MockProviderFactory {
    type Client = Box<dyn lib_provider::provider::ProviderClient>;

    fn create_client(&self, model_name: &str) -> Result<Self::Client> {
        Ok(Box::new(MockProviderClient::new(model_name)))
    }

    fn create_streaming_client(&self, model_name: &str) -> Result<Self::Client> {
        Ok(Box::new(MockProviderClient::new(model_name)))
    }
}

/// 模拟供应商客户端
struct MockProviderClient {
    model: String,
}

impl MockProviderClient {
    fn new(model: &str) -> Self {
        Self {
            model: model.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl lib_provider::provider::ProviderClient for MockProviderClient {
    async fn chat(&mut self, request: &lib_provider::ChatRequest) -> Result<lib_provider::response::ChatResponse> {
        // 模拟延迟
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        Ok(lib_provider::response::ChatResponse {
            id: "chatcmpl-" + &lib_core::next_id().unwrap_or(0).to_string(),
            object: "chat.completion".to_string(),
            created: current_millis().unwrap_or(0),
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
            usage: lib_provider::response::TokenInfo {
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

    async fn chat_stream(&mut self, request: &lib_provider::ChatRequest) -> Result<lib_provider::response::ChatStreamResponse> {
        // 模拟流式响应
        Ok(lib_provider::response::ChatStreamResponse::new(
            "chatcmpl-" + &lib_core::next_id().unwrap_or(0).to_string(),
            request.model.clone(),
            current_millis().unwrap_or(0),
        ))
    }

    fn name(&self) -> &str {
        "mock-provider"
    }

    fn model(&self) -> &str {
        &self.model
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    init_logging().await?;
    
    // 创建应用程序
    let app = GatewayApplication::new().await?;
    
    // 运行应用程序
    app.run().await
}

/// 初始化日志
async fn init_logging() -> Result<()> {
    // 创建日志目录
    let app_dir = ApplicationDirectory::normal("lingting-ai-gateway", "")?;
    let logs_dir = app_dir.logs();
    std::fs::create_dir_all(&logs_dir)?;
    
    // 创建非写入器（用于控制台输出）
    let (stdout_writer, stdout_guard) = non_blocking(std::io::stdout());
    
    // 创建滚动文件写入器
    let file_appender = rolling::daily(&logs_dir, "app");
    let (file_writer, file_guard) = non_blocking(file_appender);
    
    // 设置日志过滤器
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    // 初始化日志订阅者
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_writer(stdout_writer)
                .with_ansi(true)
        )
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_writer(file_writer)
                .with_ansi(false)
        )
        .with(filter)
        .init();
    
    // 保持写入器活跃
    tokio::spawn(async move {
        let _ = stdout_guard;
        let _ = file_guard;
    });
    
    info!("日志系统初始化完成");
    Ok(())
}