//! MAA 智能控制中间层
//! 
//! 基于 MCP 协议的 MAA 智能控制中间层，支持多种 AI 的 Function Calling 格式转换

pub mod config;
pub mod ai_client;
pub mod maa_adapter;
pub mod maa_core;
pub mod function_tools;
pub mod function_calling_server;
// pub mod operator_manager;  // Temporarily disabled due to MaaBackend dependency
pub mod copilot_matcher;

// 导出核心类型
pub use config::Config;
pub use ai_client::{
    AiClient, AiClientTrait, AiClientConfig, ProviderConfig, AiProvider, AiProviderExt,
    AiError, AiResult, ChatMessage, Tool, FunctionCall, StreamEvent
};
pub use maa_adapter::{MaaConfig, MaaStatus, MaaError, MaaResult};
pub use function_tools::{
    FunctionDefinition, FunctionCall as MaaFunctionCall, FunctionResponse,
    EnhancedMaaFunctionServer, create_enhanced_function_server
};
// pub use operator_manager::{
//     OperatorManager, OperatorManagerConfig, OperatorManagerTrait,
//     Operator, OperatorFilter, OperatorSummary, ScanResult, OperatorError, OperatorResult
// };
pub use copilot_matcher::{
    CopilotMatcher, CopilotMatcherTrait, MatcherConfig,
    CopilotData, OperatorRequirement, StageOperator, MatchStage, MatchResult, MatchScore,
    ApiClient, ApiClientTrait, ApiConfig,
    CacheManager, CacheManagerTrait, CacheConfig,
    CopilotError, CopilotResult
};

/// 应用程序错误类型
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("MCP error: {0}")]
    Mcp(String),
    
    #[error("MAA error: {0}")]
    Maa(String),
}

/// 应用程序结果类型
pub type Result<T> = std::result::Result<T, AppError>;