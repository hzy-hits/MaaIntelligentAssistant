//! Gateway Request and Response Handlers
//!
//! 定义网关请求和响应的数据结构

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 网关请求结构
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GatewayRequest {
    /// 请求ID
    pub id: Option<String>,
    /// AI格式类型
    pub format: String,
    /// 请求头信息
    pub headers: Option<HashMap<String, String>>,
    /// 请求体
    pub body: serde_json::Value,
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl GatewayRequest {
    pub fn new(format: String, body: serde_json::Value) -> Self {
        Self {
            id: None,
            format,
            headers: None,
            body,
            timestamp: chrono::Utc::now(),
        }
    }
    
    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }
    
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = Some(headers);
        self
    }
}

/// 网关响应结构
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GatewayResponse {
    /// 响应ID（对应请求ID）
    pub id: Option<String>,
    /// 是否成功
    pub success: bool,
    /// 响应数据
    pub data: Option<serde_json::Value>,
    /// 错误信息
    pub error: Option<GatewayErrorInfo>,
    /// 处理时间（毫秒）
    pub processing_time_ms: Option<u64>,
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 错误信息结构
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GatewayErrorInfo {
    /// 错误代码
    pub code: String,
    /// 错误消息
    pub message: String,
    /// 详细信息
    pub details: Option<serde_json::Value>,
}

impl GatewayResponse {
    pub fn success(id: Option<String>, data: serde_json::Value) -> Self {
        Self {
            id,
            success: true,
            data: Some(data),
            error: None,
            processing_time_ms: None,
            timestamp: chrono::Utc::now(),
        }
    }
    
    pub fn error(id: Option<String>, code: String, message: String) -> Self {
        Self {
            id,
            success: false,
            data: None,
            error: Some(GatewayErrorInfo {
                code,
                message,
                details: None,
            }),
            processing_time_ms: None,
            timestamp: chrono::Utc::now(),
        }
    }
    
    pub fn with_processing_time(mut self, time_ms: u64) -> Self {
        self.processing_time_ms = Some(time_ms);
        self
    }
}

/// 工具调用结构（用于批量调用）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolCall {
    /// 工具名称
    pub name: String,
    /// 工具参数
    pub parameters: serde_json::Value,
    /// 调用ID
    pub id: Option<String>,
    /// 优先级（可选）
    pub priority: Option<u8>,
    /// 超时时间（秒）
    pub timeout_seconds: Option<u32>,
}

impl ToolCall {
    pub fn new(name: String, parameters: serde_json::Value) -> Self {
        Self {
            name,
            parameters,
            id: None,
            priority: None,
            timeout_seconds: None,
        }
    }
    
    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }
    
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = Some(priority);
        self
    }
    
    pub fn with_timeout(mut self, timeout_seconds: u32) -> Self {
        self.timeout_seconds = Some(timeout_seconds);
        self
    }
}

/// 批量工具调用请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchToolCallRequest {
    /// 请求ID
    pub id: Option<String>,
    /// 工具调用列表
    pub calls: Vec<ToolCall>,
    /// 是否并行执行
    pub parallel: Option<bool>,
    /// 全局超时时间（秒）
    pub global_timeout_seconds: Option<u32>,
}

/// 批量工具调用响应
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchToolCallResponse {
    /// 响应ID
    pub id: Option<String>,
    /// 调用结果
    pub results: Vec<ToolCallResult>,
    /// 总体状态
    pub overall_success: bool,
    /// 执行时间统计
    pub timing: ExecutionTiming,
}

/// 单个工具调用结果
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolCallResult {
    /// 调用ID
    pub id: Option<String>,
    /// 工具名称
    pub tool_name: String,
    /// 是否成功
    pub success: bool,
    /// 结果数据
    pub data: Option<serde_json::Value>,
    /// 错误信息
    pub error: Option<String>,
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
}

/// 执行时间统计
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecutionTiming {
    /// 总执行时间（毫秒）
    pub total_time_ms: u64,
    /// 平均执行时间（毫秒）
    pub average_time_ms: u64,
    /// 最长执行时间（毫秒）
    pub max_time_ms: u64,
    /// 最短执行时间（毫秒）
    pub min_time_ms: u64,
}

impl ExecutionTiming {
    pub fn from_results(results: &[ToolCallResult]) -> Self {
        if results.is_empty() {
            return Self {
                total_time_ms: 0,
                average_time_ms: 0,
                max_time_ms: 0,
                min_time_ms: 0,
            };
        }
        
        let times: Vec<u64> = results.iter().map(|r| r.execution_time_ms).collect();
        let total = times.iter().sum();
        let avg = total / times.len() as u64;
        let max = *times.iter().max().unwrap_or(&0);
        let min = *times.iter().min().unwrap_or(&0);
        
        Self {
            total_time_ms: total,
            average_time_ms: avg,
            max_time_ms: max,
            min_time_ms: min,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_request_creation() {
        let request = GatewayRequest::new(
            "openai".to_string(),
            serde_json::json!({"test": "data"})
        ).with_id("req-123".to_string());
        
        assert_eq!(request.format, "openai");
        assert_eq!(request.id, Some("req-123".to_string()));
        assert_eq!(request.body["test"], "data");
    }

    #[test]
    fn test_gateway_response_creation() {
        let response = GatewayResponse::success(
            Some("req-123".to_string()),
            serde_json::json!({"result": "ok"})
        ).with_processing_time(150);
        
        assert!(response.success);
        assert_eq!(response.id, Some("req-123".to_string()));
        assert_eq!(response.processing_time_ms, Some(150));
    }

    #[test]
    fn test_tool_call_creation() {
        let call = ToolCall::new(
            "maa_status".to_string(),
            serde_json::json!({"verbose": true})
        ).with_id("call-456".to_string())
         .with_priority(1)
         .with_timeout(30);
        
        assert_eq!(call.name, "maa_status");
        assert_eq!(call.id, Some("call-456".to_string()));
        assert_eq!(call.priority, Some(1));
        assert_eq!(call.timeout_seconds, Some(30));
    }

    #[test]
    fn test_execution_timing() {
        let results = vec![
            ToolCallResult {
                id: Some("1".to_string()),
                tool_name: "test1".to_string(),
                success: true,
                data: None,
                error: None,
                execution_time_ms: 100,
            },
            ToolCallResult {
                id: Some("2".to_string()),
                tool_name: "test2".to_string(),
                success: true,
                data: None,
                error: None,
                execution_time_ms: 200,
            },
        ];
        
        let timing = ExecutionTiming::from_results(&results);
        assert_eq!(timing.total_time_ms, 300);
        assert_eq!(timing.average_time_ms, 150);
        assert_eq!(timing.max_time_ms, 200);
        assert_eq!(timing.min_time_ms, 100);
    }
}