//! 配置管理模块
//! 
//! 负责从环境变量和 .env 文件加载应用配置

use crate::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::env;

/// 应用配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // AI 配置 - 兼容旧版本，新版本使用 ai_client 模块的配置
    pub ai_provider: String,
    pub ai_api_key: String,
    pub ai_api_endpoint: String,
    pub ai_model: String,

    // MAA 配置
    pub maa_device: String,
    pub maa_adb_path: String,
    pub maa_resource_path: String,

    // 服务配置
    pub server_host: String,
    pub server_port: u16,
    pub log_level: String,

    // 缓存配置
    pub cache_dir: String,
    pub cache_ttl: u64,

    // 作业站配置
    pub copilot_api_url: String,
    pub copilot_cache_ttl: u64,
}

impl Config {
    /// 从环境变量加载配置
    pub fn from_env() -> Result<Self> {
        // 尝试加载 .env 文件
        dotenvy::dotenv().ok();

        Ok(Config {
            // AI 配置
            ai_provider: env::var("AI_PROVIDER").unwrap_or_else(|_| "qwen".to_string()),
            ai_api_key: env::var("AI_API_KEY")
                .map_err(|_| AppError::Config("AI_API_KEY is required".to_string()))?,
            ai_api_endpoint: env::var("AI_API_ENDPOINT")
                .unwrap_or_else(|_| "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string()),
            ai_model: env::var("AI_MODEL").unwrap_or_else(|_| "qwen-turbo".to_string()),

            // MAA 配置
            maa_device: env::var("MAA_DEVICE").unwrap_or_else(|_| "127.0.0.1:5555".to_string()),
            maa_adb_path: env::var("MAA_ADB_PATH").unwrap_or_else(|_| "adb".to_string()),
            maa_resource_path: env::var("MAA_RESOURCE_PATH")
                .unwrap_or_else(|_| "./maa-official/resource".to_string()),

            // 服务配置
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .map_err(|_| AppError::Config("Invalid SERVER_PORT".to_string()))?,
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),

            // 缓存配置
            cache_dir: env::var("CACHE_DIR").unwrap_or_else(|_| "./data".to_string()),
            cache_ttl: env::var("CACHE_TTL")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .map_err(|_| AppError::Config("Invalid CACHE_TTL".to_string()))?,

            // 作业站配置
            copilot_api_url: env::var("COPILOT_API_URL")
                .unwrap_or_else(|_| "https://prts.maa.plus".to_string()),
            copilot_cache_ttl: env::var("COPILOT_CACHE_TTL")
                .unwrap_or_else(|_| "1800".to_string())
                .parse()
                .map_err(|_| AppError::Config("Invalid COPILOT_CACHE_TTL".to_string()))?,
        })
    }

    /// 验证配置有效性
    pub fn validate(&self) -> Result<()> {
        // 验证 AI Provider
        match self.ai_provider.as_str() {
            "qwen" | "kimi" | "openai" | "ollama" => {}
            _ => return Err(AppError::Config(
                format!("Unsupported AI provider: {}", self.ai_provider)
            )),
        }

        // 验证端口范围
        if self.server_port == 0 {
            return Err(AppError::Config(
                format!("Invalid server port: {}", self.server_port)
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        // 创建测试配置
        let mut config = Config {
            ai_provider: "qwen".to_string(),
            ai_api_key: "test-key".to_string(),
            ai_api_endpoint: "https://test.example.com".to_string(),
            ai_model: "test-model".to_string(),
            maa_device: "127.0.0.1:5555".to_string(),
            maa_adb_path: "adb".to_string(),
            maa_resource_path: "./resource".to_string(),
            server_host: "0.0.0.0".to_string(),
            server_port: 8080,
            log_level: "info".to_string(),
            cache_dir: "./data".to_string(),
            cache_ttl: 3600,
            copilot_api_url: "https://prts.maa.plus".to_string(),
            copilot_cache_ttl: 1800,
        };

        // 测试有效配置
        assert!(config.validate().is_ok());

        // 测试无效 AI Provider
        config.ai_provider = "invalid".to_string();
        assert!(config.validate().is_err());

        // 恢复有效配置
        config.ai_provider = "qwen".to_string();

        // 测试无效端口
        config.server_port = 0;
        assert!(config.validate().is_err());
    }
}