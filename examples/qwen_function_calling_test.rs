//! Qwen API Function Calling 集成测试
//! 
//! 测试我们的MAA Function Calling工具与Qwen大模型的集成

use std::sync::Arc;
use serde_json::{json, Value};
use tracing::{info, error, debug};
use maa_intelligent_server::{
    MaaAdapter, MaaConfig, MaaAdapterTrait,
    AiClient, AiClientConfig, AiProvider, ProviderConfig, AiClientTrait,
    ChatMessage, Tool,
    create_function_server,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    info!("🚀 开始 Qwen Function Calling 集成测试");
    
    // 1. 创建MAA适配器
    let maa_config = MaaConfig::default();
    let maa_adapter: Arc<dyn MaaAdapterTrait + Send + Sync> = Arc::new(MaaAdapter::new(maa_config).await?);
    info!("✅ MAA适配器创建成功");
    
    // 2. 创建Function Calling服务器
    let function_server = create_function_server(maa_adapter.clone());
    let tool_definitions = function_server.get_function_definitions();
    info!("✅ Function Calling服务器创建成功，工具数量: {}", tool_definitions.len());
    
    // 3. 转换工具定义为AI客户端格式
    let ai_tools: Vec<Tool> = tool_definitions.into_iter().map(|def| {
        Tool {
            name: def.name,
            description: def.description,
            parameters: def.parameters,
        }
    }).collect();
    
    // 打印工具定义
    info!("📋 可用工具:");
    for tool in &ai_tools {
        info!("  - {}: {}", tool.name, tool.description);
    }
    
    // 4. 配置Qwen API客户端
    let qwen_config = ProviderConfig::new("qwen-plus-2025-04-28")
        .with_api_key(&std::env::var("QWEN_API_KEY").expect("QWEN_API_KEY environment variable not set"))
        .with_base_url("https://dashscope.aliyuncs.com/compatible-mode/v1")
        .with_temperature(0.7);
    
    let ai_config = AiClientConfig::new(AiProvider::Qwen)
        .add_provider(AiProvider::Qwen, qwen_config);
    
    let ai_client = AiClient::new(ai_config)?;
    info!("✅ Qwen API客户端创建成功");
    
    // 5. 测试场景列表
    let test_scenarios = vec![
        ("简单状态查询", "请帮我查看MAA的状态"),
        ("截图命令", "帮我截个图看看现在游戏状态"),
        ("干员查询", "查询我的干员信息"),
        ("日常任务", "帮我做日常任务"),
        ("复杂命令", "现在是下午，帮我检查MAA状态，如果没问题就截个图，然后做日常任务"),
    ];
    
    // 6. 执行测试场景
    for (scenario_name, user_message) in test_scenarios {
        info!("\n🧪 测试场景: {}", scenario_name);
        info!("👤 用户输入: {}", user_message);
        
        // 构建对话消息
        let messages = vec![
            ChatMessage::system("你是MAA智能助手，可以通过Function Calling控制MAA执行明日方舟相关操作。根据用户需求调用合适的工具。"),
            ChatMessage::user(user_message),
        ];
        
        // 发送给Qwen API进行Function Calling
        match ai_client.chat_completion_with_tools(messages, ai_tools.clone()).await {
            Ok(result) => {
                use maa_intelligent_server::ai_client::client::Either;
                match result {
                    Either::Left(text_response) => {
                        info!("🤖 Qwen文本响应: {}", text_response);
                    }
                    Either::Right(function_calls) => {
                        info!("🔧 Qwen调用了 {} 个函数:", function_calls.len());
                        
                        // 执行每个函数调用
                        for function_call in function_calls {
                            info!("  📞 调用函数: {} with args: {:?}", function_call.name, function_call.arguments);
                            
                            // 转换为我们的Function Call格式
                            let maa_function_call = maa_intelligent_server::mcp_tools::FunctionCall {
                                name: function_call.name.clone(),
                                arguments: function_call.arguments,
                            };
                            
                            // 执行函数
                            let response = function_server.execute_function(maa_function_call).await;
                            
                            if response.success {
                                info!("  ✅ 函数执行成功: {:?}", response.result);
                            } else {
                                error!("  ❌ 函数执行失败: {:?}", response.error);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error!("❌ API调用失败: {}", e);
            }
        }
        
        // 等待一下再进行下一个测试
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
    
    // 7. 测试思考模式
    info!("\n🧠 测试Qwen思考模式");
    test_thinking_mode(&ai_client, &ai_tools, &function_server).await?;
    
    info!("\n🎉 所有测试完成!");
    Ok(())
}

/// 测试Qwen的思考模式
async fn test_thinking_mode(
    ai_client: &AiClient,
    ai_tools: &[Tool],
    function_server: &maa_intelligent_server::mcp_tools::MaaFunctionServer,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("测试Qwen思考模式下的Function Calling");
    
    let messages = vec![
        ChatMessage::system("你是MAA智能助手。请仔细思考用户的需求，然后调用合适的工具。"),
        ChatMessage::user("我想知道现在MAA的状态，如果一切正常就帮我执行日常任务。请先思考一下这个需求需要哪些步骤。"),
    ];
    
    // 使用直接的HTTP请求来测试思考模式
    let client = reqwest::Client::new();
    let request_body = json!({
        "model": "qwen-plus-2025-04-28",
        "messages": [
            {
                "role": "system",
                "content": "你是MAA智能助手，可以通过Function Calling控制MAA执行明日方舟相关操作。请先思考用户需求，然后调用合适的工具。"
            },
            {
                "role": "user", 
                "content": "我想知道现在MAA的状态，如果一切正常就帮我执行日常任务。请先思考一下这个需求需要哪些步骤。"
            }
        ],
        "tools": ai_tools.iter().map(|tool| {
            json!({
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": tool.parameters
                }
            })
        }).collect::<Vec<_>>(),
        "enable_thinking": true,
        "stream": false
    });
    
    let response = client
        .post("https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions")
        .header("Authorization", &format!("Bearer {}", std::env::var("QWEN_API_KEY").expect("QWEN_API_KEY environment variable not set")))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;
    
    let response_text = response.text().await?;
    debug!("Qwen思考模式响应: {}", response_text);
    
    // 解析响应
    let response_json: Value = serde_json::from_str(&response_text)?;
    
    if let Some(choices) = response_json["choices"].as_array() {
        if let Some(choice) = choices.first() {
            // 检查思考过程
            if let Some(thinking) = choice["message"]["thinking"].as_str() {
                info!("🧠 Qwen思考过程:");
                println!("{}", thinking);
            }
            
            // 检查工具调用
            if let Some(tool_calls) = choice["message"]["tool_calls"].as_array() {
                info!("🔧 思考后的工具调用 ({} 个):", tool_calls.len());
                
                for tool_call in tool_calls {
                    if let Some(function_name) = tool_call["function"]["name"].as_str() {
                        let arguments = &tool_call["function"]["arguments"];
                        info!("  📞 {}: {}", function_name, arguments);
                        
                        // 解析参数并执行
                        let args: Value = if let Some(args_str) = arguments.as_str() {
                            serde_json::from_str(args_str).unwrap_or_else(|_| json!({}))
                        } else {
                            arguments.clone()
                        };
                        
                        let maa_function_call = maa_intelligent_server::mcp_tools::FunctionCall {
                            name: function_name.to_string(),
                            arguments: args,
                        };
                        
                        let response = function_server.execute_function(maa_function_call).await;
                        if response.success {
                            info!("  ✅ 执行成功: {:?}", response.result);
                        } else {
                            error!("  ❌ 执行失败: {:?}", response.error);
                        }
                    }
                }
            } else if let Some(content) = choice["message"]["content"].as_str() {
                info!("🤖 Qwen文本响应: {}", content);
            }
        }
    }
    
    Ok(())
}