//! AI 客户端集成测试
//! 
//! 测试多提供商支持、配置管理和功能切换

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::ai_client::client::Either;
    use std::env;
    use std::str::FromStr;

    /// 创建测试用的配置
    fn create_test_config() -> AiClientConfig {
        AiClientConfig::new(AiProvider::Qwen)
            .add_provider(
                AiProvider::OpenAI,
                ProviderConfig::new("gpt-3.5-turbo")
                    .with_api_key("test-openai-key")
                    .with_temperature(0.7)
            )
            .add_provider(
                AiProvider::Azure,
                ProviderConfig::new("gpt-4")
                    .with_api_key("test-azure-key")
                    .with_base_url("https://test.openai.azure.com")
                    .with_temperature(0.5)
            )
            .add_provider(
                AiProvider::Qwen,
                ProviderConfig::new("qwen-turbo")
                    .with_api_key("test-qwen-key")
                    .with_base_url("https://dashscope.aliyuncs.com/compatible-mode/v1")
                    .with_temperature(0.8)
            )
            .add_provider(
                AiProvider::Kimi,
                ProviderConfig::new("moonshot-v1-8k")
                    .with_api_key("test-kimi-key")
                    .with_base_url("https://api.moonshot.cn/v1")
                    .with_temperature(0.6)
            )
            .add_provider(
                AiProvider::Ollama,
                ProviderConfig::new("llama2")
                    .with_base_url("http://localhost:11434/v1")
                    .with_temperature(0.9)
            )
    }

    #[test]
    fn test_provider_enum() {
        // 测试字符串解析
        assert_eq!(AiProvider::from_str("openai").unwrap(), AiProvider::OpenAI);
        assert_eq!(AiProvider::from_str("AZURE").unwrap(), AiProvider::Azure);
        assert_eq!(AiProvider::from_str("qwen").unwrap(), AiProvider::Qwen);
        assert_eq!(AiProvider::from_str("kimi").unwrap(), AiProvider::Kimi);
        assert_eq!(AiProvider::from_str("ollama").unwrap(), AiProvider::Ollama);
        
        // 测试无效提供商
        assert!(AiProvider::from_str("invalid").is_err());
        
        // 测试显示
        assert_eq!(AiProvider::OpenAI.to_string(), "openai");
        assert_eq!(AiProvider::Azure.to_string(), "azure");
    }

    #[test]
    fn test_provider_extensions() {
        // 测试默认URL
        assert_eq!(AiProvider::OpenAI.default_base_url(), None);
        assert_eq!(AiProvider::Qwen.default_base_url(), Some("https://dashscope.aliyuncs.com/compatible-mode/v1"));
        assert_eq!(AiProvider::Kimi.default_base_url(), Some("https://api.moonshot.cn/v1"));
        assert_eq!(AiProvider::Ollama.default_base_url(), Some("http://localhost:11434/v1"));
        
        // 测试默认模型
        assert_eq!(AiProvider::OpenAI.default_model(), "gpt-4");
        assert_eq!(AiProvider::Qwen.default_model(), "qwen-turbo");
        assert_eq!(AiProvider::Kimi.default_model(), "moonshot-v1-8k");
        assert_eq!(AiProvider::Ollama.default_model(), "llama2");
        
        // 测试API Key要求
        assert!(AiProvider::OpenAI.requires_api_key());
        assert!(AiProvider::Azure.requires_api_key());
        assert!(AiProvider::Qwen.requires_api_key());
        assert!(AiProvider::Kimi.requires_api_key());
        assert!(!AiProvider::Ollama.requires_api_key());
        
        // 测试功能支持
        assert!(AiProvider::OpenAI.supports_function_calling());
        assert!(AiProvider::OpenAI.supports_streaming());
    }

    #[test]
    fn test_provider_config_validation() {
        // OpenAI 需要 API Key
        assert!(AiProvider::OpenAI.validate_config(None, None).is_err());
        assert!(AiProvider::OpenAI.validate_config(Some("key"), None).is_ok());
        
        // Azure 需要 API Key 和 base URL
        assert!(AiProvider::Azure.validate_config(Some("key"), None).is_err());
        assert!(AiProvider::Azure.validate_config(Some("key"), Some("url")).is_ok());
        
        // Ollama 不需要 API Key
        assert!(AiProvider::Ollama.validate_config(None, None).is_ok());
        assert!(AiProvider::Ollama.validate_config(None, Some("url")).is_ok());
    }

    #[test]
    fn test_provider_config_builder() {
        let config = ProviderConfig::new("gpt-4")
            .with_api_key("test-key")
            .with_base_url("https://api.test.com")
            .with_timeout(120)
            .with_temperature(0.5)
            .with_max_tokens(2048);
        
        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.api_key, Some("test-key".to_string()));
        assert_eq!(config.base_url, Some("https://api.test.com".to_string()));
        assert_eq!(config.timeout, Some(120));
        assert_eq!(config.temperature, Some(0.5));
        assert_eq!(config.max_tokens, Some(2048));
    }

    #[test]
    fn test_client_config() {
        let config = create_test_config();
        
        // 测试默认提供商
        assert_eq!(config.default_provider, AiProvider::Qwen);
        
        // 测试提供商配置获取
        assert!(config.get_provider_config(&AiProvider::OpenAI).is_some());
        assert!(config.get_provider_config(&AiProvider::Azure).is_some());
        assert!(config.get_provider_config(&AiProvider::Qwen).is_some());
        assert!(config.get_provider_config(&AiProvider::Kimi).is_some());
        assert!(config.get_provider_config(&AiProvider::Ollama).is_some());
        
        // 测试默认配置获取
        let default_config = config.get_default_config().unwrap();
        assert_eq!(default_config.model, "qwen-turbo");
        
        // 测试配置验证
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_client_config_validation_errors() {
        // 测试缺少默认提供商配置
        let config = AiClientConfig::new(AiProvider::OpenAI);
        assert!(config.validate().is_err());
        
        // 测试缺少API Key的配置
        let config = AiClientConfig::new(AiProvider::OpenAI)
            .add_provider(AiProvider::OpenAI, ProviderConfig::new("gpt-4"));
        assert!(config.validate().is_err());
        
        // 测试Azure缺少base URL
        let config = AiClientConfig::new(AiProvider::Azure)
            .add_provider(
                AiProvider::Azure,
                ProviderConfig::new("gpt-4").with_api_key("key")
            );
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_chat_message_creation() {
        let system_msg = ChatMessage::system("You are a helpful assistant");
        assert_eq!(system_msg.role, "system");
        assert_eq!(system_msg.content, "You are a helpful assistant");
        
        let user_msg = ChatMessage::user("Hello, world!");
        assert_eq!(user_msg.role, "user");
        assert_eq!(user_msg.content, "Hello, world!");
        
        let assistant_msg = ChatMessage::assistant("Hi there!");
        assert_eq!(assistant_msg.role, "assistant");
        assert_eq!(assistant_msg.content, "Hi there!");
    }

    #[test]
    fn test_tool_creation() {
        let tool = Tool {
            name: "get_weather".to_string(),
            description: "Get the current weather".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "The city name"
                    }
                },
                "required": ["location"]
            }),
        };
        
        assert_eq!(tool.name, "get_weather");
        assert_eq!(tool.description, "Get the current weather");
        assert!(tool.parameters.is_object());
    }

    #[test]
    fn test_function_call_creation() {
        let func_call = FunctionCall {
            name: "get_weather".to_string(),
            arguments: serde_json::json!({
                "location": "Beijing"
            }),
        };
        
        assert_eq!(func_call.name, "get_weather");
        assert_eq!(func_call.arguments["location"], "Beijing");
    }

    // 注意：以下测试需要真实的API Key才能运行，在CI环境中应该跳过
    #[tokio::test]
    #[ignore] // 忽略需要真实API的测试
    async fn test_client_creation_with_real_config() {
        // 从环境变量创建配置（需要设置相应的环境变量）
        if let Ok(config) = AiClientConfig::from_env() {
            let client = AiClient::new(config);
            assert!(client.is_ok());
        }
    }

    #[tokio::test]
    #[ignore] // 忽略需要真实API的测试
    async fn test_chat_completion() {
        // 这个测试需要真实的API Key
        if let Ok(client) = AiClient::from_env() {
            let messages = vec![
                ChatMessage::system("You are a helpful assistant."),
                ChatMessage::user("Say hello in one word."),
            ];
            
            if let Ok(response) = client.chat_completion(messages).await {
                assert!(!response.is_empty());
            }
        }
    }

    #[tokio::test]
    #[ignore] // 忽略需要真实API的测试
    async fn test_provider_switching() {
        if let Ok(mut client) = AiClient::from_env() {
            let original_provider = client.current_provider().clone();
            
            // 尝试切换到不同的提供商（如果有配置）
            for provider in [AiProvider::OpenAI, AiProvider::Qwen, AiProvider::Kimi].iter() {
                if client.config.providers.contains_key(provider) {
                    assert!(client.switch_provider(provider.clone()).await.is_ok());
                    assert_eq!(client.current_provider(), provider);
                }
            }
            
            // 切换回原始提供商
            assert!(client.switch_provider(original_provider).await.is_ok());
        }
    }

    #[tokio::test]
    #[ignore] // 忽略需要真实API的测试
    async fn test_function_calling() {
        if let Ok(client) = AiClient::from_env() {
            let messages = vec![
                ChatMessage::system("You are a helpful assistant with access to tools."),
                ChatMessage::user("What's the weather like in Beijing?"),
            ];
            
            let tools = vec![
                Tool {
                    name: "get_weather".to_string(),
                    description: "Get the current weather in a city".to_string(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "location": {
                                "type": "string",
                                "description": "The city name"
                            }
                        },
                        "required": ["location"]
                    }),
                }
            ];
            
            match client.chat_completion_with_tools(messages, tools).await {
                Ok(Either::Left(text)) => {
                    println!("Got text response: {}", text);
                    assert!(!text.is_empty());
                }
                Ok(Either::Right(function_calls)) => {
                    println!("Got function calls: {:?}", function_calls);
                    assert!(!function_calls.is_empty());
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            }
        }
    }

    #[test]
    fn test_config_from_env_simulation() {
        // 模拟环境变量测试
        env::set_var("AI_PROVIDER", "qwen");
        env::set_var("AI_API_KEY", "test-key");
        env::set_var("AI_MODEL", "qwen-turbo");
        env::set_var("AI_BASE_URL", "https://dashscope.aliyuncs.com/compatible-mode/v1");
        env::set_var("AI_TEMPERATURE", "0.7");
        env::set_var("AI_MAX_TOKENS", "4096");
        
        // 添加特定提供商配置
        env::set_var("OPENAI_API_KEY", "openai-test-key");
        env::set_var("OPENAI_MODEL", "gpt-4");
        env::set_var("KIMI_API_KEY", "kimi-test-key");
        env::set_var("KIMI_MODEL", "moonshot-v1-8k");
        
        if let Ok(config) = AiClientConfig::from_env() {
            assert_eq!(config.default_provider, AiProvider::Qwen);
            
            // 检查默认提供商配置
            let qwen_config = config.get_provider_config(&AiProvider::Qwen).unwrap();
            assert_eq!(qwen_config.model, "qwen-turbo");
            assert_eq!(qwen_config.api_key, Some("test-key".to_string()));
            
            // 检查其他提供商配置
            if let Some(openai_config) = config.get_provider_config(&AiProvider::OpenAI) {
                assert_eq!(openai_config.model, "gpt-4");
                assert_eq!(openai_config.api_key, Some("openai-test-key".to_string()));
            }
            
            if let Some(kimi_config) = config.get_provider_config(&AiProvider::Kimi) {
                assert_eq!(kimi_config.model, "moonshot-v1-8k");
                assert_eq!(kimi_config.api_key, Some("kimi-test-key".to_string()));
            }
        }
        
        // 清理环境变量
        env::remove_var("AI_PROVIDER");
        env::remove_var("AI_API_KEY");
        env::remove_var("AI_MODEL");
        env::remove_var("AI_BASE_URL");
        env::remove_var("AI_TEMPERATURE");
        env::remove_var("AI_MAX_TOKENS");
        env::remove_var("OPENAI_API_KEY");
        env::remove_var("OPENAI_MODEL");
        env::remove_var("KIMI_API_KEY");
        env::remove_var("KIMI_MODEL");
    }
}

/// 性能测试模块
#[cfg(test)]
mod benchmarks {
    use super::super::*;
    use std::time::Instant;

    #[tokio::test]
    #[ignore] // 忽略性能测试，除非明确需要
    async fn benchmark_config_creation() {
        let start = Instant::now();
        
        for _ in 0..1000 {
            let _config = AiClientConfig::new(AiProvider::Qwen)
                .add_provider(
                    AiProvider::OpenAI,
                    ProviderConfig::new("gpt-4").with_api_key("test-key")
                )
                .add_provider(
                    AiProvider::Qwen,
                    ProviderConfig::new("qwen-turbo").with_api_key("test-key")
                );
        }
        
        let duration = start.elapsed();
        println!("创建1000个配置耗时: {:?}", duration);
        assert!(duration.as_millis() < 100); // 应该在100ms内完成
    }

    #[tokio::test]
    #[ignore] // 忽略性能测试
    async fn benchmark_message_conversion() {
        let config = AiClientConfig::new(AiProvider::Qwen)
            .add_provider(
                AiProvider::Qwen,
                ProviderConfig::new("qwen-turbo").with_api_key("test-key")
            );
        
        if let Ok(client) = AiClient::new(config) {
            let messages = vec![
                ChatMessage::system("You are a helpful assistant"),
                ChatMessage::user("Hello"),
                ChatMessage::assistant("Hi"),
            ];
            
            let start = Instant::now();
            
            for _ in 0..10000 {
                let _converted = client.convert_messages(messages.clone());
            }
            
            let duration = start.elapsed();
            println!("转换10000次消息耗时: {:?}", duration);
            assert!(duration.as_millis() < 500); // 应该在500ms内完成
        }
    }
}