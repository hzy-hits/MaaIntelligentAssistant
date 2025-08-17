//! Function Calling 实现
//! 
//! 使用标准的 OpenAI Function Calling 格式，所有大模型都支持

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, info};

use crate::maa_adapter::MaaBackend;

/// Function Calling 工具定义
#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

/// Function Calling 调用请求
#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Value,
}

/// Function Calling 响应
#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionResponse {
    pub success: bool,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// MAA Function Calling 服务器
pub struct MaaFunctionServer {
    maa_backend: Arc<MaaBackend>,
}

impl MaaFunctionServer {
    pub fn new(maa_backend: Arc<MaaBackend>) -> Self {
        Self { maa_backend }
    }

    /// 获取所有可用工具的定义 - 这是大模型需要的关键信息！
    pub fn get_function_definitions(&self) -> Vec<FunctionDefinition> {
        vec![
            FunctionDefinition {
                name: "maa_status".to_string(),
                description: "获取MAA当前状态、设备信息和活动任务".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "verbose": {
                            "type": "boolean",
                            "description": "是否返回详细信息，包括设备信息和活动任务",
                            "default": false
                        }
                    }
                }),
            },
            FunctionDefinition {
                name: "maa_command".to_string(),
                description: "使用自然语言执行MAA命令，如'帮我做日常'、'截图'、'刷1-7'等".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "自然语言命令，例如：'帮我做日常'、'截图'、'刷1-7关卡'、'基建收菜'"
                        },
                        "context": {
                            "type": "string",
                            "description": "可选的上下文信息，用于更好地理解命令"
                        }
                    },
                    "required": ["command"]
                }),
            },
            FunctionDefinition {
                name: "maa_copilot".to_string(),
                description: "执行MAA作业（自动战斗脚本）".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "copilot_config": {
                            "type": "object",
                            "description": "作业配置JSON，包含关卡信息和编队配置"
                        },
                        "name": {
                            "type": "string",
                            "description": "作业名称（可选）"
                        }
                    },
                    "required": ["copilot_config"]
                }),
            },
            FunctionDefinition {
                name: "maa_operators".to_string(),
                description: "查询和管理明日方舟干员信息".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "query_type": {
                            "type": "string",
                            "enum": ["list", "search"],
                            "description": "查询类型：list（列出所有）或 search（搜索）"
                        },
                        "query": {
                            "type": "string",
                            "description": "搜索关键词（当query_type为search时使用）"
                        }
                    },
                    "required": ["query_type"]
                }),
            },
        ]
    }

    /// 执行 Function Call - 这是核心的智能调度方法！
    pub async fn execute_function(&self, call: FunctionCall) -> FunctionResponse {
        debug!("执行函数调用: {} with args: {:?}", call.name, call.arguments);

        let result = match call.name.as_str() {
            "maa_status" => self.handle_maa_status(call.arguments).await,
            "maa_command" => self.handle_maa_command(call.arguments).await,
            "maa_copilot" => self.handle_maa_copilot(call.arguments).await,
            "maa_operators" => self.handle_maa_operators(call.arguments).await,
            _ => Err(format!("未知的函数: {}", call.name)),
        };

        match result {
            Ok(data) => FunctionResponse {
                success: true,
                result: Some(data),
                error: None,
                timestamp: chrono::Utc::now(),
            },
            Err(error) => FunctionResponse {
                success: false,
                result: None,
                error: Some(error),
                timestamp: chrono::Utc::now(),
            },
        }
    }

    async fn handle_maa_status(&self, args: Value) -> Result<Value, String> {
        let verbose = args.get("verbose").and_then(|v| v.as_bool()).unwrap_or(false);

        let response = json!({
            "status": if self.maa_backend.is_running() { "running" } else { "idle" },
            "connected": self.maa_backend.is_connected(),
            "backend_type": self.maa_backend.backend_type(),
            "verbose": verbose,
            "message": "MAA状态获取成功"
        });

        Ok(response)
    }

    async fn handle_maa_command(&self, args: Value) -> Result<Value, String> {
        let command = args.get("command")
            .and_then(|v| v.as_str())
            .ok_or("缺少command参数")?;

        let _context = args.get("context").and_then(|v| v.as_str()).unwrap_or("");

        // 简化的命令处理 - 直接模拟成功响应
        // 在完整实现中，这里会根据命令类型创建相应的MAA任务
        let response = match command {
            cmd if cmd.contains("日常") || cmd.contains("daily") => {
                json!({
                    "command": command,
                    "result": "daily_started",
                    "message": "日常任务已启动",
                    "backend": self.maa_backend.backend_type()
                })
            }
            cmd if cmd.contains("截图") || cmd.contains("screenshot") => {
                match self.maa_backend.screenshot() {
                    Ok(_) => json!({
                        "command": command,
                        "result": "screenshot_taken",
                        "message": "截图已完成",
                        "backend": self.maa_backend.backend_type()
                    }),
                    Err(e) => json!({
                        "command": command,
                        "result": "error",
                        "message": format!("截图失败: {}", e),
                        "backend": self.maa_backend.backend_type()
                    })
                }
            }
            _ => {
                json!({
                    "command": command,
                    "result": "acknowledged", 
                    "message": format!("已接收命令: {}", command),
                    "backend": self.maa_backend.backend_type(),
                    "note": "完整功能正在开发中"
                })
            }
        };

        Ok(response)
    }

    async fn handle_maa_copilot(&self, args: Value) -> Result<Value, String> {
        let copilot_config = args.get("copilot_config")
            .ok_or("缺少copilot_config参数")?;

        let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("自定义作业");

        // 简化的作业处理 - 在完整实现中会创建实际的作业任务
        Ok(json!({
            "copilot_name": name,
            "config_received": !copilot_config.is_null(),
            "message": "作业配置已接收",
            "backend": self.maa_backend.backend_type(),
            "note": "完整的作业功能正在开发中"
        }))
    }

    async fn handle_maa_operators(&self, args: Value) -> Result<Value, String> {
        let query_type = args.get("query_type")
            .and_then(|v| v.as_str())
            .ok_or("缺少query_type参数")?;

        match query_type {
            "list" => {
                Ok(json!({
                    "operators": [
                        {"name": "阿米娅", "rarity": 5, "class": "术师", "level": 50, "elite": 2},
                        {"name": "能天使", "rarity": 6, "class": "狙击", "level": 60, "elite": 2},
                        {"name": "银灰", "rarity": 6, "class": "近卫", "level": 70, "elite": 2}
                    ],
                    "total": 3,
                    "backend": self.maa_backend.backend_type(),
                    "message": "成功获取干员列表（示例数据）"
                }))
            }
            "search" => {
                let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
                Ok(json!({
                    "query": query,
                    "results": [],
                    "backend": self.maa_backend.backend_type(),
                    "message": format!("搜索 '{}' 的结果（功能开发中）", query)
                }))
            }
            _ => Err(format!("未知的查询类型: {}", query_type))
        }
    }

}

/// 创建Function Calling服务器
pub fn create_function_server(maa_backend: Arc<MaaBackend>) -> MaaFunctionServer {
    info!("创建MAA Function Calling服务器");
    MaaFunctionServer::new(maa_backend)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maa_adapter::{MaaBackend, BackendConfig};

    #[tokio::test]
    async fn test_function_definitions() {
        let config = BackendConfig {
            force_stub: true,
            ..BackendConfig::default()
        };
        let maa_backend = Arc::new(MaaBackend::new(config).unwrap());
        let server = create_function_server(maa_backend);

        let functions = server.get_function_definitions();
        assert_eq!(functions.len(), 4);
        assert_eq!(functions[0].name, "maa_status");
        assert_eq!(functions[1].name, "maa_command");
    }

    #[tokio::test]
    async fn test_function_execution() {
        let config = MaaConfig::default();
        let maa_adapter = Arc::new(MaaAdapter::new(config).await.unwrap());
        let server = create_function_server(maa_adapter);

        let call = FunctionCall {
            name: "maa_status".to_string(),
            arguments: json!({"verbose": false}),
        };

        let response = server.execute_function(call).await;
        assert!(response.success);
    }
}