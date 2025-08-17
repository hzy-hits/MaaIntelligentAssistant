//! AI 客户端配置管理

use crate::ai_client::{AiError, AiResult, AiProvider, AiProviderExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 单个提供商的配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// API Key (可选，本地服务不需要)
    pub api_key: Option<String>,
    /// API 端点 (可选，使用默认值)
    pub base_url: Option<String>,
    /// 模型名称
    pub model: String,
    /// 超时设置(秒)
    pub timeout: Option<u64>,
    /// 最大重试次数
    pub max_retries: Option<u32>,
    /// 温度参数
    pub temperature: Option<f32>,
    /// 最大 Token 数
    pub max_tokens: Option<u32>,
}

impl ProviderConfig {
    /// 创建新的提供商配置
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            api_key: None,
            base_url: None,
            model: model.into(),
            timeout: Some(60),
            max_retries: Some(3),
            temperature: Some(0.7),
            max_tokens: Some(4096),
        }
    }
    
    /// 设置 API Key
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }
    
    /// 设置 API 端点
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }
    
    /// 设置超时
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    /// 设置温度参数
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }
    
    /// 设置最大 Token 数
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }
}

/// AI 客户端总配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiClientConfig {
    /// 默认提供商
    pub default_provider: AiProvider,
    /// 各提供商配置
    pub providers: HashMap<AiProvider, ProviderConfig>,
}

impl AiClientConfig {
    /// 创建新的客户端配置
    pub fn new(default_provider: AiProvider) -> Self {
        Self {
            default_provider,
            providers: HashMap::new(),
        }
    }
    
    /// 添加提供商配置
    pub fn add_provider(mut self, provider: AiProvider, config: ProviderConfig) -> Self {
        self.providers.insert(provider, config);
        self
    }
    
    /// 获取提供商配置
    pub fn get_provider_config(&self, provider: &AiProvider) -> Option<&ProviderConfig> {
        self.providers.get(provider)
    }
    
    /// 获取默认提供商配置
    pub fn get_default_config(&self) -> Option<&ProviderConfig> {
        self.get_provider_config(&self.default_provider)
    }
    
    /// 验证所有配置
    pub fn validate(&self) -> AiResult<()> {
        // 检查默认提供商是否有配置
        if !self.providers.contains_key(&self.default_provider) {
            return Err(AiError::Config(format!(
                "Default provider {} is not configured", self.default_provider
            )));
        }
        
        // 验证每个提供商配置
        for (provider, config) in &self.providers {
            provider.validate_config(
                config.api_key.as_deref(),
                config.base_url.as_deref()
            )?;
        }
        
        Ok(())
    }
    
    /// 从环境变量构建配置
    pub fn from_env() -> AiResult<Self> {
        use std::env;
        
        // 获取默认提供商
        let default_provider = env::var("AI_PROVIDER")
            .unwrap_or_else(|_| "qwen".to_string())
            .parse::<AiProvider>()?;
        
        let mut config = AiClientConfig::new(default_provider.clone());
        
        // 添加通用配置
        if let Ok(api_key) = env::var("AI_API_KEY") {
            let model = env::var("AI_MODEL")
                .unwrap_or_else(|_| default_provider.default_model().to_string());
            
            let mut provider_config = ProviderConfig::new(model).with_api_key(api_key);
            
            // 设置可选的 base_url
            if let Ok(base_url) = env::var("AI_BASE_URL") {
                provider_config = provider_config.with_base_url(base_url);
            } else if let Some(default_url) = default_provider.default_base_url() {
                provider_config = provider_config.with_base_url(default_url);
            }
            
            // 设置可选参数
            if let Ok(timeout) = env::var("AI_TIMEOUT") {
                if let Ok(timeout) = timeout.parse::<u64>() {
                    provider_config = provider_config.with_timeout(timeout);
                }
            }
            
            if let Ok(temperature) = env::var("AI_TEMPERATURE") {
                if let Ok(temperature) = temperature.parse::<f32>() {
                    provider_config = provider_config.with_temperature(temperature);
                }
            }
            
            if let Ok(max_tokens) = env::var("AI_MAX_TOKENS") {
                if let Ok(max_tokens) = max_tokens.parse::<u32>() {
                    provider_config = provider_config.with_max_tokens(max_tokens);
                }
            }
            
            config = config.add_provider(default_provider, provider_config);
        }
        
        // 尝试加载其他提供商配置
        config = config.add_provider_from_env("OPENAI", AiProvider::OpenAI)?;
        config = config.add_provider_from_env("AZURE", AiProvider::Azure)?;
        config = config.add_provider_from_env("QWEN", AiProvider::Qwen)?;
        config = config.add_provider_from_env("KIMI", AiProvider::Kimi)?;
        config = config.add_provider_from_env("OLLAMA", AiProvider::Ollama)?;
        
        config.validate()?;
        Ok(config)
    }
    
    /// 从环境变量添加单个提供商配置
    fn add_provider_from_env(mut self, env_prefix: &str, provider: AiProvider) -> AiResult<Self> {
        use std::env;
        
        let api_key_var = format!("{}_API_KEY", env_prefix);
        let base_url_var = format!("{}_BASE_URL", env_prefix);
        let model_var = format!("{}_MODEL", env_prefix);
        
        // 如果有 API Key 或者是不需要 API Key 的提供商，添加配置
        if let Ok(api_key) = env::var(&api_key_var) {
            let model = env::var(&model_var)
                .unwrap_or_else(|_| provider.default_model().to_string());
            
            let mut config = ProviderConfig::new(model).with_api_key(api_key);
            
            if let Ok(base_url) = env::var(&base_url_var) {
                config = config.with_base_url(base_url);
            } else if let Some(default_url) = provider.default_base_url() {
                config = config.with_base_url(default_url);
            }
            
            self.providers.insert(provider, config);
        } else if !provider.requires_api_key() {
            // Ollama 等本地服务，即使没有 API Key 也可以添加
            let model = env::var(&model_var)
                .unwrap_or_else(|_| provider.default_model().to_string());
            
            let mut config = ProviderConfig::new(model);
            
            if let Ok(base_url) = env::var(&base_url_var) {
                config = config.with_base_url(base_url);
            } else if let Some(default_url) = provider.default_base_url() {
                config = config.with_base_url(default_url);
            }
            
            self.providers.insert(provider, config);
        }
        
        Ok(self)
    }
}

impl Default for AiClientConfig {
    fn default() -> Self {
        Self::new(AiProvider::Qwen)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_config_builder() {
        let config = ProviderConfig::new("gpt-4")
            .with_api_key("test-key")
            .with_base_url("https://api.test.com")
            .with_temperature(0.5)
            .with_max_tokens(2048);
        
        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.api_key, Some("test-key".to_string()));
        assert_eq!(config.base_url, Some("https://api.test.com".to_string()));
        assert_eq!(config.temperature, Some(0.5));
        assert_eq!(config.max_tokens, Some(2048));
    }

    #[test]
    fn test_client_config() {
        let provider_config = ProviderConfig::new("gpt-4").with_api_key("test-key");
        
        let config = AiClientConfig::new(AiProvider::OpenAI)
            .add_provider(AiProvider::OpenAI, provider_config);
        
        assert_eq!(config.default_provider, AiProvider::OpenAI);
        assert!(config.get_provider_config(&AiProvider::OpenAI).is_some());
        assert!(config.get_default_config().is_some());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation() {
        // 缺少默认提供商配置
        let config = AiClientConfig::new(AiProvider::OpenAI);
        assert!(config.validate().is_err());
        
        // 添加配置但缺少 API Key
        let provider_config = ProviderConfig::new("gpt-4");
        let config = config.add_provider(AiProvider::OpenAI, provider_config);
        assert!(config.validate().is_err());
        
        // 正确配置
        let provider_config = ProviderConfig::new("gpt-4").with_api_key("test-key");
        let config = AiClientConfig::new(AiProvider::OpenAI)
            .add_provider(AiProvider::OpenAI, provider_config);
        assert!(config.validate().is_ok());
    }
}