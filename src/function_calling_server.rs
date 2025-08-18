//! Function Calling HTTP 服务器
//! 
//! 提供标准的HTTP API给大模型调用，支持OpenAI Function Calling格式

use std::sync::Arc;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing::{info, error};

use crate::function_tools::{EnhancedMaaFunctionServer, FunctionDefinition, FunctionCall, FunctionResponse};
use crate::ai_client::client::AiClientTrait;

/// Function Calling服务器接口trait
#[async_trait]
pub trait FunctionCallingServerTrait: Send + Sync {
    /// 获取函数定义
    fn get_function_definitions(&self) -> Vec<FunctionDefinition>;
    
    /// 执行函数调用
    async fn execute_function(&self, call: FunctionCall) -> FunctionResponse;
}

/// 为增强EnhancedMaaFunctionServer实现trait
#[async_trait]
impl FunctionCallingServerTrait for EnhancedMaaFunctionServer {
    fn get_function_definitions(&self) -> Vec<FunctionDefinition> {
        self.get_function_definitions()
    }
    
    async fn execute_function(&self, call: FunctionCall) -> FunctionResponse {
        self.execute_function(call).await
    }
}

/// 服务器状态
#[derive(Clone)]
pub struct AppState {
    pub function_server: Arc<dyn FunctionCallingServerTrait>,
}

/// 工具列表响应
#[derive(Serialize)]
pub struct ToolsResponse {
    pub tools: Vec<FunctionDefinition>,
    pub count: usize,
    pub server_info: ServerInfo,
}

/// 服务器信息
#[derive(Serialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub supported_formats: Vec<String>,
}

/// 函数调用请求（兼容OpenAI格式）
#[derive(Deserialize)]
pub struct CallRequest {
    pub function_call: FunctionCall,
}

/// 错误响应
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 启动Function Calling HTTP服务器
pub async fn start_function_calling_server<T: FunctionCallingServerTrait + 'static>(
    function_server: Arc<T>,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState { 
        function_server: function_server as Arc<dyn FunctionCallingServerTrait> 
    };

    // 检查静态文件目录是否存在
    let serve_static = if std::path::Path::new("./static").exists() {
        info!("发现静态文件目录，启用前端文件服务");
        Some(ServeDir::new("./static"))
    } else {
        info!("静态文件目录不存在，仅提供API服务");
        None
    };

    let mut app = Router::new()
        .route("/api/tools", get(list_tools_handler))
        .route("/api/call", post(call_function_handler))
        .route("/api/chat", post(chat_handler))
        .route("/api/health", get(health_handler))
        // 保持兼容性的API端点
        .route("/tools", get(list_tools_handler))
        .route("/call", post(call_function_handler))
        .route("/chat", post(chat_handler))
        .route("/health", get(health_handler))
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
        )
        .with_state(state);

    // 如果存在静态文件目录，添加前端文件服务
    if let Some(serve_dir) = serve_static {
        app = app
            .route("/", get(root_handler))
            .fallback_service(serve_dir);
    } else {
        app = app.route("/", get(root_handler));
    }

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    
    info!("MAA Function Calling服务器启动成功！");
    info!("地址: http://{}", addr);
    
    if std::path::Path::new("./static").exists() {
        info!("前端界面: http://{}", addr);
    }
    
    info!("API端点:");
    info!("   GET  /health       - 健康检查");
    info!("   GET  /tools        - 获取工具列表");
    info!("   POST /call         - 执行函数调用");
    info!("   POST /chat         - AI聊天代理");
    info!("   GET  /api/health   - 健康检查 (API路径)");
    info!("   GET  /api/tools    - 获取工具列表 (API路径)");
    info!("   POST /api/call     - 执行函数调用 (API路径)");
    info!("   POST /api/chat     - AI聊天代理 (API路径)");

    axum::serve(listener, app).await?;
    Ok(())
}

/// 根路径处理器
async fn root_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "name": "MAA智能控制中间层",
        "version": "0.1.0",
        "description": "为大模型提供MAA控制能力的Function Calling API",
        "endpoints": {
            "tools": "/tools",
            "call": "/call",
            "health": "/health"
        },
        "usage": {
            "1": "GET /tools - 获取所有可用工具的定义",
            "2": "POST /call - 执行函数调用",
            "example": {
                "url": "/call",
                "method": "POST",
                "body": {
                    "function_call": {
                        "name": "maa_command",
                        "arguments": {
                            "command": "帮我做日常"
                        }
                    }
                }
            }
        }
    }))
}

/// 获取工具列表
async fn list_tools_handler(State(state): State<AppState>) -> Json<ToolsResponse> {
    let tools = state.function_server.get_function_definitions();
    let count = tools.len();

    Json(ToolsResponse {
        tools,
        count,
        server_info: ServerInfo {
            name: "MAA智能控制中间层".to_string(),
            version: "0.1.0".to_string(),
            description: "为大模型提供MAA控制能力".to_string(),
            supported_formats: vec![
                "OpenAI Function Calling".to_string(),
                "Claude Tools".to_string(),
                "通用JSON-RPC".to_string(),
            ],
        },
    })
}

/// 执行函数调用
async fn call_function_handler(
    State(state): State<AppState>,
    Json(request): Json<CallRequest>,
) -> Result<Json<FunctionResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("收到函数调用请求: {}", request.function_call.name);

    let response = state.function_server.execute_function(request.function_call).await;

    if response.success {
        info!("函数调用成功");
        Ok(Json(response))
    } else {
        error!("函数调用失败: {:?}", response.error);
        Ok(Json(response)) // 仍然返回200，但在响应中标明失败
    }
}

/// 健康检查
async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": "maa-function-calling",
        "version": "0.1.0"
    }))
}

/// 聊天请求
#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub tools: Vec<FunctionDefinition>,
    pub system_prompt: Option<String>,
}

/// 聊天消息
#[derive(Debug, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// 聊天处理器
async fn chat_handler(
    State(_state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    use crate::ai_client::{AiClient, ChatMessage as AiChatMessage, Tool as AiTool};

    info!("收到聊天请求，消息数: {}", request.messages.len());

    // 调试：打印环境变量
    use std::env;
    info!("环境变量调试:");
    info!("  AI_PROVIDER: {:?}", env::var("AI_PROVIDER"));
    info!("  AI_API_KEY: {:?}", env::var("AI_API_KEY").map(|_| "[HIDDEN]"));
    info!("  AI_BASE_URL: {:?}", env::var("AI_BASE_URL"));
    info!("  AI_MODEL: {:?}", env::var("AI_MODEL"));

    // 创建AI客户端
    let ai_client = match AiClient::from_env() {
        Ok(client) => client,
        Err(e) => {
            error!("创建AI客户端失败: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("AI客户端初始化失败: {}", e),
                    code: "AI_CLIENT_ERROR".to_string(),
                    timestamp: chrono::Utc::now(),
                }),
            ));
        }
    };

    // 转换消息格式
    let mut ai_messages: Vec<AiChatMessage> = Vec::new();
    
    // 添加系统消息
    if let Some(system_prompt) = request.system_prompt {
        ai_messages.push(AiChatMessage {
            role: "system".to_string(),
            content: system_prompt,
        });
    }
    
    // 添加用户消息
    for msg in request.messages {
        ai_messages.push(AiChatMessage {
            role: msg.role,
            content: msg.content,
        });
    }

    // 转换工具格式
    let ai_tools: Vec<AiTool> = request.tools.into_iter().map(|tool| AiTool {
        name: tool.name,
        description: tool.description,
        parameters: tool.parameters,
    }).collect();


    // 调用AI聊天
    match ai_client.chat_completion_with_tools(ai_messages, ai_tools).await {
        Ok(result) => {
            use crate::ai_client::client::Either;
            
            match result {
                Either::Left(text_response) => {
                    // 纯文本响应
                    Ok(Json(serde_json::json!({
                        "choices": [{
                            "message": {
                                "role": "assistant",
                                "content": text_response,
                                "tool_calls": null
                            }
                        }]
                    })))
                },
                Either::Right(function_calls) => {
                    // 函数调用响应
                    let tool_calls: Vec<serde_json::Value> = function_calls.into_iter().map(|fc| {
                        serde_json::json!({
                            "function": {
                                "name": fc.name,
                                "arguments": serde_json::to_string(&fc.arguments).unwrap_or_default()
                            }
                        })
                    }).collect();

                    Ok(Json(serde_json::json!({
                        "choices": [{
                            "message": {
                                "role": "assistant",
                                "content": null,
                                "tool_calls": tool_calls
                            }
                        }]
                    })))
                }
            }
        },
        Err(e) => {
            error!("AI聊天调用失败: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("AI聊天失败: {}", e),
                    code: "AI_CHAT_ERROR".to_string(),
                    timestamp: chrono::Utc::now(),
                }),
            ))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::maa_adapter::{MaaAdapter, MaaAdapterTrait, MaaConfig};
    use crate::mcp_tools::create_function_server;

    #[tokio::test]
    async fn test_server_endpoints() {
        let config = MaaConfig::default();
        let maa_adapter = Arc::new(MaaAdapter::new(config).await.unwrap());
        let function_server = Arc::new(create_function_server(maa_adapter));
        
        let state = AppState { function_server };

        // 测试工具列表
        let response = list_tools_handler(State(state.clone())).await;
        assert!(response.count > 0);
        assert_eq!(response.tools[0].name, "maa_status");

        // 测试健康检查
        let health_response = health_handler().await;
        assert_eq!(health_response.0["status"], "healthy");
    }
}
