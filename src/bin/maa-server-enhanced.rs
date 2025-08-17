//! MAA 智能控制中间层 - 增强版主程序入口
//! 
//! 启动增强版 Function Calling HTTP 服务器，支持完整的16种MAA任务类型

use std::sync::Arc;
use maa_intelligent_server::{
    MaaBackend, BackendConfig, create_enhanced_function_server,
    EnhancedMaaFunctionServer, FunctionDefinition, 
    MaaFunctionCall, FunctionResponse,
    function_calling_server::start_function_calling_server,
};
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载环境变量
    dotenvy::dotenv().ok();
    
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("info,maa_intelligent_server=debug")
        .init();

    info!("MAA智能控制中间层启动 - 增强版");
    info!("架构: 大模型API → Enhanced Function Calling (16任务) → MAA适配器 → MAA Core");

    // 1. 初始化MAA后端
    info!("初始化MAA后端...");
    let backend_config = BackendConfig::default();
    let maa_backend = match MaaBackend::new(backend_config) {
        Ok(backend) => {
            info!("MAA后端初始化成功，模式: {}", backend.backend_type());
            Arc::new(backend)
        }
        Err(e) => {
            error!("MAA后端初始化失败: {}", e);
            error!("这可能是因为MAA Core库未正确配置，将使用stub模式");
            // 强制使用stub模式作为fallback
            let fallback_config = BackendConfig {
                force_stub: true,
                ..BackendConfig::default()
            };
            let fallback_backend = MaaBackend::new(fallback_config)?;
            info!("使用fallback stub模式");
            Arc::new(fallback_backend)
        }
    };

    // 2. 创建增强Function Calling服务器
    info!("创建增强Function Calling服务器...");
    let enhanced_server = Arc::new(create_enhanced_function_server(maa_backend));

    // 3. 显示可用的16种MAA任务工具
    info!("增强版MAA工具集 (16种任务类型):");
    let tools = enhanced_server.get_function_definitions();
    
    info!("=== 核心游戏功能 (4个) ===");
    for (i, tool) in tools.iter().take(4).enumerate() {
        info!("   {}. {} - {}", i + 1, tool.name, tool.description);
    }
    
    info!("=== 高级自动化功能 (4个) ===");
    for (i, tool) in tools.iter().skip(4).take(4).enumerate() {
        info!("   {}. {} - {}", i + 5, tool.name, tool.description);
    }
    
    info!("=== 辅助功能 (4个) ===");
    for (i, tool) in tools.iter().skip(8).take(4).enumerate() {
        info!("   {}. {} - {}", i + 9, tool.name, tool.description);
    }
    
    info!("=== 系统功能 (4个) ===");
    for (i, tool) in tools.iter().skip(12).take(4).enumerate() {
        info!("   {}. {} - {}", i + 13, tool.name, tool.description);
    }

    // 4. 启动HTTP服务器
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);

    info!("启动增强Function Calling HTTP服务器，端口: {}", port);
    
    println!("\nMAA智能控制中间层启动成功！（增强版 - 16种任务类型）");
    println!("服务地址: http://localhost:{}", port);
    println!("\n增强功能特性:");
    println!("• 完整的16种MAA任务类型支持");
    println!("• 智能参数解析和自然语言理解");
    println!("• 复杂策略配置和自动化流程");
    println!("• 高级错误处理和恢复机制");
    
    println!("\n大模型使用方法:");
    println!("1. 获取工具列表: GET  http://localhost:{}/tools", port);
    println!("2. 执行函数调用: POST http://localhost:{}/call", port);
    println!("3. 健康检查:     GET  http://localhost:{}/health", port);
    
    println!("\n增强工具调用示例:");
    println!(r#"# 启动游戏
curl -X POST http://localhost:{}/call \
  -H "Content-Type: application/json" \
  -d '{{
    "function_call": {{
      "name": "maa_startup",
      "arguments": {{
        "action": "start_game",
        "client_type": "Official"
      }}
    }}
  }}'"#, port);

    println!(r#"
# 增强战斗任务
curl -X POST http://localhost:{}/call \
  -H "Content-Type: application/json" \
  -d '{{
    "function_call": {{
      "name": "maa_combat_enhanced",
      "arguments": {{
        "stage": "1-7",
        "strategy": {{
          "mode": "times",
          "target_value": 10
        }},
        "resources": {{
          "use_medicine": true,
          "medicine_limit": 5
        }}
      }}
    }}
  }}'"#, port);

    println!("\n支持的大模型格式:");
    println!("   • OpenAI Function Calling");
    println!("   • Claude Tools");
    println!("   • Qwen Function Calling");
    println!("   • Kimi Function Calling");
    println!("   • 任何支持JSON-RPC的AI");

    // 启动服务器 - 直接使用增强服务器
    if let Err(e) = start_function_calling_server(enhanced_server, port).await {
        error!("服务器启动失败: {}", e);
        return Err(e);
    }

    Ok(())
}