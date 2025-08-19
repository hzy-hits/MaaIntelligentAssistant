//! MAA 单例增强服务器
//!
//! 使用单例模式直接调用MAA Core，同时兼容现有的16个Function Calling工具

use axum::{
    response::{Json, IntoResponse},
    routing::{get, post},
    Router,
};
use serde::{Deserialize};
use serde_json::{json};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, error, debug, Level};
use tracing_subscriber;
use anyhow::Result;

// 导入我们的模块
use maa_intelligent_server::function_tools::{
    EnhancedMaaFunctionServer,
    FunctionCall,
    create_enhanced_function_server
};
use maa_intelligent_server::maa_core::{create_maa_task_channel, MaaWorker};
use maa_intelligent_server::config::{CONFIG};

/// Function Calling 请求格式
#[derive(Debug, Deserialize)]
struct FunctionCallRequest {
    function_call: FunctionCall,
}

/// 应用状态
#[derive(Clone)]
struct AppState {
    enhanced_server: EnhancedMaaFunctionServer,
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
    
    // 初始化应用状态
    let app_state = AppState {
        enhanced_server,
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