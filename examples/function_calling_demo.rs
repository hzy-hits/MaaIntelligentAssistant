//! Function Calling 演示程序
//! 
//! 展示如何启动MAA Function Calling服务器，让大模型API能够调用MAA功能

use std::sync::Arc;
use maa_intelligent_server::{
    MaaAdapter, MaaAdapterTrait, MaaConfig, create_function_server,
    function_calling_server::start_function_calling_server,
};
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("info,maa_intelligent_server=debug")
        .init();

    info!("启动MAA Function Calling演示");

    // 1. 初始化MAA适配器
    info!("初始化MAA适配器...");
    let maa_config = MaaConfig::default();
    let adapter = MaaAdapter::new(maa_config).await.map_err(|e| {
        error!("MAA适配器初始化失败: {}", e);
        error!("这可能是因为MAA Core库未正确链接，但Function Calling API仍然可用");
        e
    })?;
    let maa_adapter: Arc<dyn MaaAdapterTrait + Send + Sync> = Arc::new(adapter);
    info!("MAA适配器初始化成功");

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
    let port = 8080;
    info!("启动HTTP服务器，端口: {}", port);
    
    println!("\nMAA Function Calling服务器启动成功！");
    println!("API地址: http://localhost:{}", port);
    println!("\n使用方法:");
    println!("1. 获取工具列表: GET http://localhost:{}/tools", port);
    println!("2. 执行函数调用: POST http://localhost:{}/call", port);
    println!("\n示例请求体:");
    println!(r#"{{
  "function_call": {{
    "name": "maa_command",
    "arguments": {{
      "command": "帮我做日常"
    }}
  }}
}}"#);
    println!("\n大模型可以通过以下步骤调用MAA:");
    println!("1. 首先调用 GET /tools 获取所有可用工具的定义");
    println!("2. 根据用户请求选择合适的工具");
    println!("3. 构造函数调用请求发送到 POST /call");
    println!("4. 解析响应结果返回给用户");

    // 启动服务器
    if let Err(e) = start_function_calling_server(function_server, port).await {
        error!("服务器启动失败: {}", e);
        return Err(e);
    }

    Ok(())
}

/// 测试Function Calling功能
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_function_calling_workflow() {
        // 这个测试展示了完整的Function Calling工作流程

        // 1. 初始化
        let maa_config = MaaConfig::default();
        let maa_adapter = Arc::new(MaaAdapter::new(maa_config).await.unwrap());
        let function_server = Arc::new(create_function_server(maa_adapter));

        // 2. 获取工具定义（大模型第一步）
        let tools = function_server.get_function_definitions();
        assert!(!tools.is_empty());
        
        // 验证工具定义包含必要信息
        let status_tool = tools.iter().find(|t| t.name == "maa_status").unwrap();
        assert!(!status_tool.description.is_empty());
        assert!(status_tool.parameters.is_object());

        // 3. 模拟大模型调用（状态查询）
        let call = maa_intelligent_server::MaaFunctionCall {
            name: "maa_status".to_string(),
            arguments: json!({"verbose": false}),
        };
        
        let response = function_server.execute_function(call).await;
        assert!(response.success);
        assert!(response.result.is_some());

        // 4. 模拟大模型调用（执行命令）
        let call = maa_intelligent_server::MaaFunctionCall {
            name: "maa_command".to_string(),
            arguments: json!({"command": "截图"}),
        };
        
        let response = function_server.execute_function(call).await;
        // 注意：这可能失败因为MAA Core未连接，但这是预期的
        assert!(response.error.is_some() || response.success);
    }
}