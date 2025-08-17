//! AI 提供商定义和扩展方法

use crate::ai_client::{AiError, AiResult};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// AI 提供商枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiProvider {
    /// OpenAI 官方 API
    OpenAI,
    /// Azure OpenAI Service
    Azure,
    /// 阿里云通义千问
    Qwen,
    /// 月之暗面 Kimi
    Kimi,
    /// 本地 Ollama 部署
    Ollama,
}

impl FromStr for AiProvider {
    type Err = AiError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "openai" => Ok(AiProvider::OpenAI),
            "azure" => Ok(AiProvider::Azure),
            "qwen" => Ok(AiProvider::Qwen),
            "kimi" => Ok(AiProvider::Kimi),
            "ollama" => Ok(AiProvider::Ollama),
            _ => Err(AiError::UnsupportedProvider(s.to_string())),
        }
    }
}

impl std::fmt::Display for AiProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiProvider::OpenAI => write!(f, "openai"),
            AiProvider::Azure => write!(f, "azure"),
            AiProvider::Qwen => write!(f, "qwen"),
            AiProvider::Kimi => write!(f, "kimi"),
            AiProvider::Ollama => write!(f, "ollama"),
        }
    }
}

/// AI 提供商扩展方法
pub trait AiProviderExt {
    /// 获取默认的 API 端点
    fn default_base_url(&self) -> Option<&'static str>;
    
    /// 获取默认模型
    fn default_model(&self) -> &'static str;
    
    /// 是否需要 API Key
    fn requires_api_key(&self) -> bool;
    
    /// 是否支持函数调用
    fn supports_function_calling(&self) -> bool;
    
    /// 是否支持流式响应
    fn supports_streaming(&self) -> bool;
    
    /// 验证配置
    fn validate_config(&self, api_key: Option<&str>, base_url: Option<&str>) -> AiResult<()>;
}

impl AiProviderExt for AiProvider {
    fn default_base_url(&self) -> Option<&'static str> {
        match self {
            AiProvider::OpenAI => None, // 使用 async-openai 默认值
            AiProvider::Azure => None,  // 需要用户提供
            AiProvider::Qwen => Some("https://dashscope.aliyuncs.com/compatible-mode/v1"),
            AiProvider::Kimi => Some("https://api.moonshot.cn/v1"),
            AiProvider::Ollama => Some("http://localhost:11434/v1"),
        }
    }
    
    fn default_model(&self) -> &'static str {
        match self {
            AiProvider::OpenAI => "gpt-4",
            AiProvider::Azure => "gpt-4", // 用户配置的部署名称
            AiProvider::Qwen => "qwen-turbo",
            AiProvider::Kimi => "moonshot-v1-8k",
            AiProvider::Ollama => "llama2",
        }
    }
    
    fn requires_api_key(&self) -> bool {
        match self {
            AiProvider::OpenAI => true,
            AiProvider::Azure => true,
            AiProvider::Qwen => true,
            AiProvider::Kimi => true,
            AiProvider::Ollama => false, // 本地部署通常不需要
        }
    }
    
    fn supports_function_calling(&self) -> bool {
        match self {
            AiProvider::OpenAI => true,
            AiProvider::Azure => true,
            AiProvider::Qwen => true,
            AiProvider::Kimi => true,
            AiProvider::Ollama => true, // 取决于模型，这里假设支持
        }
    }
    
    fn supports_streaming(&self) -> bool {
        match self {
            AiProvider::OpenAI => true,
            AiProvider::Azure => true,
            AiProvider::Qwen => true,
            AiProvider::Kimi => true,
            AiProvider::Ollama => true,
        }
    }
    
    fn validate_config(&self, api_key: Option<&str>, base_url: Option<&str>) -> AiResult<()> {
        // 检查 API Key 要求
        if self.requires_api_key() && api_key.is_none() {
            return Err(AiError::Config(format!(
                "Provider {} requires an API key", self
            )));
        }
        
        // 检查 Azure 特殊要求
        if *self == AiProvider::Azure && base_url.is_none() {
            return Err(AiError::Config(
                "Azure provider requires a custom base URL".to_string()
            ));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_from_string() {
        assert_eq!(AiProvider::from_str("openai").unwrap(), AiProvider::OpenAI);
        assert_eq!(AiProvider::from_str("QWEN").unwrap(), AiProvider::Qwen);
        assert_eq!(AiProvider::from_str("kimi").unwrap(), AiProvider::Kimi);
        assert!(AiProvider::from_str("invalid").is_err());
    }

    #[test]
    fn test_provider_display() {
        assert_eq!(AiProvider::OpenAI.to_string(), "openai");
        assert_eq!(AiProvider::Qwen.to_string(), "qwen");
    }

    #[test]
    fn test_provider_validation() {
        // OpenAI 需要 API Key
        assert!(AiProvider::OpenAI.validate_config(None, None).is_err());
        assert!(AiProvider::OpenAI.validate_config(Some("key"), None).is_ok());
        
        // Ollama 不需要 API Key
        assert!(AiProvider::Ollama.validate_config(None, None).is_ok());
        
        // Azure 需要 API Key 和 base URL
        assert!(AiProvider::Azure.validate_config(Some("key"), None).is_err());
        assert!(AiProvider::Azure.validate_config(Some("key"), Some("url")).is_ok());
    }
}