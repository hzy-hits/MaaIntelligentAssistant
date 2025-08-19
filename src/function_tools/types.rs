//! Function Calling 核心类型定义
//! 
//! 提供统一的错误处理、响应格式和状态管理

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Function calling工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Function calling请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// 任务执行上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub game_state: GameState,
    pub last_operations: Vec<String>,
    pub recommendations: Vec<String>,
}

/// 游戏状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub current_sanity: Option<i32>,
    pub max_sanity: Option<i32>,
    pub medicine_count: Option<i32>,
    pub stone_count: Option<i32>,
    pub recruit_tickets: Option<i32>,
    pub current_stage: Option<String>,
    pub last_login: Option<DateTime<Utc>>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            current_sanity: None,
            max_sanity: None,
            medicine_count: None,
            stone_count: None,
            recruit_tickets: None,
            current_stage: None,
            last_login: None,
        }
    }
}

impl Default for TaskContext {
    fn default() -> Self {
        Self {
            user_id: None,
            session_id: None,
            game_state: GameState::default(),
            last_operations: Vec::new(),
            recommendations: Vec::new(),
        }
    }
}

/// 增强的 Function calling 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionResponse {
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<MaaError>,
    pub timestamp: DateTime<Utc>,
    pub execution_time_ms: Option<u64>,
    pub metadata: ResponseMetadata,
}

/// MAA 错误类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaaError {
    pub error_type: ErrorType,
    pub message: String,
    pub details: Option<String>,
    pub suggestion: Option<String>,
    pub error_code: Option<String>,
}

impl std::fmt::Display for MaaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}] {}", self.error_type, self.message)
    }
}

impl std::error::Error for MaaError {}


/// 错误类型分类
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorType {
    /// 参数错误
    ParameterError,
    /// MAA 核心错误
    MaaCoreError,
    /// 设备连接错误
    DeviceError,
    /// 游戏状态错误
    GameStateError,
    /// 系统错误
    SystemError,
    /// 超时错误
    TimeoutError,
    /// 未知错误
    UnknownError,
}

/// 响应元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub task_id: Option<String>,
    pub function_name: String,
    pub recommendations: Vec<String>,
    pub next_actions: Vec<String>,
    pub resource_usage: Option<ResourceUsage>,
}

/// 资源使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub sanity_used: Option<i32>,
    pub medicine_used: Option<i32>,
    pub stone_used: Option<i32>,
    pub recruit_tickets_used: Option<i32>,
    pub items_gained: HashMap<String, i32>,
}

impl FunctionResponse {
    /// 创建成功响应
    pub fn success(function_name: &str, result: serde_json::Value) -> Self {
        Self {
            success: true,
            result: Some(result),
            error: None,
            timestamp: Utc::now(),
            execution_time_ms: None,
            metadata: ResponseMetadata {
                task_id: None,
                function_name: function_name.to_string(),
                recommendations: Vec::new(),
                next_actions: Vec::new(),
                resource_usage: None,
            },
        }
    }

    /// 创建错误响应
    pub fn error(function_name: &str, error: MaaError) -> Self {
        Self {
            success: false,
            result: None,
            error: Some(error),
            timestamp: Utc::now(),
            execution_time_ms: None,
            metadata: ResponseMetadata {
                task_id: None,
                function_name: function_name.to_string(),
                recommendations: Vec::new(),
                next_actions: Vec::new(),
                resource_usage: None,
            },
        }
    }

    /// 创建简单错误响应（向后兼容）
    pub fn simple_error(function_name: &str, message: String) -> Self {
        let error = MaaError {
            error_type: ErrorType::UnknownError,
            message,
            details: None,
            suggestion: None,
            error_code: None,
        };
        Self::error(function_name, error)
    }

    /// 设置执行时间
    pub fn with_execution_time(mut self, duration_ms: u64) -> Self {
        self.execution_time_ms = Some(duration_ms);
        self
    }

    /// 设置任务ID
    pub fn with_task_id(mut self, task_id: String) -> Self {
        self.metadata.task_id = Some(task_id);
        self
    }

    /// 添加推荐操作
    pub fn with_recommendations(mut self, recommendations: Vec<String>) -> Self {
        self.metadata.recommendations = recommendations;
        self
    }

    /// 添加后续操作
    pub fn with_next_actions(mut self, next_actions: Vec<String>) -> Self {
        self.metadata.next_actions = next_actions;
        self
    }

    /// 添加资源使用情况
    pub fn with_resource_usage(mut self, usage: ResourceUsage) -> Self {
        self.metadata.resource_usage = Some(usage);
        self
    }
}

impl MaaError {
    /// 创建参数错误
    pub fn parameter_error(message: &str, suggestion: Option<&str>) -> Self {
        Self {
            error_type: ErrorType::ParameterError,
            message: message.to_string(),
            details: None,
            suggestion: suggestion.map(|s| s.to_string()),
            error_code: Some("PARAM_ERROR".to_string()),
        }
    }

    /// 创建参数验证错误
    pub fn validation_error(message: &str, suggestion: Option<&str>) -> Self {
        Self {
            error_type: ErrorType::ParameterError,
            message: message.to_string(),
            details: None,
            suggestion: suggestion.map(|s| s.to_string()),
            error_code: Some("VALIDATION_ERROR".to_string()),
        }
    }

    /// 创建 MAA 核心错误
    pub fn maa_core_error(message: &str, details: Option<&str>) -> Self {
        Self {
            error_type: ErrorType::MaaCoreError,
            message: message.to_string(),
            details: details.map(|d| d.to_string()),
            suggestion: Some("请检查MAA设置和设备连接".to_string()),
            error_code: Some("MAA_CORE_ERROR".to_string()),
        }
    }

    /// 创建设备连接错误
    pub fn device_error(message: &str) -> Self {
        Self {
            error_type: ErrorType::DeviceError,
            message: message.to_string(),
            details: None,
            suggestion: Some("请检查设备连接、ADB设置或模拟器状态".to_string()),
            error_code: Some("DEVICE_ERROR".to_string()),
        }
    }

    /// 创建游戏状态错误
    pub fn game_state_error(message: &str, suggestion: &str) -> Self {
        Self {
            error_type: ErrorType::GameStateError,
            message: message.to_string(),
            details: None,
            suggestion: Some(suggestion.to_string()),
            error_code: Some("GAME_STATE_ERROR".to_string()),
        }
    }

    /// 创建超时错误
    pub fn timeout_error(message: &str, timeout_seconds: u64) -> Self {
        Self {
            error_type: ErrorType::TimeoutError,
            message: message.to_string(),
            details: Some(format!("超时时间: {}秒", timeout_seconds)),
            suggestion: Some("请检查网络连接或增加超时时间".to_string()),
            error_code: Some("TIMEOUT_ERROR".to_string()),
        }
    }
}