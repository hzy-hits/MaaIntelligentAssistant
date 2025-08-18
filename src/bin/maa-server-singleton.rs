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
    FunctionCall
};

/// Function Calling 请求格式
#[derive(Debug, Deserialize)]
struct FunctionCallRequest {
    function_call: FunctionCall,
}

/// 应用状态
#[derive(Clone)]
struct AppState {
    version: String,
    started_at: String,
    enhanced_server: EnhancedMaaFunctionServer,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("🚀 启动 MAA 单例增强服务器");
    info!("📋 支持 16 个完整的 MAA Function Calling 工具");
    info!("🎯 简化架构：HTTP → Enhanced Tools → MaaCore单例");
    
    // 直接创建增强Function Calling服务器，无需中间服务层
    let enhanced_server = EnhancedMaaFunctionServer::new();
    
    info!("✅ MAA 增强服务器创建成功，使用直接单例模式");
    
    // 初始化应用状态
    let app_state = AppState {
        version: "1.0.0-singleton".to_string(),
        started_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        enhanced_server,
    };

    // 构建路由器
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
        .route("/api/health", get(health_handler))
        .route("/tools", get(tools_handler))
        .route("/api/tools", get(tools_handler))
        .route("/call", post(call_handler))
        .route("/api/call", post(call_handler))
        .route("/status", get(status_handler))
        .with_state(app_state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_headers(Any)
                .allow_methods(Any),
        );

    // 启动服务器
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);

    let addr = format!("0.0.0.0:{}", port);
    info!("🌐 服务器监听: http://{}", addr);
    info!("📚 API文档: http://localhost:{}/tools", port);
    info!("💓 健康检查: http://localhost:{}/health", port);
    info!("🎮 MAA 控制: 支持 PlayCover iOS 游戏和 Android 模拟器");

    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// 根路径处理器
async fn root_handler() -> impl IntoResponse {
    Json(json!({
        "name": "MAA 单例增强服务器",
        "version": "1.0.0-singleton",
        "description": "使用单例模式的MAA智能控制服务器，支持16个增强Function Calling工具",
        "endpoints": {
            "health": "/health",
            "tools": "/tools", 
            "call": "/call",
            "status": "/status"
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
                "error": response.error.unwrap_or("Unknown error".to_string()),
                "timestamp": response.timestamp,
                "backend": "singleton"
            }))
        }
    }
}