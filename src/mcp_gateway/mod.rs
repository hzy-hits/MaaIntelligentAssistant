//! MCP Gateway Module
//!
//! 这个模块实现了从云端AI到本地MAA的关键桥梁，支持多种AI服务格式的Function Calling

pub mod server;
pub mod formats;
pub mod handlers;

pub use server::{McpGateway, GatewayConfig, GatewayError};
pub use formats::{FormatHandler, OpenAIHandler, ClaudeHandler, QwenHandler, KimiHandler};
pub use handlers::{GatewayRequest, GatewayResponse, ToolCall};


use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::error;

// use crate::mcp_tools::MaaToolsHandler;

/// MCP网关错误类型
#[derive(Debug, thiserror::Error)]
pub enum McpGatewayError {
    #[error("Unsupported AI format: {format}")]
    UnsupportedFormat { format: String },
    
    #[error("Invalid request: {message}")]
    InvalidRequest { message: String },
    
    #[error("Tool execution failed: {tool} - {message}")]
    ToolExecutionError { tool: String, message: String },
    
    #[error("Format conversion failed: {message}")]
    FormatConversion { message: String },
    
    #[error("Server error: {message}")]
    ServerError { message: String },
}

/// MCP网关结果类型
pub type McpGatewayResult<T> = Result<T, McpGatewayError>;

/// 通用的工具调用请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UnifiedToolCall {
    /// 工具名称
    pub name: String,
    /// 工具参数
    pub arguments: serde_json::Value,
    /// 调用ID（用于跟踪）
    pub id: Option<String>,
}

/// 通用的工具调用响应
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UnifiedToolResponse {
    /// 调用ID
    pub id: Option<String>,
    /// 是否成功
    pub success: bool,
    /// 响应数据
    pub data: Option<serde_json::Value>,
    /// 错误信息
    pub error: Option<String>,
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl UnifiedToolResponse {
    pub fn success(id: Option<String>, data: serde_json::Value) -> Self {
        Self {
            id,
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn error(id: Option<String>, error: impl Into<String>) -> Self {
        Self {
            id,
            success: false,
            data: None,
            error: Some(error.into()),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// MCP网关核心trait
#[async_trait]
pub trait McpGatewayTrait {
    /// 处理来自AI的Function Calling请求
    async fn handle_function_call(&self, format: &str, request: serde_json::Value) -> McpGatewayResult<serde_json::Value>;
    
    /// 处理统一格式的工具调用
    async fn handle_unified_call(&self, call: UnifiedToolCall) -> McpGatewayResult<UnifiedToolResponse>;
    
    /// 获取支持的AI格式列表
    fn get_supported_formats(&self) -> Vec<String>;
    
    /// 获取可用工具列表
    async fn get_available_tools(&self) -> McpGatewayResult<serde_json::Value>;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_unified_tool_response() {
        let success_resp = UnifiedToolResponse::success(
            Some("test-123".to_string()),
            serde_json::json!({"status": "ok"})
        );
        
        assert!(success_resp.success);
        assert_eq!(success_resp.id, Some("test-123".to_string()));
        assert!(success_resp.data.is_some());
        
        let error_resp = UnifiedToolResponse::error(
            Some("test-456".to_string()),
            "Test error"
        );
        
        assert!(!error_resp.success);
        assert_eq!(error_resp.error, Some("Test error".to_string()));
    }
}