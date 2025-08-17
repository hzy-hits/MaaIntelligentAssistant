//! Qwen API Function Calling HTTP 测试
//! 
//! 测试我们的Function Calling HTTP服务器与Qwen大模型的集成

use serde_json::{json, Value};
use std::time::Duration;
use tracing::{info, error};
use maa_intelligent_server::MaaAdapterTrait;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    info!("🚀 开始 Qwen Function Calling HTTP 集成测试");
    
    // 1. 创建MAA适配器和Function Calling服务器
    let maa_config = maa_intelligent_server::MaaConfig::default();
    let maa_adapter = std::sync::Arc::new(
        maa_intelligent_server::MaaAdapter::new(maa_config).await?
    );
    let function_server = std::sync::Arc::new(
        maa_intelligent_server::create_function_server(maa_adapter)
    );
    
    // 2. 启动我们的Function Calling服务器（后台任务）
    let server_function_server = function_server.clone();
    tokio::spawn(async move {
        if let Err(e) = maa_intelligent_server::function_calling_server::start_function_calling_server(
            server_function_server, 8080
        ).await {
            error!("服务器启动失败: {}", e);
        }
    });
    
    // 等待服务器启动
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // 2. 获取我们的工具定义
    let client = reqwest::Client::new();
    let tools_response = client
        .get("http://localhost:8080/tools")
        .send()
        .await?;
    
    if !tools_response.status().is_success() {
        error!("无法获取工具定义: {}", tools_response.status());
        return Err("服务器未响应".into());
    }
    
    let tools: Vec<Value> = tools_response.json().await?;
    info!("✅ 获取到 {} 个工具定义", tools.len());
    
    // 打印工具定义
    for tool in &tools {
        if let (Some(name), Some(desc)) = (tool["name"].as_str(), tool["description"].as_str()) {
            info!("  - {}: {}", name, desc);
        }
    }
    
    // 3. 准备Qwen API的工具格式
    let qwen_tools: Vec<Value> = tools.iter().map(|tool| {
        json!({
            "type": "function",
            "function": {
                "name": tool["name"],
                "description": tool["description"],
                "parameters": tool["parameters"]
            }
        })
    }).collect();
    
    // 4. 测试场景
    let test_scenarios = vec![
        ("状态查询", "请帮我查看MAA的状态"),
        ("截图命令", "帮我截个图"),
        ("干员查询", "查询我的干员列表"),
        ("综合任务", "帮我检查状态，然后截图，最后查看干员"),
    ];
    
    for (scenario_name, user_message) in test_scenarios {
        info!("\n🧪 测试场景: {}", scenario_name);
        info!("👤 用户输入: {}", user_message);
        
        // 5. 调用Qwen API
        let qwen_request = json!({
            "model": "qwen-plus-2025-04-28",
            "messages": [
                {
                    "role": "system",
                    "content": "你是MAA智能助手，可以通过Function Calling控制MAA执行明日方舟相关操作。根据用户需求调用合适的工具。"
                },
                {
                    "role": "user", 
                    "content": user_message
                }
            ],
            "tools": qwen_tools,
            "enable_thinking": true,
            "stream": false
        });
        
        let qwen_response = client
            .post("https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions")
            .header("Authorization", &format!("Bearer {}", std::env::var("QWEN_API_KEY").expect("QWEN_API_KEY environment variable not set")))
            .header("Content-Type", "application/json")
            .json(&qwen_request)
            .send()
            .await?;
        
        if !qwen_response.status().is_success() {
            error!("Qwen API调用失败: {}", qwen_response.status());
            let error_text = qwen_response.text().await?;
            error!("错误详情: {}", error_text);
            continue;
        }
        
        let qwen_result: Value = qwen_response.json().await?;
        
        // 6. 处理Qwen响应
        if let Some(choices) = qwen_result["choices"].as_array() {
            if let Some(choice) = choices.first() {
                // 显示思考过程
                if let Some(thinking) = choice["message"]["thinking"].as_str() {
                    info!("🧠 Qwen思考过程:");
                    println!("{}", thinking);
                }
                
                // 处理工具调用
                if let Some(tool_calls) = choice["message"]["tool_calls"].as_array() {
                    info!("🔧 Qwen选择调用 {} 个工具:", tool_calls.len());
                    
                    for tool_call in tool_calls {
                        if let Some(function_name) = tool_call["function"]["name"].as_str() {
                            let arguments = &tool_call["function"]["arguments"];
                            info!("  📞 调用: {} with args: {}", function_name, arguments);
                            
                            // 7. 执行Function Call（调用我们的服务器）
                            let function_request = json!({
                                "function_call": {
                                    "name": function_name,
                                    "arguments": if arguments.is_string() {
                                        serde_json::from_str::<Value>(arguments.as_str().unwrap()).unwrap_or(json!({}))
                                    } else {
                                        arguments.clone()
                                    }
                                }
                            });
                            
                            let function_response = client
                                .post("http://localhost:8080/call")
                                .header("Content-Type", "application/json")
                                .json(&function_request)
                                .send()
                                .await?;
                            
                            if function_response.status().is_success() {
                                let result: Value = function_response.json().await?;
                                info!("  ✅ 执行成功: {}", result);
                            } else {
                                let error_text = function_response.text().await?;
                                error!("  ❌ 执行失败: {}", error_text);
                            }
                        }
                    }
                } else if let Some(content) = choice["message"]["content"].as_str() {
                    info!("🤖 Qwen文本响应: {}", content);
                }
            }
        }
        
        // 等待一下再进行下一个测试
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    
    // 8. 测试直接Function Calling（无AI中介）
    info!("\n🔧 测试直接Function Calling");
    test_direct_function_calling(&client).await?;
    
    info!("\n🎉 所有测试完成!");
    Ok(())
}

/// 测试直接调用我们的Function Calling接口
async fn test_direct_function_calling(client: &reqwest::Client) -> Result<(), Box<dyn std::error::Error>> {
    let test_calls = vec![
        ("maa_status", json!({"verbose": false})),
        ("maa_command", json!({"command": "截图"})),
        ("maa_operators", json!({"query_type": "list"})),
        ("maa_copilot", json!({
            "copilot_config": {
                "stage": "1-7",
                "team": []
            },
            "name": "测试作业"
        })),
    ];
    
    for (function_name, arguments) in test_calls {
        info!("🧪 直接测试: {}", function_name);
        
        let request = json!({
            "function_call": {
                "name": function_name,
                "arguments": arguments
            }
        });
        
        let response = client
            .post("http://localhost:8080/call")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;
        
        if response.status().is_success() {
            let result: Value = response.json().await?;
            info!("  ✅ {}: {}", function_name, result);
        } else {
            let error_text = response.text().await?;
            error!("  ❌ {}: {}", function_name, error_text);
        }
    }
    
    Ok(())
}