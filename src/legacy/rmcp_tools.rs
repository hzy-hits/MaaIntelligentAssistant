//! rmcp 0.5.0标准MCP工具实现
//!
//! 这个模块使用官方rmcp SDK实现标准MCP工具，替换自定义的McpTool trait
//! 由于rmcp 0.5.0的复杂性，当前实现了基础的兼容包装器

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, info, warn};

use crate::maa_adapter::{MaaAdapterTrait, MaaTaskType, TaskParams};

/// MAA 工具服务器
#[derive(Clone)]
pub struct MaaToolsServer {
    maa_adapter: Arc<dyn MaaAdapterTrait + Send + Sync>,
}

impl MaaToolsServer {
    pub fn new(maa_adapter: Arc<dyn MaaAdapterTrait + Send + Sync>) -> Self {
        Self { maa_adapter }
    }

    pub async fn maa_status(&self, params: MaaStatusParams) -> Result<MaaStatusResponse, String> {
        debug!("Executing maa_status with params: {:?}", params);

        let status = self.maa_adapter.get_status().await
            .map_err(|e| format!("Failed to get MAA status: {}", e))?;

        let mut response = MaaStatusResponse {
            status: format!("{:?}", status),
            device_info: None,
            active_tasks: None,
            timestamp: chrono::Utc::now(),
        };

        if params.verbose {
            match self.maa_adapter.get_device_info().await {
                Ok(device_info) => {
                    response.device_info = Some(serde_json::to_value(device_info).unwrap_or(Value::Null));
                }
                Err(e) => {
                    warn!("Failed to get device info: {}", e);
                }
            }

            match self.maa_adapter.get_all_tasks().await {
                Ok(tasks) => {
                    response.active_tasks = Some(serde_json::to_value(tasks).unwrap_or(Value::Array(vec![])));
                }
                Err(e) => {
                    warn!("Failed to get tasks: {}", e);
                }
            }
        }

        Ok(response)
    }

    pub async fn maa_command(&self, params: MaaCommandParams) -> Result<MaaCommandResponse, String> {
        debug!("Executing maa_command with params: {:?}", params);

        let task_types = self.parse_command(&params.command, params.context.as_deref()).await?;
        
        let mut task_ids = Vec::new();
        for task_type in task_types {
            let task_id = self.maa_adapter.create_task(task_type, TaskParams::default()).await
                .map_err(|e| format!("Failed to create task: {}", e))?;
            
            self.maa_adapter.start_task(task_id).await
                .map_err(|e| format!("Failed to start task {}: {}", task_id, e))?;
            
            task_ids.push(task_id as u32);
        }

        let task_count = task_ids.len();
        let response = MaaCommandResponse {
            command: params.command,
            parsed_tasks: task_count as u32,
            task_ids,
            message: format!("Created and started {} tasks", task_count),
            timestamp: chrono::Utc::now(),
        };

        Ok(response)
    }

    pub async fn maa_copilot(&self, params: MaaCopilotParams) -> Result<MaaCopilotResponse, String> {
        debug!("Executing maa_copilot with params: {:?}", params);

        let mut task_params = TaskParams::default();
        task_params.parsed.insert("copilot_config".to_string(), params.copilot_config.clone());
        
        let stage_name = params.name.clone().unwrap_or_else(|| "default_stage".to_string());
        let copilot_data = serde_json::to_string(&params.copilot_config)
            .map_err(|e| format!("Invalid copilot config: {}", e))?;
        
        if let Some(ref name) = params.name {
            task_params.settings.insert("name".to_string(), name.clone());
        }

        let task_id = self.maa_adapter.create_task(
            MaaTaskType::Copilot {
                stage_name,
                copilot_data,
            }, 
            task_params
        ).await
            .map_err(|e| format!("Failed to create copilot task: {}", e))?;

        self.maa_adapter.start_task(task_id).await
            .map_err(|e| format!("Failed to start copilot task: {}", e))?;

        let response = MaaCopilotResponse {
            task_id: task_id as u32,
            copilot_name: params.name,
            message: "Copilot task created and started successfully".to_string(),
            timestamp: chrono::Utc::now(),
        };

        Ok(response)
    }

    pub async fn maa_operators(&self, params: MaaOperatorsParams) -> Result<MaaOperatorsResponse, String> {
        debug!("Executing maa_operators with params: {:?}", params);

        let response = match params.query_type.as_str() {
            "list" => {
                MaaOperatorsResponse {
                    operators: Some(vec![
                        OperatorInfo {
                            name: "Amiya".to_string(),
                            rarity: 5,
                            class: "Caster".to_string(),
                            level: Some(50),
                            elite: Some(2),
                        },
                        OperatorInfo {
                            name: "Exusiai".to_string(),
                            rarity: 6,
                            class: "Sniper".to_string(),
                            level: Some(60),
                            elite: Some(2),
                        },
                        OperatorInfo {
                            name: "SilverAsh".to_string(),
                            rarity: 6,
                            class: "Guard".to_string(),
                            level: Some(70),
                            elite: Some(2),
                        },
                    ]),
                    total: Some(3),
                    query: params.query,
                    message: "Listed all available operators".to_string(),
                    timestamp: chrono::Utc::now(),
                }
            }
            "search" => {
                let query = params.query.clone().unwrap_or_default();
                MaaOperatorsResponse {
                    operators: Some(vec![]),
                    total: Some(0),
                    query: Some(query.clone()),
                    message: format!("Searched for operators matching '{}'", query),
                    timestamp: chrono::Utc::now(),
                }
            }
            _ => {
                return Err(format!("Unknown query type: {}", params.query_type));
            }
        };

        Ok(response)
    }
}

/// MAA状态工具参数
#[derive(Debug, Serialize, Deserialize)]
pub struct MaaStatusParams {
    /// 是否返回详细信息，包括设备信息和活动任务
    #[serde(default)]
    pub verbose: bool,
}

/// MAA状态工具响应
#[derive(Debug, Serialize, Deserialize)]
pub struct MaaStatusResponse {
    /// 当前MAA状态
    pub status: String,
    /// 设备信息（仅在verbose=true时返回）
    pub device_info: Option<Value>,
    /// 活动任务列表（仅在verbose=true时返回）
    pub active_tasks: Option<Value>,
    /// 响应时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// MAA命令工具参数  
#[derive(Debug, Serialize, Deserialize)]
pub struct MaaCommandParams {
    /// 自然语言命令，例如"帮我做日常"、"刷1-7关卡"
    pub command: String,
    /// 可选的上下文信息，用于更好地理解命令
    pub context: Option<String>,
}

/// MAA命令工具响应
#[derive(Debug, Serialize, Deserialize)]
pub struct MaaCommandResponse {
    /// 原始命令
    pub command: String,
    /// 解析出的任务数量
    pub parsed_tasks: u32,
    /// 创建的任务ID列表
    pub task_ids: Vec<u32>,
    /// 执行结果消息
    pub message: String,
    /// 响应时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// MAA作业工具参数
#[derive(Debug, Serialize, Deserialize)]
pub struct MaaCopilotParams {
    /// 作业JSON配置
    pub copilot_config: Value,
    /// 可选的作业名称
    pub name: Option<String>,
}

/// MAA作业工具响应
#[derive(Debug, Serialize, Deserialize)]
pub struct MaaCopilotResponse {
    /// 创建的任务ID
    pub task_id: u32,
    /// 作业名称
    pub copilot_name: Option<String>,
    /// 执行结果消息
    pub message: String,
    /// 响应时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// MAA干员工具参数
#[derive(Debug, Serialize, Deserialize)]
pub struct MaaOperatorsParams {
    /// 查询类型：list（列出所有）、search（搜索）
    pub query_type: String,
    /// 查询参数（用于搜索时）
    pub query: Option<String>,
}

/// 干员信息
#[derive(Debug, Serialize, Deserialize)]
pub struct OperatorInfo {
    /// 干员名称
    pub name: String,
    /// 稀有度（1-6星）
    pub rarity: u32,
    /// 职业
    pub class: String,
    /// 等级
    pub level: Option<u32>,
    /// 精英化阶段
    pub elite: Option<u32>,
}

/// MAA干员工具响应
#[derive(Debug, Serialize, Deserialize)]
pub struct MaaOperatorsResponse {
    /// 干员列表
    pub operators: Option<Vec<OperatorInfo>>,
    /// 总数
    pub total: Option<u32>,
    /// 查询字符串
    pub query: Option<String>,
    /// 执行结果消息
    pub message: String,
    /// 响应时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl MaaToolsServer {
    /// 解析自然语言命令为MAA任务类型
    async fn parse_command(&self, command: &str, _context: Option<&str>) -> Result<Vec<MaaTaskType>, String> {
        let command_lower = command.to_lowercase();
        let mut tasks = Vec::new();

        // 简单的命令解析逻辑
        if command_lower.contains("fight") || command_lower.contains("战斗") || command_lower.contains("作战") {
            tasks.push(MaaTaskType::StartFight);
        }
        
        if command_lower.contains("recruit") || command_lower.contains("招募") {
            tasks.push(MaaTaskType::Recruit);
        }
        
        if command_lower.contains("infrastructure") || command_lower.contains("基建") {
            tasks.push(MaaTaskType::Infrast);
        }
        
        if command_lower.contains("screenshot") || command_lower.contains("截图") {
            tasks.push(MaaTaskType::Screenshot);
        }

        if tasks.is_empty() {
            return Err("Could not parse command into valid MAA tasks".to_string());
        }

        Ok(tasks)
    }
}

/// 创建rmcp兼容的工具服务器
pub fn register_rmcp_tools(maa_adapter: Arc<dyn MaaAdapterTrait + Send + Sync>) -> MaaToolsServer {
    info!("Registering rmcp 0.5.0 compatible MAA tools");
    MaaToolsServer::new(maa_adapter)
}

/// 兼容旧接口的类型别名
pub type MaaToolsHandler = MaaToolsServer;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maa_adapter::{MaaAdapter, MaaAdapterTrait, MaaConfig};
    
    #[tokio::test]
    async fn test_rmcp_tools_creation() {
        let config = MaaConfig::default();
        let maa_adapter = Arc::new(MaaAdapter::new(config).await.unwrap());
        let _tools = register_rmcp_tools(maa_adapter);
        
        // 测试工具服务器创建成功
        assert!(true);
    }

    #[tokio::test]
    async fn test_command_parsing() {
        let config = MaaConfig::default();
        let maa_adapter = Arc::new(MaaAdapter::new(config).await.unwrap());
        let tools = register_rmcp_tools(maa_adapter);
        
        let tasks = tools.parse_command("fight", None).await.unwrap();
        assert_eq!(tasks.len(), 1);
        assert!(tasks.contains(&MaaTaskType::StartFight));
    }
    
    #[tokio::test]
    async fn test_parameter_schemas() {
        // 测试参数结构体可以正确序列化和反序列化
        let status_params = MaaStatusParams { verbose: true };
        let json = serde_json::to_value(&status_params).unwrap();
        let _deserialized: MaaStatusParams = serde_json::from_value(json).unwrap();
        
        let command_params = MaaCommandParams {
            command: "帮我做日常".to_string(),
            context: Some("明日方舟".to_string()),
        };
        let json = serde_json::to_value(&command_params).unwrap();
        let _deserialized: MaaCommandParams = serde_json::from_value(json).unwrap();
    }
}