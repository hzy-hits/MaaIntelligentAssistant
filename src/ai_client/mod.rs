//! AI 客户端模块
//! 
//! 基于 async-openai 实现的统一 AI 客户端，支持多个提供商：
//! - OpenAI (官方)
//! - Azure OpenAI Service  
//! - Qwen (阿里云)
//! - Kimi (Moonshot AI)
//! - Ollama (本地部署)

pub mod client;
pub mod config;
pub mod provider;

#[cfg(test)]
mod tests;

// 重新导出核心类型
pub use client::{AiClient, AiClientTrait};
pub use config::{AiClientConfig, ProviderConfig};
pub use provider::{AiProvider, AiProviderExt};

/// AI 客户端错误类型
#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("API error: {0}")]
    Api(#[from] async_openai::error::OpenAIError),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Provider not supported: {0}")]
    UnsupportedProvider(String),
    
    #[error("Authentication failed: {0}")]
    Authentication(String),
    
    #[error("Rate limit exceeded")]
    RateLimit,
    
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Invalid response format: {0}")]
    InvalidResponse(String),
}

/// AI 客户端结果类型
pub type AiResult<T> = std::result::Result<T, AiError>;

/// 聊天消息类型
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }
    
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }
    
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}

/// 工具调用定义
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// 函数调用结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// 流式响应事件
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum StreamEvent {
    Content(String),
    FunctionCall(FunctionCall),
    Done,
    Error(String),
}