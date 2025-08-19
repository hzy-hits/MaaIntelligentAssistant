//! MAA 单例增强服务器
//!
//! 使用单例模式直接调用MAA Core，同时兼容现有的16个Function Calling工具

use axum::{
    response::{Json, IntoResponse},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, error, debug, warn, Level};
use tracing_subscriber;
use anyhow::Result;
use std::sync::Arc;

// 导入我们的模块
use maa_intelligent_server::function_tools::{
    EnhancedMaaFunctionServer,
    FunctionCall,
    create_enhanced_function_server
};
use maa_intelligent_server::maa_core::{create_maa_task_channel, MaaWorker};
use maa_intelligent_server::config::{CONFIG};
use maa_intelligent_server::ai_client::{AiClient, AiClientTrait, AiClientConfig, AiProvider, ProviderConfig, ChatMessage as AiChatMessage, Tool};
use maa_intelligent_server::ai_client::client::Either;

/// Function Calling 请求格式
#[derive(Debug, Deserialize)]
struct FunctionCallRequest {
    function_call: FunctionCall,
}

/// 聊天请求格式
#[derive(Debug, Deserialize)]
struct ChatRequest {
    messages: Vec<ChatMessage>,
    // tools 和 system_prompt 字段保留但未使用，用于前端兼容性
    #[allow(dead_code)]
    tools: Option<Vec<serde_json::Value>>,
    #[allow(dead_code)] 
    system_prompt: Option<String>,
}

/// 聊天消息格式
#[derive(Debug, Deserialize, Serialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

// ChatResponse REMOVED - 已改为OpenAI兼容格式，直接使用JSON响应

/// 应用状态
#[derive(Clone)]
struct AppState {
    enhanced_server: EnhancedMaaFunctionServer,
    ai_client: Arc<AiClient>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 设置panic hook来捕获崩溃信息
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("PANIC发生!");
        eprintln!("Panic信息: {}", panic_info);
        eprintln!("位置: {:?}", panic_info.location());
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            eprintln!("Panic消息: {}", s);
        }
        eprintln!("服务器将退出");
    }));
    
    // 加载 .env 配置文件
    if let Err(e) = dotenvy::dotenv() {
        eprintln!("Warning: 无法加载 .env 文件: {}", e);
    }
    
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("🚀 启动 MAA 单例增强服务器");
    info!("📋 支持 16 个完整的 MAA Function Calling 工具");
    info!("🎯 新架构：HTTP → Enhanced Tools → 任务队列 → MAA工作线程");
    
    // 创建MAA任务队列
    let (task_sender, task_receiver) = create_maa_task_channel();
    
    // 启动MAA工作线程（在单独线程中运行，避免Send问题）
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("无法创建tokio runtime");
        
        let local_set = tokio::task::LocalSet::new();
        let maa_worker = MaaWorker::new();
        
        local_set.spawn_local(async move {
            maa_worker.run(task_receiver).await;
        });
        
        rt.block_on(local_set);
    });
    
    info!("🔄 MAA工作线程已启动");
    
    // 创建增强Function Calling服务器，传入任务发送器
    let enhanced_server = create_enhanced_function_server(task_sender);
    
    info!("✅ MAA 增强服务器创建成功，使用任务队列架构");
    
    // 创建AI客户端配置（使用环境变量）
    let ai_client = match AiClient::from_env() {
        Ok(client) => {
            info!("🤖 AI客户端从环境变量初始化成功");
            client
        },
        Err(e) => {
            warn!("AI客户端环境变量初始化失败，使用默认配置: {}", e);
            // 降级到默认配置
            let provider_config = ProviderConfig::new("qwen-plus")
                .with_api_key(std::env::var("AI_API_KEY").unwrap_or("dummy-key".to_string()));
            let ai_config = AiClientConfig::new(AiProvider::Qwen)
                .add_provider(AiProvider::Qwen, provider_config);
            AiClient::new(ai_config).map_err(|e| anyhow::anyhow!("AI客户端初始化失败: {}", e))?
        }
    };
    info!("🤖 AI客户端初始化成功");
    
    // 初始化应用状态
    let app_state = AppState {
        enhanced_server,
        ai_client: Arc::new(ai_client),
    };

    // 构建路由器
    let app = Router::new()
        .route("/", get(root_handler))
        .route(&CONFIG.server.health_check_path, get(health_handler))
        .route("/api/health", get(health_handler))
        .route(&CONFIG.server.tools_path, get(tools_handler))
        .route("/api/tools", get(tools_handler))
        .route(&CONFIG.server.call_path, post(call_handler))
        .route("/api/call", post(call_handler))
        .route(&CONFIG.server.status_path, get(status_handler))
        .route("/chat", post(chat_handler))
        .with_state(app_state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_headers(Any)
                .allow_methods(Any),
        );

    // 启动服务器
    let port = std::env::var(&CONFIG.env_keys.server_port)
        .unwrap_or_else(|_| CONFIG.server.default_port.clone())
        .parse::<u16>()
        .unwrap_or_else(|_| CONFIG.server.default_port.parse().unwrap_or(8080));

    let addr = CONFIG.server.bind_address(Some(&port.to_string()));
    info!("服务器监听: http://{}", addr);
    info!("API文档: http://localhost:{}{}", port, CONFIG.server.tools_path);
    info!("健康检查: http://localhost:{}{}", port, CONFIG.server.health_check_path);
    info!("MAA 控制: 支持 PlayCover iOS 游戏和 Android 模拟器");

    let listener = TcpListener::bind(&addr).await?;
    
    info!("服务器启动完成，开始处理请求...");
    
    // 包装服务器运行以捕获错误
    match axum::serve(listener, app).await {
        Ok(_) => {
            info!("服务器正常关闭");
            Ok(())
        },
        Err(e) => {
            error!("服务器运行错误: {}", e);
            Err(e.into())
        }
    }
}

/// 根路径处理器
async fn root_handler() -> impl IntoResponse {
    Json(json!({
        "name": "MAA 单例增强服务器",
        "version": "1.0.0-singleton",
        "description": "使用单例模式的MAA智能控制服务器，支持16个增强Function Calling工具",
        "endpoints": {
            "health": &CONFIG.server.health_check_path,
            "tools": &CONFIG.server.tools_path, 
            "call": &CONFIG.server.call_path,
            "status": &CONFIG.server.status_path
        },
        "features": {
            "singleton_mode": true,
            "enhanced_tools": 16,
            "real_maa_integration": true,
            "supported_devices": ["PlayCover iOS", "Android Emulators", "Real Devices"]
        }
    }))
}

/// 健康检查处理器
async fn health_handler() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "server": "maa-server-singleton",
        "version": "1.0.0-singleton",
        "timestamp": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        "maa_core": "singleton_ready",
        "backend_type": "singleton"
    }))
}

/// 状态处理器
async fn status_handler() -> impl IntoResponse {
    use maa_intelligent_server::maa_core::get_maa_status;
    
    match get_maa_status().await {
        Ok(status) => Json(json!({
            "server_status": "running",
            "maa_status": status,
            "backend_mode": "singleton",
            "timestamp": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()
        })),
        Err(e) => {
            error!("获取MAA状态失败: {}", e);
            Json(json!({
                "server_status": "running", 
                "maa_status": "error",
                "backend_mode": "singleton",
                "error": e.to_string(),
                "timestamp": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()
            }))
        }
    }
}

/// 工具列表处理器
async fn tools_handler(
    axum::extract::State(state): axum::extract::State<AppState>
) -> impl IntoResponse {
    let functions = state.enhanced_server.get_function_definitions();

    Json(json!({
        "functions": functions,
        "total_count": functions.len(),
        "server": "maa-server-singleton",
        "description": "MAA增强Function Calling工具集 - 单例模式",
        "backend_type": "singleton",
        "categories": {
            "core_game": ["maa_startup", "maa_combat_enhanced", "maa_recruit_enhanced", "maa_infrastructure_enhanced"],
            "advanced_automation": ["maa_roguelike_enhanced", "maa_copilot_enhanced", "maa_sss_copilot", "maa_reclamation"],
            "auxiliary": ["maa_rewards_enhanced", "maa_credit_store_enhanced", "maa_depot_management", "maa_operator_box"],
            "system": ["maa_closedown", "maa_custom_task", "maa_video_recognition", "maa_system_management"]
        }
    }))
}

/// Function Calling 处理器
async fn call_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(request): Json<FunctionCallRequest>
) -> impl IntoResponse {
    debug!("收到增强Function Call: {} with args: {}", request.function_call.name, request.function_call.arguments);
    
    // 使用增强服务器处理Function Call
    let response = state.enhanced_server.execute_function(request.function_call).await;
    
    match response.success {
        true => {
            debug!("增强Function call成功");
            Json(json!({
                "success": true,
                "result": response.result.unwrap_or(json!({})),
                "timestamp": response.timestamp,
                "backend": "singleton"
            }))
        }
        false => {
            error!("增强Function call失败: {:?}", response.error);
            Json(json!({
                "success": false,
                "error": response.error.map(|e| e.message).unwrap_or("Unknown error".to_string()),
                "timestamp": response.timestamp,
                "backend": "singleton"
            }))
        }
    }
}

/// 聊天处理器 - 标准Function Calling流程
async fn chat_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(request): Json<ChatRequest>
) -> impl IntoResponse {
    debug!("收到聊天请求: {} 条消息", request.messages.len());
    
    // 获取最后一条用户消息
    let user_message = request.messages.iter()
        .filter(|msg| msg.role == "user")
        .last()
        .map(|msg| msg.content.clone())
        .unwrap_or_else(|| "你好".to_string());
    
    info!("处理用户消息: {}", user_message);
    
    // 从文件加载系统提示词
    let system_prompt = match tokio::fs::read_to_string("docs/MAA_SYSTEM_PROMPT.md").await {
        Ok(content) => content,
        Err(_) => "你是MAA（明日方舟自动化助手）的智能控制助手。你能理解用户的自然语言请求，智能地调用MAA功能工具来执行游戏自动化任务。".to_string()
    };

    // 获取所有可用的MAA工具定义
    let tools = state.enhanced_server.get_function_definitions()
        .into_iter()
        .map(|def| Tool {
            name: def.name,
            description: def.description,
            parameters: def.parameters,
        })
        .collect::<Vec<_>>();

    // 转换消息格式
    let mut ai_messages = vec![
        AiChatMessage::system(system_prompt),
    ];
    
    // 添加历史消息
    for msg in request.messages {
        ai_messages.push(AiChatMessage {
            role: msg.role,
            content: msg.content,
        });
    }
    
    // 第一步：AI分析并可能调用工具
    match state.ai_client.chat_completion_with_tools(ai_messages, tools).await {
        Ok(result) => {
            // Either已经在顶部导入
            
            match result {
                Either::Left(text_response) => {
                    // 纯文本响应，无工具调用
                    Json(json!({
                        "choices": [{
                            "message": {
                                "role": "assistant",
                                "content": text_response,
                                "tool_calls": null
                            }
                        }]
                    }))
                },
                Either::Right(function_calls) => {
                    // AI决定调用工具
                    let mut results = Vec::new();
                    let mut tool_calls_info = Vec::new();
                    
                    for function_call in function_calls {
                        info!("AI决定调用工具: {} with args: {:?}", function_call.name, function_call.arguments);
                        
                        // 执行MAA工具
                        let result = call_maa_function(&state, &function_call.name, function_call.arguments.clone()).await;
                        
                        // 记录工具调用信息（用于前端显示）
                        tool_calls_info.push(json!({
                            "function": {
                                "name": function_call.name.clone(),
                                "arguments": serde_json::to_string(&function_call.arguments).unwrap_or_default()
                            }
                        }));
                        
                        results.push((function_call.name.clone(), result));
                    }
                    
                    // 第二步：将工具执行结果反馈给AI，让AI生成用户友好的回复
                    let results_summary = results.iter().map(|(name, result)| {
                        match result {
                            Ok(data) => format!("✅ {} 执行成功: {:?}", name, data),
                            Err(e) => format!("❌ {} 执行失败: {}", name, e)
                        }
                    }).collect::<Vec<_>>().join("\n");
                    
                    let followup_prompt = format!(
                        "以下是MAA工具的执行结果：\n{}\n\n请根据这些结果，给用户一个友好、专业的中文回复，总结执行情况并提供后续建议。",
                        results_summary
                    );
                    
                    let final_messages = vec![
                        AiChatMessage::system("你是MAA智能助手，需要根据工具执行结果给用户友好的反馈。".to_string()),
                        AiChatMessage::user(followup_prompt),
                    ];
                    
                    match state.ai_client.chat_completion(final_messages).await {
                        Ok(final_response) => {
                            Json(json!({
                                "choices": [{
                                    "message": {
                                        "role": "assistant",
                                        "content": final_response,
                                        "tool_calls": tool_calls_info
                                    }
                                }]
                            }))
                        },
                        Err(e) => {
                            error!("AI最终响应生成失败: {}", e);
                            // 降级到简单回复
                            Json(json!({
                                "choices": [{
                                    "message": {
                                        "role": "assistant", 
                                        "content": format!("工具执行完成：\n{}", results_summary),
                                        "tool_calls": tool_calls_info
                                    }
                                }]
                            }))
                        }
                    }
                }
            }
        },
        Err(e) => {
            error!("AI调用失败: {}", e);
            Json(json!({
                "choices": [{
                    "message": {
                        "role": "assistant",
                        "content": format!("抱歉，AI服务暂时不可用：{}。请稍后再试。", e),
                        "tool_calls": null
                    }
                }]
            }))
        }
    }
}

/// 辅助函数：调用MAA功能
async fn call_maa_function(
    state: &AppState, 
    function_name: &str, 
    arguments: serde_json::Value
) -> Result<serde_json::Value> {
    use maa_intelligent_server::function_tools::FunctionCall;
    
    let function_call = FunctionCall {
        name: function_name.to_string(),
        arguments: arguments,
    };
    
    let response = state.enhanced_server.execute_function(function_call).await;
    
    if response.success {
        Ok(response.result.unwrap_or(json!({})))
    } else {
        Err(anyhow::anyhow!("MAA调用失败: {:?}", response.error))
    }
}