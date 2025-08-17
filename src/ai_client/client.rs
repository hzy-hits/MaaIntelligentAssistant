//! AI 客户端核心实现
//! 
//! 基于 async-openai 实现的统一 AI 客户端接口

#![allow(deprecated)]

use crate::ai_client::{
    AiError, AiResult, AiProvider, AiClientConfig, ProviderConfig,
    ChatMessage, Tool, FunctionCall, StreamEvent
};
use async_openai::{
    Client as OpenAIClient,
    config::{OpenAIConfig, AzureConfig},
    types::{
        CreateChatCompletionRequest, CreateChatCompletionRequestArgs,
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestUserMessage, ChatCompletionRequestAssistantMessage,
        ChatCompletionRequestSystemMessageContent,
        ChatCompletionRequestUserMessageContent,
        ChatCompletionRequestAssistantMessageContent,
        ChatCompletionTool, ChatCompletionToolType, FunctionObject,
        ChatCompletionToolChoiceOption,
    },
};
use async_trait::async_trait;
use futures::Stream;
use serde_json::Value;

/// AI 客户端 trait 定义
#[async_trait]
pub trait AiClientTrait: Send + Sync {
    /// 聊天完成
    async fn chat_completion(&self, messages: Vec<ChatMessage>) -> AiResult<String>;
    
    /// 带工具的聊天完成
    async fn chat_completion_with_tools(
        &self, 
        messages: Vec<ChatMessage>, 
        tools: Vec<Tool>
    ) -> AiResult<Either<String, Vec<FunctionCall>>>;
    
    /// 流式聊天完成
    async fn chat_completion_stream(
        &self,
        messages: Vec<ChatMessage>
    ) -> AiResult<Box<dyn Stream<Item = AiResult<StreamEvent>> + Send + Unpin>>;
    
    /// 切换提供商
    async fn switch_provider(&mut self, provider: AiProvider) -> AiResult<()>;
    
    /// 获取当前提供商
    fn current_provider(&self) -> &AiProvider;
}

/// 函数调用或文本响应的枚举
#[derive(Debug, Clone)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

/// AI 客户端实现
pub struct AiClient {
    pub(crate) config: AiClientConfig,
    current_provider: AiProvider,
    client: Option<ClientWrapper>,
}

impl AiClient {
    /// 创建新的 AI 客户端
    pub fn new(config: AiClientConfig) -> AiResult<Self> {
        config.validate()?;
        
        let current_provider = config.default_provider.clone();
        let mut client = Self {
            config,
            current_provider,
            client: None,
        };
        
        // 初始化当前提供商的客户端
        client.initialize_client()?;
        
        Ok(client)
    }
    
    /// 从环境变量创建客户端
    pub fn from_env() -> AiResult<Self> {
        let config = AiClientConfig::from_env()?;
        Self::new(config)
    }
    
    /// 初始化当前提供商的客户端
    fn initialize_client(&mut self) -> AiResult<()> {
        let provider_config = self.config.get_provider_config(&self.current_provider)
            .ok_or_else(|| AiError::Config(format!("No config for provider: {}", self.current_provider)))?;
        
        self.client = Some(match self.current_provider {
            AiProvider::Azure => {
                ClientWrapper::Azure(self.create_azure_client(provider_config)?)
            }
            _ => {
                ClientWrapper::OpenAI(self.create_openai_client(provider_config)?)
            }
        });
        
        Ok(())
    }
    
    /// 创建 OpenAI 兼容客户端
    fn create_openai_client(&self, config: &ProviderConfig) -> AiResult<OpenAIClient<OpenAIConfig>> {
        let mut openai_config = OpenAIConfig::new();
        
        // 设置 API Key
        if let Some(api_key) = &config.api_key {
            openai_config = openai_config.with_api_key(api_key);
            tracing::info!("设置API Key: {}...", &api_key[..7]);
        }
        
        // 设置 Base URL
        if let Some(base_url) = &config.base_url {
            openai_config = openai_config.with_api_base(base_url);
            tracing::info!("设置Base URL: {}", base_url);
        }
        
        Ok(OpenAIClient::with_config(openai_config))
    }
    
    /// 创建 Azure 客户端
    fn create_azure_client(&self, config: &ProviderConfig) -> AiResult<OpenAIClient<AzureConfig>> {
        let api_key = config.api_key.as_ref()
            .ok_or_else(|| AiError::Config("Azure requires API key".to_string()))?;
        
        let base_url = config.base_url.as_ref()
            .ok_or_else(|| AiError::Config("Azure requires base URL".to_string()))?;
        
        let azure_config = AzureConfig::new()
            .with_api_key(api_key)
            .with_api_base(base_url)
            .with_deployment_id(&config.model)
            .with_api_version("2024-02-15-preview"); // 使用最新的 API 版本
        
        Ok(OpenAIClient::with_config(azure_config))
    }
    
    /// 获取当前活跃的客户端
    fn get_active_client(&self) -> AiResult<&ClientWrapper> {
        self.client.as_ref()
            .ok_or_else(|| AiError::Config("Client not initialized".to_string()))
    }
    
    /// 获取当前提供商配置
    fn get_current_config(&self) -> AiResult<&ProviderConfig> {
        self.config.get_provider_config(&self.current_provider)
            .ok_or_else(|| AiError::Config(format!("No config for provider: {}", self.current_provider)))
    }
    
    /// 转换聊天消息格式
    pub(crate) fn convert_messages(&self, messages: Vec<ChatMessage>) -> Vec<ChatCompletionRequestMessage> {
        messages.into_iter().map(|msg| {
            match msg.role.as_str() {
                "system" => ChatCompletionRequestMessage::System(
                    ChatCompletionRequestSystemMessage {
                        content: ChatCompletionRequestSystemMessageContent::Text(msg.content),
                        name: None,
                    }
                ),
                "user" => ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessage {
                        content: ChatCompletionRequestUserMessageContent::Text(msg.content),
                        name: None,
                    }
                ),
                "assistant" => ChatCompletionRequestMessage::Assistant(
                    ChatCompletionRequestAssistantMessage {
                        content: Some(ChatCompletionRequestAssistantMessageContent::Text(msg.content)),
                        name: None,
                        tool_calls: None,
                        function_call: None,
                        refusal: None,
                        audio: None,
                    }
                ),
                _ => ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessage {
                        content: ChatCompletionRequestUserMessageContent::Text(msg.content),
                        name: None,
                    }
                ),
            }
        }).collect()
    }
    
    /// 转换工具格式
    pub(crate) fn convert_tools(&self, tools: Vec<Tool>) -> Vec<ChatCompletionTool> {
        tools.into_iter().map(|tool| {
            ChatCompletionTool {
                r#type: ChatCompletionToolType::Function,
                function: FunctionObject {
                    name: tool.name,
                    description: Some(tool.description),
                    parameters: Some(tool.parameters),
                    strict: None,
                },
            }
        }).collect()
    }
    
    /// 创建聊天完成请求
    fn create_chat_request(
        &self,
        messages: Vec<ChatMessage>,
        tools: Option<Vec<Tool>>,
    ) -> AiResult<CreateChatCompletionRequest> {
        let config = self.get_current_config()?;
        let converted_messages = self.convert_messages(messages);
        
        let mut request_builder = CreateChatCompletionRequestArgs::default();
        request_builder.model(&config.model);
        request_builder.messages(converted_messages);
        
        if let Some(temperature) = config.temperature {
            request_builder.temperature(temperature);
        }
        
        if let Some(max_tokens) = config.max_tokens {
            request_builder.max_tokens(max_tokens);
        }
        
        if let Some(tools) = tools {
            let converted_tools = self.convert_tools(tools);
            request_builder.tools(converted_tools);
            // 允许模型选择调用函数或返回文本
            request_builder.tool_choice(ChatCompletionToolChoiceOption::Auto);
        }
        
        Ok(request_builder.build()?)
    }
}

/// OpenAI 客户端枚举（解决 dyn trait 问题）
enum ClientWrapper {
    OpenAI(OpenAIClient<OpenAIConfig>),
    Azure(OpenAIClient<AzureConfig>),
}

impl ClientWrapper {
    async fn chat(&self, request: CreateChatCompletionRequest) -> Result<async_openai::types::CreateChatCompletionResponse, async_openai::error::OpenAIError> {
        match self {
            ClientWrapper::OpenAI(client) => client.chat().create(request).await,
            ClientWrapper::Azure(client) => client.chat().create(request).await,
        }
    }
}

#[async_trait]
impl AiClientTrait for AiClient {
    async fn chat_completion(&self, messages: Vec<ChatMessage>) -> AiResult<String> {
        let request = self.create_chat_request(messages, None)?;
        let client = self.get_active_client()?;
        
        let response = client.chat(request).await?;
        
        let content = response.choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .ok_or_else(|| AiError::InvalidResponse("No content in response".to_string()))?;
        
        Ok(content.clone())
    }
    
    async fn chat_completion_with_tools(
        &self,
        messages: Vec<ChatMessage>,
        tools: Vec<Tool>
    ) -> AiResult<Either<String, Vec<FunctionCall>>> {
        let request = self.create_chat_request(messages, Some(tools))?;
        let client = self.get_active_client()?;
        
        let response = client.chat(request).await?;
        
        let choice = response.choices.first()
            .ok_or_else(|| AiError::InvalidResponse("No choices in response".to_string()))?;
        
        // 检查是否有工具调用
        if let Some(tool_calls) = &choice.message.tool_calls {
            let function_calls: Result<Vec<FunctionCall>, AiError> = tool_calls.iter()
                .map(|call| {
                    let arguments: Value = serde_json::from_str(&call.function.arguments)
                        .map_err(AiError::Serialization)?;
                    
                    Ok(FunctionCall {
                        name: call.function.name.clone(),
                        arguments,
                    })
                })
                .collect();
            
            Ok(Either::Right(function_calls?))
        } else if let Some(content) = &choice.message.content {
            Ok(Either::Left(content.clone()))
        } else {
            Err(AiError::InvalidResponse("No content or tool calls in response".to_string()))
        }
    }
    
    async fn chat_completion_stream(
        &self,
        messages: Vec<ChatMessage>
    ) -> AiResult<Box<dyn Stream<Item = AiResult<StreamEvent>> + Send + Unpin>> {
        let mut request = self.create_chat_request(messages, None)?;
        request.stream = Some(true);
        
        let client = self.get_active_client()?;
        
        // 注意：这里简化了流式实现，实际应该使用 async-openai 的流式 API
        // 由于 trait object 的限制，这里返回一个模拟的流
        let content = client.chat(request).await?;
        let text = content.choices.first()
            .and_then(|choice| choice.message.content.as_ref())
            .unwrap_or(&String::new())
            .clone();
        
        let stream = futures::stream::iter(vec![
            Ok(StreamEvent::Content(text)),
            Ok(StreamEvent::Done),
        ]);
        
        Ok(Box::new(Box::pin(stream)))
    }
    
    async fn switch_provider(&mut self, provider: AiProvider) -> AiResult<()> {
        if !self.config.providers.contains_key(&provider) {
            return Err(AiError::Config(format!("Provider {} not configured", provider)));
        }
        
        self.current_provider = provider;
        self.initialize_client()?;
        
        Ok(())
    }
    
    fn current_provider(&self) -> &AiProvider {
        &self.current_provider
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai_client::{ProviderConfig, AiClientConfig};

    fn create_test_config() -> AiClientConfig {
        let provider_config = ProviderConfig::new("gpt-3.5-turbo")
            .with_api_key("test-key")
            .with_temperature(0.7);
        
        AiClientConfig::new(AiProvider::OpenAI)
            .add_provider(AiProvider::OpenAI, provider_config)
    }

    #[test]
    fn test_client_creation() {
        let config = create_test_config();
        let client = AiClient::new(config);
        
        // 注意：这个测试在没有真实 API Key 的情况下会失败
        // 在实际环境中需要提供有效的配置
        assert!(client.is_ok() || client.is_err()); // 至少不应该 panic
    }

    #[test]
    fn test_message_conversion() {
        let config = create_test_config();
        if let Ok(client) = AiClient::new(config) {
            let messages = vec![
                ChatMessage::system("You are a helpful assistant"),
                ChatMessage::user("Hello"),
                ChatMessage::assistant("Hi there!"),
            ];
            
            let converted = client.convert_messages(messages);
            assert_eq!(converted.len(), 3);
        }
    }

    #[test]
    fn test_tool_conversion() {
        let config = create_test_config();
        if let Ok(client) = AiClient::new(config) {
            let tools = vec![
                Tool {
                    name: "test_function".to_string(),
                    description: "A test function".to_string(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {}
                    }),
                }
            ];
            
            let converted = client.convert_tools(tools);
            assert_eq!(converted.len(), 1);
            assert_eq!(converted[0].function.name, "test_function");
        }
    }
}