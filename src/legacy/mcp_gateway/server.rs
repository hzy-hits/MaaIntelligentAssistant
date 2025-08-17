//! MCP Gateway Server Implementation
//!
//! 实现HTTP服务器接收AI的Function Calling，转换为标准MCP调用

use std::collections::HashMap;
use std::sync::Arc;
use std::net::SocketAddr;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::{debug, error, info};

use crate::mcp_tools::MaaToolsHandler;
use super::{
    FormatHandler, OpenAIHandler, ClaudeHandler, QwenHandler, KimiHandler,
    McpGatewayTrait, McpGatewayError, McpGatewayResult,
    UnifiedToolCall, UnifiedToolResponse
};

/// Gateway配置
#[derive(Debug, Clone)]
pub struct GatewayConfig {
    /// 服务器监听地址
    pub bind_address: SocketAddr,
    /// 是否启用CORS
    pub enable_cors: bool,
    /// 超时时间（秒）
    pub timeout_seconds: u64,
    /// 最大请求体大小（字节）
    pub max_request_size: usize,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1:8080".parse().unwrap(),
            enable_cors: true,
            timeout_seconds: 30,
            max_request_size: 1024 * 1024, // 1MB
        }
    }
}

/// Gateway错误
pub type GatewayError = McpGatewayError;

/// MCP Gateway主要实现
pub struct McpGateway {
    /// MAA工具处理器
    maa_tools: Arc<MaaToolsHandler>,
    /// AI格式处理器
    format_handlers: HashMap<String, Box<dyn FormatHandler + Send + Sync>>,
    /// 配置
    config: GatewayConfig,
}

impl McpGateway {
    /// 创建新的Gateway实例
    pub fn new(maa_tools: Arc<MaaToolsHandler>, config: GatewayConfig) -> Self {
        let mut format_handlers: HashMap<String, Box<dyn FormatHandler + Send + Sync>> = HashMap::new();
        
        // 注册各种AI格式处理器
        format_handlers.insert("openai".to_string(), Box::new(OpenAIHandler::new()));
        format_handlers.insert("claude".to_string(), Box::new(ClaudeHandler::new()));
        format_handlers.insert("qwen".to_string(), Box::new(QwenHandler::new()));
        format_handlers.insert("kimi".to_string(), Box::new(KimiHandler::new()));
        
        Self {
            maa_tools,
            format_handlers,
            config,
        }
    }

    /// 启动HTTP服务器
    pub async fn start(&self) -> McpGatewayResult<()> {
        info!("Starting MCP Gateway server on {}", self.config.bind_address);

        let app = self.create_router();
        
        let listener = TcpListener::bind(&self.config.bind_address).await
            .map_err(|e| McpGatewayError::ServerError { 
                message: format!("Failed to bind to {}: {}", self.config.bind_address, e) 
            })?;

        info!("MCP Gateway server listening on {}", self.config.bind_address);
        
        axum::serve(listener, app).await
            .map_err(|e| McpGatewayError::ServerError { 
                message: format!("Server error: {}", e) 
            })?;

        Ok(())
    }

    /// 创建Axum路由器
    fn create_router(&self) -> Router {
        let mut router = Router::new()
            .route("/health", get(health_check))
            .route("/tools", get(list_tools))
            .route("/call/:format", post(handle_format_call))
            .route("/call/unified", post(handle_unified_call))
            .with_state(Arc::new(self.clone()));

        if self.config.enable_cors {
            router = router.layer(
                ServiceBuilder::new()
                    .layer(CorsLayer::permissive())
            );
        }

        router
    }
}

// 实现Clone trait以便在Axum中使用
impl Clone for McpGateway {
    fn clone(&self) -> Self {
        Self {
            maa_tools: self.maa_tools.clone(),
            format_handlers: HashMap::new(), // 简化clone，实际使用中需要考虑
            config: self.config.clone(),
        }
    }
}

#[async_trait::async_trait]
impl McpGatewayTrait for McpGateway {
    async fn handle_function_call(&self, format: &str, request: serde_json::Value) -> McpGatewayResult<serde_json::Value> {
        debug!("Handling function call for format: {}", format);

        let handler = self.format_handlers.get(format)
            .ok_or_else(|| McpGatewayError::UnsupportedFormat { 
                format: format.to_string() 
            })?;

        // 转换AI请求为统一格式
        let unified_call = handler.convert_function_call(request)
            .map_err(|e| McpGatewayError::FormatConversion { 
                message: format!("Failed to convert {} request: {}", format, e) 
            })?;

        // 处理统一格式的调用
        let unified_response = self.handle_unified_call(unified_call).await?;

        // 转换响应为AI格式
        let ai_response = handler.convert_response(serde_json::to_value(unified_response).unwrap())
            .map_err(|e| McpGatewayError::FormatConversion { 
                message: format!("Failed to convert response to {} format: {}", format, e) 
            })?;

        Ok(ai_response)
    }

    async fn handle_unified_call(&self, call: UnifiedToolCall) -> McpGatewayResult<UnifiedToolResponse> {
        debug!("Handling unified tool call: {}", call.name);

        let result = match call.name.as_str() {
            "maa_status" => {
                let params: crate::mcp_tools::MaaStatusParams = serde_json::from_value(call.arguments)
                    .map_err(|e| McpGatewayError::InvalidRequest { 
                        message: format!("Invalid maa_status parameters: {}", e) 
                    })?;
                self.maa_tools.maa_status(params).await
                    .map(|response| serde_json::to_value(response).unwrap())
            }
            "maa_command" => {
                let params: crate::mcp_tools::MaaCommandParams = serde_json::from_value(call.arguments)
                    .map_err(|e| McpGatewayError::InvalidRequest { 
                        message: format!("Invalid maa_command parameters: {}", e) 
                    })?;
                self.maa_tools.maa_command(params).await
                    .map(|response| serde_json::to_value(response).unwrap())
            }
            "maa_copilot" => {
                let params: crate::mcp_tools::MaaCopilotParams = serde_json::from_value(call.arguments)
                    .map_err(|e| McpGatewayError::InvalidRequest { 
                        message: format!("Invalid maa_copilot parameters: {}", e) 
                    })?;
                self.maa_tools.maa_copilot(params).await
                    .map(|response| serde_json::to_value(response).unwrap())
            }
            "maa_operators" => {
                let params: crate::mcp_tools::MaaOperatorsParams = serde_json::from_value(call.arguments)
                    .map_err(|e| McpGatewayError::InvalidRequest { 
                        message: format!("Invalid maa_operators parameters: {}", e) 
                    })?;
                self.maa_tools.maa_operators(params).await
                    .map(|response| serde_json::to_value(response).unwrap())
            }
            _ => {
                return Ok(UnifiedToolResponse::error(
                    call.id,
                    format!("Unknown tool: {}", call.name)
                ));
            }
        };

        match result {
            Ok(data) => Ok(UnifiedToolResponse::success(call.id, data)),
            Err(error) => Ok(UnifiedToolResponse::error(call.id, error)),
        }
    }

    fn get_supported_formats(&self) -> Vec<String> {
        self.format_handlers.keys().cloned().collect()
    }

    async fn get_available_tools(&self) -> McpGatewayResult<serde_json::Value> {
        Ok(serde_json::json!({
            "tools": [
                {
                    "name": "maa_status",
                    "description": "Get MAA adapter status and information",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "verbose": {
                                "type": "boolean",
                                "description": "Include detailed information"
                            }
                        }
                    }
                },
                {
                    "name": "maa_command", 
                    "description": "Execute MAA commands from natural language",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "command": {
                                "type": "string",
                                "description": "Natural language command"
                            },
                            "context": {
                                "type": "string",
                                "description": "Optional context information"
                            }
                        },
                        "required": ["command"]
                    }
                },
                {
                    "name": "maa_copilot",
                    "description": "Execute MAA copilot operations", 
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "copilot_config": {
                                "type": "object",
                                "description": "Copilot configuration JSON"
                            },
                            "name": {
                                "type": "string",
                                "description": "Optional copilot name"
                            }
                        },
                        "required": ["copilot_config"]
                    }
                },
                {
                    "name": "maa_operators",
                    "description": "Query and manage MAA operators",
                    "parameters": {
                        "type": "object", 
                        "properties": {
                            "query_type": {
                                "type": "string",
                                "enum": ["list", "search"],
                                "description": "Type of operator query"
                            },
                            "query": {
                                "type": "string",
                                "description": "Search query (for search type)"
                            }
                        },
                        "required": ["query_type"]
                    }
                }
            ],
            "supported_formats": self.get_supported_formats()
        }))
    }
}

/// HTTP处理器类型定义
type GatewayState = Arc<McpGateway>;

/// 健康检查端点
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "mcp-gateway",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// 列出可用工具
async fn list_tools(State(gateway): State<GatewayState>) -> Result<Json<serde_json::Value>, StatusCode> {
    match gateway.get_available_tools().await {
        Ok(tools) => Ok(Json(tools)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 处理特定格式的Function Calling
async fn handle_format_call(
    Path(format): Path<String>,
    State(gateway): State<GatewayState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    debug!("Received {} format call: {:?}", format, request);

    match gateway.handle_function_call(&format, request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Format call failed: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// 处理统一格式的工具调用
async fn handle_unified_call(
    State(gateway): State<GatewayState>,
    Json(call): Json<UnifiedToolCall>,
) -> Result<Json<UnifiedToolResponse>, StatusCode> {
    debug!("Received unified call: {:?}", call);

    match gateway.handle_unified_call(call).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Unified call failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maa_adapter::{MaaAdapter, MaaAdapterTrait, MaaConfig};
    use crate::mcp_tools::register_rmcp_tools;

    #[tokio::test]
    async fn test_gateway_creation() {
        let maa_config = MaaConfig::default();
        let maa_adapter = Arc::new(MaaAdapter::new(maa_config).await.unwrap());
        let maa_tools = Arc::new(register_rmcp_tools(maa_adapter));
        
        let gateway_config = GatewayConfig::default();
        let gateway = McpGateway::new(maa_tools, gateway_config);
        
        let formats = gateway.get_supported_formats();
        assert!(formats.contains(&"openai".to_string()));
        assert!(formats.contains(&"claude".to_string()));
    }

    #[tokio::test]
    async fn test_unified_tool_call() {
        let maa_config = MaaConfig::default();
        let maa_adapter = Arc::new(MaaAdapter::new(maa_config).await.unwrap());
        let maa_tools = Arc::new(register_rmcp_tools(maa_adapter));
        
        let gateway_config = GatewayConfig::default();
        let gateway = McpGateway::new(maa_tools, gateway_config);
        
        let call = UnifiedToolCall {
            name: "maa_status".to_string(),
            arguments: serde_json::json!({"verbose": false}),
            id: Some("test-123".to_string()),
        };
        
        let response = gateway.handle_unified_call(call).await.unwrap();
        assert!(response.success);
        assert_eq!(response.id, Some("test-123".to_string()));
    }
}