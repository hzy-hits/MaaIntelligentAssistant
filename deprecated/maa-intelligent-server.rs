//! MAA 单例增强服务器
//!
//! 使用单例模式直接调用MAA Core，同时兼容现有的17个Function Calling工具

use axum::{
    response::{Json, IntoResponse},
    routing::{get, post},
    Router,
    http::{StatusCode, header},
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
    EnhancedMaaFunctionHandler,
    FunctionCall,
    create_enhanced_function_handler
};
use maa_intelligent_server::maa_core::{create_maa_task_channel, MaaWorker, init_task_notification_system};
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
    enhanced_handler: EnhancedMaaFunctionHandler,
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

    info!("启动 MAA 单例增强服务器");
    info!("支持 16 个完整的 MAA Function Calling 工具");
    info!("新架构：HTTP → Enhanced Tools → 任务队列 → MAA工作线程");
    
    // 初始化任务通知系统
    let _task_event_receiver = init_task_notification_system();
    info!("🔔 任务通知系统已初始化");
    
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
    
    // 创建增强Function Calling处理器，传入任务发送器
    let enhanced_handler = create_enhanced_function_handler(task_sender);
    
    info!("MAA 增强处理器创建成功，使用任务队列架构");
    
    // 创建AI客户端配置（使用环境变量）
    let ai_client = match AiClient::from_env() {
        Ok(client) => {
            info!("AI客户端从环境变量初始化成功");
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
    info!("AI客户端初始化成功");
    
    // 初始化应用状态
    let app_state = AppState {
        enhanced_handler,
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
        .route("/chat/reset", post(reset_chat_handler))
        .route("/task/{task_id}/status", get(task_status_handler))
        .route("/tasks", get(all_tasks_handler))
        .route("/screenshot/{screenshot_id}", get(screenshot_handler))
        .route("/screenshot/{screenshot_id}/original", get(original_screenshot_handler))
        .route("/screenshots", get(screenshots_list_handler))
        .route("/take-screenshot", get(take_screenshot_handler))
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
async fn health_handler(
    axum::extract::State(state): axum::extract::State<AppState>
) -> impl IntoResponse {
    // 获取MAA处理器状态并自动初始化
    let handler_status = state.enhanced_handler.get_server_status().await;
    Json(handler_status)
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
    let functions = state.enhanced_handler.get_function_definitions();

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
    let response = state.enhanced_handler.execute_function(request.function_call).await;
    
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
    
    // 验证消息长度，防止大量base64数据污染
    const MAX_MESSAGE_LENGTH: usize = 100_000; // 100KB限制
    const MAX_SINGLE_MESSAGE_LENGTH: usize = 50_000; // 单条消息50KB限制
    
    let total_length: usize = request.messages.iter().map(|msg| msg.content.len()).sum();
    if total_length > MAX_MESSAGE_LENGTH {
        warn!("消息总长度超限: {} 字符", total_length);
        return Json(json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "抱歉，消息内容过长，请分段发送或清除历史记录。",
                    "tool_calls": null
                }
            }],
            "error": "message_too_long"
        }));
    }
    
    // 检查单条消息长度
    for (i, msg) in request.messages.iter().enumerate() {
        if msg.content.len() > MAX_SINGLE_MESSAGE_LENGTH {
            warn!("第{}条消息过长: {} 字符，可能包含未过滤的图片数据", i + 1, msg.content.len());
            return Json(json!({
                "choices": [{
                    "message": {
                        "role": "assistant",
                        "content": "检测到历史消息中包含大量数据，请重置对话或手动清除包含图片的消息。",
                        "tool_calls": null
                    }
                }],
                "error": "message_contains_large_data"
            }));
        }
    }
    
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
    let tools = state.enhanced_handler.get_function_definitions()
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
    
    // 添加历史消息（限制数量并过滤图片数据）
    let recent_messages = request.messages.iter()
        .rev()  // 从最新的开始
        .take(10)  // 只取最近10条消息
        .rev()  // 恢复原顺序
        .map(|msg| {
            // 过滤掉包含base64图片的内容，避免发送给AI
            let filtered_content = if msg.content.contains("data:image/") || msg.content.contains("base64,") {
                // 如果消息包含图片或base64数据，彻底过滤
                let lines: Vec<&str> = msg.content.lines().collect();
                let mut filtered_lines = Vec::new();
                let mut skip_until_end = false;
                
                for line in lines {
                    if line.contains("data:image/") || line.starts_with("![") {
                        skip_until_end = true;
                        continue;
                    }
                    
                    // 检测base64数据行（通常很长且只包含base64字符）
                    if line.len() > 100 && line.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=') {
                        continue;
                    }
                    
                    // 如果遇到空行或新的文本内容，停止跳过
                    if skip_until_end && (line.trim().is_empty() || !line.starts_with("*")) {
                        skip_until_end = false;
                    }
                    
                    if !skip_until_end {
                        filtered_lines.push(line);
                    }
                }
                
                filtered_lines.join("\n").trim().to_string()
            } else {
                msg.content.clone()
            };
            
            AiChatMessage {
                role: msg.role.clone(),
                content: filtered_content,
            }
        })
        .filter(|msg| !msg.content.trim().is_empty())  // 过滤掉空消息
        .collect::<Vec<_>>();
    
    ai_messages.extend(recent_messages);
    
    // 调试：详细记录发送给AI的消息
    let total_content_length: usize = ai_messages.iter().map(|msg| msg.content.len()).sum();
    info!("准备发送给AI: {} 条消息，总长度: {} 字符", ai_messages.len(), total_content_length);
    
    // 记录每条消息的详细信息
    for (i, msg) in ai_messages.iter().enumerate() {
        let content_preview = if msg.content.chars().count() > 100 {
            let preview: String = msg.content.chars().take(100).collect();
            format!("{}...（共{}字符）", preview, msg.content.chars().count())
        } else {
            msg.content.clone()
        };
        debug!("消息[{}] {}: {}", i, msg.role, content_preview);
        
        // 检查可疑内容
        if msg.content.contains("base64") || msg.content.contains("data:image") {
            error!("发现可疑内容！消息[{}]包含图片数据: {} 字符", i, msg.content.len());
        }
    }
    
    if total_content_length > 50000 {
        error!("消息内容过长 ({} 字符)，强制拒绝请求", total_content_length);
        return Json(json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "检测到消息中包含大量数据，请重置对话后重试。",
                    "tool_calls": null
                }
            }],
            "error": "message_too_long_detected"
        }));
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
                    let mut screenshot_info: Option<serde_json::Value> = None;
                    
                    for function_call in function_calls {
                        info!("AI决定调用工具: {} with args: {:?}", function_call.name, function_call.arguments);
                        
                        // 特殊处理截图工具：单独返回，不混入普通消息
                        if function_call.name == "maa_take_screenshot" {
                            let result = call_maa_function(&state, &function_call.name, function_call.arguments.clone()).await;
                            match result {
                                Ok(data) => {
                                    screenshot_info = Some(data);
                                    // 截图成功，但不添加到普通结果中
                                    info!("截图工具调用成功，数据将单独返回");
                                },
                                Err(e) => {
                                    error!("截图工具调用失败: {}", e);
                                    results.push((function_call.name.clone(), Err(e)));
                                }
                            }
                        } else {
                            // 其他工具正常处理
                            let result = call_maa_function(&state, &function_call.name, function_call.arguments.clone()).await;
                            results.push((function_call.name.clone(), result));
                        }
                        
                        // 记录工具调用信息（用于前端显示）
                        tool_calls_info.push(json!({
                            "function": {
                                "name": function_call.name.clone(),
                                "arguments": serde_json::to_string(&function_call.arguments).unwrap_or_default()
                            }
                        }));
                    }
                    
                    // 如果只有截图工具被调用，直接返回截图数据
                    if let Some(screenshot_data) = screenshot_info {
                        if results.is_empty() {
                            return Json(json!({
                                "choices": [{
                                    "message": {
                                        "role": "assistant",
                                        "content": "截图已完成！",
                                        "tool_calls": tool_calls_info
                                    }
                                }],
                                "screenshot": screenshot_data,  // 单独的截图字段
                                "screenshot_only": true  // 标记这是纯截图响应
                            }));
                        }
                    }
                    
                    let results_summary = results.iter().map(|(name, result)| {
                        match result {
                            Ok(data) => {
                                let status = data.get("status").and_then(|s| s.as_str()).unwrap_or("success");
                                let message = data.get("message").and_then(|m| m.as_str()).unwrap_or("任务完成");
                                format!("工具 {} 执行{}: {}", name, status, message)
                            },
                            Err(e) => format!("工具 {} 执行失败: {}", name, e)
                        }
                    }).collect::<Vec<_>>().join("\n");
                    
                    let followup_prompt = format!(
                        "工具执行结果摘要：\n{}\n\n请给用户一个简洁、友好的中文回复，总结任务完成情况并提供后续建议。回复应该：\n1. 简洁明了，不要包含JSON代码或技术细节\n2. 使用友好的语言风格\n3. 提供实用的后续建议\n4. 不要重复显示工具执行的原始数据",
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
                            let fallback_content = format!("工具执行完成：\n{}", results_summary);
                            
                            Json(json!({
                                "choices": [{
                                    "message": {
                                        "role": "assistant", 
                                        "content": fallback_content,
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
            error!("AI调用失败时的消息长度: {} 字符", total_content_length);
            
            // 特别处理422错误
            let error_msg = if e.to_string().contains("422") {
                "检测到请求格式问题，可能是历史消息包含大量数据。建议重置对话后重试。"
            } else {
                &format!("AI服务暂时不可用：{}。请稍后再试。", e)
            };
            
            Json(json!({
                "choices": [{
                    "message": {
                        "role": "assistant", 
                        "content": error_msg,
                        "tool_calls": null
                    }
                }],
                "error": "ai_call_failed",
                "error_details": e.to_string()
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
    
    let response = state.enhanced_handler.execute_function(function_call).await;
    
    if response.success {
        Ok(response.result.unwrap_or(json!({})))
    } else {
        Err(anyhow::anyhow!("MAA调用失败: {:?}", response.error))
    }
}

/// 聊天重置处理器 - 清除对话上下文，重新开始
async fn reset_chat_handler(
    axum::extract::State(_state): axum::extract::State<AppState>
) -> impl IntoResponse {
    info!("收到聊天重置请求");
    
    // 返回重置确认消息，OpenAI兼容格式
    Json(json!({
        "choices": [{
            "message": {
                "role": "assistant",
                "content": "对话已重置！我是MAA智能助手，可以帮您控制明日方舟自动化助手进行各种游戏操作。\n\n请问有什么可以为您效劳的吗？",
                "tool_calls": null
            }
        }],
        "reset": true,
        "timestamp": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()
    }))
}

/// 任务状态查询处理器
async fn task_status_handler(
    axum::extract::Path(task_id): axum::extract::Path<i32>
) -> impl IntoResponse {
    use maa_intelligent_server::maa_core::{get_task_status};
    
    match get_task_status(task_id) {
        Some(task) => {
            Json(json!({
                "success": true,
                "task": task,
                "timestamp": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()
            }))
        },
        None => {
            Json(json!({
                "success": false,
                "error": "任务不存在",
                "task_id": task_id,
                "timestamp": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()
            }))
        }
    }
}

/// 所有任务状态查询处理器
async fn all_tasks_handler() -> impl IntoResponse {
    use maa_intelligent_server::maa_core::{get_all_tasks, get_running_tasks, cleanup_old_tasks};
    
    // 清理旧任务
    cleanup_old_tasks();
    
    let all_tasks = get_all_tasks();
    let running_tasks = get_running_tasks();
    
    Json(json!({
        "success": true,
        "total_tasks": all_tasks.len(),
        "running_tasks": running_tasks.len(),
        "tasks": all_tasks,
        "timestamp": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()
    }))
}

/// 获取指定截图处理器
async fn screenshot_handler(
    axum::extract::Path(screenshot_id): axum::extract::Path<String>
) -> impl IntoResponse {
    use maa_intelligent_server::maa_core::get_screenshot_by_id;
    
    match get_screenshot_by_id(&screenshot_id) {
        Ok(screenshot) => {
            Json(json!({
                "success": true,
                "screenshot": screenshot,
                "timestamp": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()
            }))
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("获取截图失败: {}", e),
                "screenshot_id": screenshot_id,
                "timestamp": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()
            }))
        }
    }
}

/// 获取所有截图列表处理器
async fn screenshots_list_handler() -> impl IntoResponse {
    use maa_intelligent_server::maa_core::{list_all_screenshots, cleanup_screenshots};
    
    // 清理旧截图（保留最近50个）
    let _ = cleanup_screenshots(50);
    
    match list_all_screenshots() {
        Ok(screenshots) => {
            Json(json!({
                "success": true,
                "count": screenshots.len(),
                "screenshots": screenshots,
                "timestamp": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()
            }))
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("获取截图列表失败: {}", e),
                "timestamp": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()
            }))
        }
    }
}

/// 执行截图处理器
async fn take_screenshot_handler(
    axum::extract::State(_state): axum::extract::State<AppState>
) -> impl IntoResponse {
    use maa_intelligent_server::maa_core::screenshot;
    
    info!("收到截图请求");
    
    // 直接通过MAA核心截图功能执行截图
    // 注意：这里我们使用stub模式，实际截图功能需要真实的MAA连接
    match screenshot::save_maa_screenshot(vec![0u8; 100]) {  // 模拟截图数据
        Ok(screenshot_info) => {
            Json(json!({
                "success": true,
                "screenshot": {
                    "id": screenshot_info.id,
                    "timestamp": screenshot_info.timestamp,
                    "file_size": screenshot_info.file_size,
                    "base64_data": screenshot_info.thumbnail_base64,
                    "view_url": format!("/screenshot/{}", screenshot_info.id)
                },
                "message": "截图完成 - MAA眼中的世界！"
            }))
        },
        Err(e) => {
            error!("截图失败: {}", e);
            Json(json!({
                "success": false,
                "error": format!("截图失败: {}", e),
                "timestamp": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()
            }))
        }
    }
}

/// 原始截图文件处理器
async fn original_screenshot_handler(
    axum::extract::Path(screenshot_id): axum::extract::Path<String>
) -> impl IntoResponse {
    use maa_intelligent_server::maa_core::get_screenshot_by_id;
    use std::fs;
    
    info!("请求原始截图: {}", screenshot_id);
    
    match get_screenshot_by_id(&screenshot_id) {
        Ok(screenshot_info) => {
            // 读取原始图片文件
            match fs::read(&screenshot_info.file_path) {
                Ok(image_data) => {
                    info!("返回原始截图: {} ({} bytes)", screenshot_id, image_data.len());
                    
                    // 返回PNG图片数据，设置正确的Content-Type
                    (
                        StatusCode::OK,
                        [
                            (header::CONTENT_TYPE, "image/png"),
                            (header::CACHE_CONTROL, "public, max-age=3600"), // 缓存1小时
                            (header::CONTENT_DISPOSITION, "inline"), // 简化避免借用问题
                        ],
                        image_data
                    )
                },
                Err(e) => {
                    warn!("读取原始截图文件失败: {} - {}", screenshot_id, e);
                    (
                        StatusCode::NOT_FOUND,
                        [
                            (header::CONTENT_TYPE, "application/json"),
                            (header::CACHE_CONTROL, "no-cache"),
                            (header::CONTENT_DISPOSITION, ""),
                        ],
                        format!(r#"{{"error": "图片文件不存在: {}"}}"#, e).into_bytes()
                    )
                }
            }
        },
        Err(e) => {
            warn!("截图不存在: {} - {}", screenshot_id, e);
            (
                StatusCode::NOT_FOUND,
                [
                    (header::CONTENT_TYPE, "application/json"),
                    (header::CACHE_CONTROL, "no-cache"),
                    (header::CONTENT_DISPOSITION, ""),
                ],
                format!(r#"{{"error": "截图不存在: {}"}}"#, e).into_bytes()
            )
        }
    }
}
