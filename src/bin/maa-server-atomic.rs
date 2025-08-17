//! MAA 原子级控制服务器
//! 
//! 提供细颗粒度的MAA底层操作控制，基于FFI接口的原子级Function Calling

use std::sync::Arc;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use maa_intelligent_server::{
    maa_adapter::{MaaBackend, BackendConfig},
    mcp_tools::create_atomic_function_server,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("MAA智能控制中间层启动 - 原子级控制版");
    info!("架构: 大模型API → Atomic Function Calling (11原子操作) → MAA FFI → MAA Core");

    // 初始化MAA后端
    info!("初始化MAA后端...");
    let backend_config = BackendConfig {
        resource_path: "/Users/ivena/Desktop/Fairy/maa/maa-remote-server/maa-official".to_string(),
        prefer_real: true,
        force_stub: false,
        verbose: true,
    };

    let maa_backend = match MaaBackend::new(backend_config) {
        Ok(backend) => {
            info!("MAA后端初始化成功，模式: {}", backend.backend_type());
            Arc::new(backend)
        }
        Err(e) => {
            error!("MAA后端初始化失败: {}", e);
            return Err(anyhow::anyhow!("MAA后端初始化失败: {}", e));
        }
    };

    // 创建原子级Function Calling服务器
    info!("创建原子级Function Calling服务器...");
    let atomic_server = create_atomic_function_server(maa_backend.clone());
    
    info!("原子级MAA工具集 (14种操作: 11个原子级 + 3个自定义任务):");
    info!("=== 设备控制操作 (3个) ===");
    info!("   1. maa_click - 精确像素级点击操作");
    info!("   2. maa_screenshot - 屏幕截图获取，支持多种格式和ROI");
    info!("   3. maa_swipe - 滑动手势控制，支持自定义路径和速度");
    
    info!("=== 连接管理操作 (1个) ===");
    info!("   4. maa_connection - 设备连接管理，支持连接、断开、状态查询");
    
    info!("=== 任务管理操作 (1个) ===");
    info!("   5. maa_task_management - 底层任务管理，支持创建、执行、监控、控制");
    
    info!("=== 图像识别操作 (1个) ===");
    info!("   6. maa_image_recognition - 实时图像识别，支持模板匹配、OCR、特征点匹配");
    
    info!("=== 系统监控操作 (5个) ===");
    info!("   7. maa_device_info - 设备信息查询，包括分辨率、系统版本、能力");
    info!("   8. maa_system_monitor - 系统性能监控，CPU、内存、任务队列状态");
    info!("   9. maa_log_management - 日志系统管理，支持级别设置、查询、过滤");
    info!("   10. maa_callback_events - 回调事件管理，实时状态更新和事件订阅");
    info!("   11. maa_text_input - 文本输入操作，支持多种输入方法");
    
    info!("=== maa-cli 自定义任务 (3个) ===");
    info!("   12. maa_custom_task - 复杂任务组合，支持条件判断和任务变体");
    info!("   13. maa_single_step - 单步原子操作的简化接口");
    info!("   14. maa_video_recognition - 视频文件识别和分析");

    // 启动HTTP服务器，使用自定义处理器
    info!("启动原子级Function Calling HTTP服务器，端口: 8081");
    
    println!("\nMAA智能控制中间层启动成功！（原子级控制版 - 14种操作）");
    println!("服务地址: http://localhost:8081");
    println!("\n原子级控制特性:");
    println!("• 直接访问MAA Core FFI接口");
    println!("• 像素级精确设备控制");
    println!("• 实时图像识别和处理");
    println!("• 细颗粒度任务管理");
    println!("• 完整的系统监控和日志");
    println!("\n大模型使用方法:");
    println!("1. 获取工具列表: GET  http://localhost:8081/tools");
    println!("2. 执行函数调用: POST http://localhost:8081/call");
    println!("3. 健康检查:     GET  http://localhost:8081/health");
    
    println!("\n原子级操作调用示例:");
    println!("# 精确点击");
    println!("curl -X POST http://localhost:8081/call \\");
    println!("  -H \"Content-Type: application/json\" \\");
    println!("  -d '{{");
    println!("    \"function_call\": {{");
    println!("      \"name\": \"maa_click\",");
    println!("      \"arguments\": {{");
    println!("        \"x\": 960,");
    println!("        \"y\": 540,");
    println!("        \"wait_completion\": true");
    println!("      }}");
    println!("    }}");
    println!("  }}'");
    
    println!("\n# 屏幕截图");
    println!("curl -X POST http://localhost:8081/call \\");
    println!("  -H \"Content-Type: application/json\" \\");
    println!("  -d '{{");
    println!("    \"function_call\": {{");
    println!("      \"name\": \"maa_screenshot\",");
    println!("      \"arguments\": {{");
    println!("        \"format\": \"png\",");
    println!("        \"roi\": [100, 100, 800, 600]");
    println!("      }}");
    println!("    }}");
    println!("  }}'");
    
    println!("\n# 图像识别");
    println!("curl -X POST http://localhost:8081/call \\");
    println!("  -H \"Content-Type: application/json\" \\");
    println!("  -d '{{");
    println!("    \"function_call\": {{");
    println!("      \"name\": \"maa_image_recognition\",");
    println!("      \"arguments\": {{");
    println!("        \"algorithm\": \"MatchTemplate\",");
    println!("        \"template\": \"开始行动\",");
    println!("        \"threshold\": 0.8");
    println!("      }}");
    println!("    }}");
    println!("  }}'");

    println!("\n支持的大模型格式:");
    println!("   • OpenAI Function Calling");
    println!("   • Claude Tools");
    println!("   • Qwen Function Calling");
    println!("   • 任何支持JSON-RPC的AI");

    // 启动服务器，端口8081，避免与增强服务器冲突
    if let Err(e) = maa_intelligent_server::function_calling_server::start_function_calling_server(
        Arc::new(atomic_server),
        8081
    ).await {
        return Err(anyhow::anyhow!("启动服务器失败: {}", e));
    }
    
    Ok(())
}