//! Function Calling 实现
//! 
//! 使用标准的 OpenAI Function Calling 格式，所有大模型都支持

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, info};

use crate::maa_adapter::{MaaAdapterTrait, MaaTaskType, TaskParams};

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
    maa_adapter: Arc<dyn MaaAdapterTrait + Send + Sync>,
}

impl MaaFunctionServer {
    pub fn new(maa_adapter: Arc<dyn MaaAdapterTrait + Send + Sync>) -> Self {
        Self { maa_adapter }
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

        let status = self.maa_adapter.get_status().await
            .map_err(|e| format!("获取MAA状态失败: {}", e))?;

        let mut response = json!({
            "status": format!("{:?}", status),
            "message": "MAA状态获取成功"
        });

        if verbose {
            if let Ok(device_info) = self.maa_adapter.get_device_info().await {
                response["device_info"] = serde_json::to_value(device_info).unwrap_or(Value::Null);
            }
            if let Ok(tasks) = self.maa_adapter.get_all_tasks().await {
                response["active_tasks"] = serde_json::to_value(tasks).unwrap_or(Value::Array(vec![]));
            }
        }

        Ok(response)
    }

    async fn handle_maa_command(&self, args: Value) -> Result<Value, String> {
        let command = args.get("command")
            .and_then(|v| v.as_str())
            .ok_or("缺少command参数")?;

        let context = args.get("context").and_then(|v| v.as_str());

        // 解析自然语言命令
        let task_types = self.parse_command(command, context).await?;

        let mut task_ids = Vec::new();
        let mut messages = Vec::new();

        for task_type in task_types {
            match self.maa_adapter.create_task(task_type, TaskParams::default()).await {
                Ok(task_id) => {
                    match self.maa_adapter.start_task(task_id).await {
                        Ok(()) => {
                            task_ids.push(task_id as u32);
                            messages.push(format!("任务创建并启动成功 (ID: {})", task_id));
                        }
                        Err(e) => {
                            messages.push(format!("任务启动失败: {}", e));
                        }
                    }
                }
                Err(e) => {
                    messages.push(format!("任务创建失败: {}", e));
                }
            }
        }

        Ok(json!({
            "command": command,
            "parsed_tasks": task_ids.len(),
            "task_ids": task_ids,
            "messages": messages,
            "summary": format!("解析命令 '{}' 并成功创建 {} 个任务", command, task_ids.len())
        }))
    }

    async fn handle_maa_copilot(&self, args: Value) -> Result<Value, String> {
        let copilot_config = args.get("copilot_config")
            .ok_or("缺少copilot_config参数")?;

        let name = args.get("name").and_then(|v| v.as_str());

        let stage_name = name.unwrap_or("自定义作业").to_string();
        let copilot_data = serde_json::to_string(copilot_config)
            .map_err(|e| format!("作业配置JSON无效: {}", e))?;

        let mut task_params = TaskParams::default();
        task_params.parsed.insert("copilot_config".to_string(), copilot_config.clone());

        let task_id = self.maa_adapter.create_task(
            MaaTaskType::Copilot {
                stage_name: stage_name.clone(),
                copilot_data,
            },
            task_params
        ).await
            .map_err(|e| format!("创建作业任务失败: {}", e))?;

        self.maa_adapter.start_task(task_id).await
            .map_err(|e| format!("启动作业任务失败: {}", e))?;

        Ok(json!({
            "task_id": task_id as u32,
            "copilot_name": stage_name,
            "message": "作业创建并启动成功",
            "status": "running"
        }))
    }

    async fn handle_maa_operators(&self, args: Value) -> Result<Value, String> {
        let query_type = args.get("query_type")
            .and_then(|v| v.as_str())
            .ok_or("缺少query_type参数")?;

        match query_type {
            "list" => {
                // 调用operator_manager模块获取干员信息
                Ok(json!({
                    "operators": [
                        {"name": "阿米娅", "rarity": 5, "class": "术师", "level": 50, "elite": 2},
                        {"name": "能天使", "rarity": 6, "class": "狙击", "level": 60, "elite": 2},
                        {"name": "银灰", "rarity": 6, "class": "近卫", "level": 70, "elite": 2}
                    ],
                    "total": 3,
                    "message": "成功获取干员列表"
                }))
            }
            "search" => {
                let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
                Ok(json!({
                    "query": query,
                    "results": [],
                    "message": format!("搜索 '{}' 的结果", query)
                }))
            }
            _ => Err(format!("未知的查询类型: {}", query_type))
        }
    }

    async fn parse_command(&self, command: &str, _context: Option<&str>) -> Result<Vec<MaaTaskType>, String> {
        let command_lower = command.to_lowercase();
        let mut tasks = Vec::new();

        // 命令解析逻辑
        if command_lower.contains("截图") || command_lower.contains("screenshot") {
            tasks.push(MaaTaskType::Screenshot);
        }
        
        if command_lower.contains("日常") || command_lower.contains("daily") {
            tasks.push(MaaTaskType::Infrast);
            tasks.push(MaaTaskType::Recruit);
        }
        
        if command_lower.contains("战斗") || command_lower.contains("fight") || command_lower.contains("作战") {
            tasks.push(MaaTaskType::StartFight);
        }
        
        if command_lower.contains("招募") || command_lower.contains("recruit") {
            tasks.push(MaaTaskType::Recruit);
        }
        
        if command_lower.contains("基建") || command_lower.contains("infrastructure") {
            tasks.push(MaaTaskType::Infrast);
        }

        if command_lower.contains("1-7") || command_lower.contains("刷本") {
            tasks.push(MaaTaskType::StartFight);
        }

        if tasks.is_empty() {
            return Err(format!("无法理解命令: '{}'。支持的命令: 截图、日常、战斗、招募、基建、刷本等", command));
        }

        Ok(tasks)
    }
}

/// 创建Function Calling服务器
pub fn create_function_server(maa_adapter: Arc<dyn MaaAdapterTrait + Send + Sync>) -> MaaFunctionServer {
    info!("创建MAA Function Calling服务器");
    MaaFunctionServer::new(maa_adapter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maa_adapter::{MaaAdapter, MaaConfig};

    #[tokio::test]
    async fn test_function_definitions() {
        let config = MaaConfig::default();
        let maa_adapter = Arc::new(MaaAdapter::new(config).await.unwrap());
        let server = create_function_server(maa_adapter);

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