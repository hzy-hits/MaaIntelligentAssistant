//! MAA 智能控制中间层 - 主程序入口
//! 
//! 启动 Function Calling HTTP 服务器，为大模型提供MAA控制能力

use std::sync::Arc;
use maa_intelligent_server::{
    MaaAdapter, MaaAdapterTrait, MaaConfig, create_function_server,
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

    info!("MAA智能控制中间层启动");
    info!("架构: 大模型API → Function Calling → MAA适配器 → MAA Core");

    // 1. 初始化MAA适配器
    info!("初始化MAA适配器...");
    let maa_config = MaaConfig::default();
    let maa_adapter = match MaaAdapter::new(maa_config).await {
        Ok(adapter) => {
            info!("MAA适配器初始化成功");
            Arc::new(adapter) as Arc<dyn MaaAdapterTrait + Send + Sync>
        }
        Err(e) => {
            error!("MAA适配器初始化失败: {}", e);
            error!("这可能是因为MAA Core库未正确配置");
            error!("请检查 MAA Core 是否正确安装和配置");
            return Err(e.into());
        }
    };

    // 2. 创建Function Calling服务器
    info!("创建Function Calling服务器...");
    let function_server = Arc::new(create_function_server(maa_adapter));

    // 3. 显示可用工具
    info!("可用的MAA工具:");
    let tools = function_server.get_function_definitions();
    for (i, tool) in tools.iter().enumerate() {
        info!("   {}. {} - {}", i + 1, tool.name, tool.description);
    }

    // 4. 启动HTTP服务器
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);

    info!("启动Function Calling HTTP服务器，端口: {}", port);
    
    println!("\nMAA智能控制中间层启动成功！");
    println!("服务地址: http://localhost:{}", port);
    println!("\n大模型使用方法:");
    println!("1. 获取工具列表: GET  http://localhost:{}/tools", port);
    println!("2. 执行函数调用: POST http://localhost:{}/call", port);
    println!("3. 健康检查:     GET  http://localhost:{}/health", port);
    
    println!("\n调用示例:");
    println!(r#"curl -X POST http://localhost:{}/call \
  -H "Content-Type: application/json" \
  -d '{{
    "function_call": {{
      "name": "maa_command",
      "arguments": {{"command": "帮我做日常"}}
    }}
  }}'"#, port);

    println!("\n支持的大模型格式:");
    println!("   • OpenAI Function Calling");
    println!("   • Claude Tools");
    println!("   • Qwen Function Calling");
    println!("   • Kimi Function Calling");
    println!("   • 任何支持JSON-RPC的AI");

    // 启动服务器
    if let Err(e) = start_function_calling_server(function_server, port).await {
        error!("服务器启动失败: {}", e);
        return Err(e);
    }

    Ok(())
}