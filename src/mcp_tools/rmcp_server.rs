//! 正确的 rmcp 0.5.0 MCP 服务器实现
//!
//! 这是重新设计的真正使用官方 rmcp SDK 的实现

use std::sync::Arc;
use std::future::Future;
use rmcp::{
    tool, tool_router,
    handler::server::{ServerHandler, tool::ToolRouter},
    model::{Content, CallToolResult, InitializeRequestParam, InitializeResult, 
            ListToolsResult, CallToolRequestParam, PaginatedRequestParam},
    service::RequestContext, RoleServer,
    ErrorData as McpError,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::maa_adapter::{MaaAdapterTrait, MaaTaskType, TaskParams};

/// MAA工具服务器 - 使用正确的rmcp实现
#[derive(Clone)]
pub struct MaaMcpServer {
    maa_adapter: Arc<dyn MaaAdapterTrait + Send + Sync>,
    tool_router: ToolRouter<Self>,
}

/// MAA状态查询参数
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusRequest {
    /// 是否返回详细信息
    #[serde(default)]
    pub verbose: bool,
}

/// MAA命令执行参数
#[derive(Debug, Serialize, Deserialize)]
pub struct CommandRequest {
    /// 自然语言命令
    pub command: String,
    /// 可选的上下文信息
    pub context: Option<String>,
}

/// MAA作业执行参数
#[derive(Debug, Serialize, Deserialize)]
pub struct CopilotRequest {
    /// 作业配置JSON
    pub copilot_config: serde_json::Value,
    /// 可选的作业名称
    pub name: Option<String>,
}

/// MAA干员查询参数
#[derive(Debug, Serialize, Deserialize)]
pub struct OperatorsRequest {
    /// 查询类型：list, search
    pub query_type: String,
    /// 查询参数
    pub query: Option<String>,
}

#[tool_router]
impl MaaMcpServer {
    pub fn new(maa_adapter: Arc<dyn MaaAdapterTrait + Send + Sync>) -> Self {
        Self {
            maa_adapter,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "获取MAA适配器状态和设备信息")]
    async fn maa_status(&self, request: StatusRequest) -> Result<CallToolResult, McpError> {
        debug!("Executing maa_status with verbose: {}", request.verbose);

        // 获取基本状态
        let status = self.maa_adapter.get_status().await
            .map_err(|e| McpError::internal_error(format!("Failed to get MAA status: {}", e)))?;

        let mut result = vec![
            Content::text(format!("MAA状态: {:?}", status))
        ];

        // 如果需要详细信息
        if request.verbose {
            match self.maa_adapter.get_device_info().await {
                Ok(device_info) => {
                    let device_json = serde_json::to_string_pretty(&device_info)
                        .unwrap_or_else(|_| "设备信息序列化失败".to_string());
                    result.push(Content::text(format!("设备信息:\n{}", device_json)));
                }
                Err(e) => {
                    result.push(Content::text(format!("获取设备信息失败: {}", e)));
                }
            }

            match self.maa_adapter.get_all_tasks().await {
                Ok(tasks) => {
                    let tasks_json = serde_json::to_string_pretty(&tasks)
                        .unwrap_or_else(|_| "任务信息序列化失败".to_string());
                    result.push(Content::text(format!("活动任务:\n{}", tasks_json)));
                }
                Err(e) => {
                    result.push(Content::text(format!("获取任务信息失败: {}", e)));
                }
            }
        }

        Ok(CallToolResult::success(result))
    }

    #[tool(description = "使用自然语言执行MAA命令，如'帮我做日常'、'截图'等")]
    async fn maa_command(&self, request: CommandRequest) -> Result<CallToolResult, McpError> {
        debug!("Executing maa_command: {}", request.command);

        // 解析自然语言命令
        let task_types = self.parse_natural_language(&request.command, request.context.as_deref())
            .await
            .map_err(|e| McpError::invalid_params(format!("命令解析失败: {}", e)))?;

        let mut results = vec![
            Content::text(format!("解析命令 '{}' 为 {} 个任务", request.command, task_types.len()))
        ];

        // 执行任务
        let mut task_ids = Vec::new();
        for (idx, task_type) in task_types.into_iter().enumerate() {
            match self.maa_adapter.create_task(task_type, TaskParams::default()).await {
                Ok(task_id) => {
                    match self.maa_adapter.start_task(task_id).await {
                        Ok(()) => {
                            task_ids.push(task_id);
                            results.push(Content::text(format!("任务 {} 创建并启动成功 (ID: {})", idx + 1, task_id)));
                        }
                        Err(e) => {
                            results.push(Content::text(format!("任务 {} 启动失败: {}", idx + 1, e)));
                        }
                    }
                }
                Err(e) => {
                    results.push(Content::text(format!("任务 {} 创建失败: {}", idx + 1, e)));
                }
            }
        }

        results.push(Content::text(format!("总计成功创建 {} 个任务", task_ids.len())));

        Ok(CallToolResult::success(results))
    }

    #[tool(description = "执行MAA作业，需要提供作业配置JSON")]
    async fn maa_copilot(&self, request: CopilotRequest) -> Result<CallToolResult, McpError> {
        debug!("Executing maa_copilot with name: {:?}", request.name);

        let stage_name = request.name.clone().unwrap_or_else(|| "自定义作业".to_string());
        let copilot_data = serde_json::to_string(&request.copilot_config)
            .map_err(|e| McpError::invalid_params(format!("作业配置JSON无效: {}", e)))?;

        let mut task_params = TaskParams::default();
        task_params.parsed.insert("copilot_config".to_string(), request.copilot_config);
        
        if let Some(ref name) = request.name {
            task_params.settings.insert("name".to_string(), name.clone());
        }

        // 创建作业任务
        let task_id = self.maa_adapter.create_task(
            MaaTaskType::Copilot {
                stage_name: stage_name.clone(),
                copilot_data,
            },
            task_params
        ).await
            .map_err(|e| McpError::internal_error(format!("创建作业任务失败: {}", e)))?;

        // 启动任务
        self.maa_adapter.start_task(task_id).await
            .map_err(|e| McpError::internal_error(format!("启动作业任务失败: {}", e)))?;

        let results = vec![
            Content::text(format!("作业 '{}' 创建成功", stage_name)),
            Content::text(format!("任务ID: {}", task_id)),
            Content::text("作业已开始执行".to_string()),
        ];

        Ok(CallToolResult::success(results))
    }

    #[tool(description = "查询和管理MAA干员信息")]
    async fn maa_operators(&self, request: OperatorsRequest) -> Result<CallToolResult, McpError> {
        debug!("Executing maa_operators: {} {:?}", request.query_type, request.query);

        let results = match request.query_type.as_str() {
            "list" => {
                // 这里应该调用operator_manager模块
                // 目前返回模拟数据
                vec![
                    Content::text("当前检测到的干员:".to_string()),
                    Content::text("• 阿米娅 (5★ 术师) - E2 Lv.50".to_string()),
                    Content::text("• 能天使 (6★ 狙击) - E2 Lv.60".to_string()),
                    Content::text("• 银灰 (6★ 近卫) - E2 Lv.70".to_string()),
                    Content::text("总计: 3 个干员".to_string()),
                ]
            }
            "search" => {
                let query = request.query.unwrap_or_default();
                vec![
                    Content::text(format!("搜索干员: '{}'", query)),
                    Content::text("搜索结果: 暂无匹配的干员".to_string()),
                ]
            }
            _ => {
                return Err(McpError::invalid_params(format!("未知的查询类型: {}", request.query_type)));
            }
        };

        Ok(CallToolResult::success(results))
    }
}

impl MaaMcpServer {
    /// 解析自然语言命令为MAA任务类型
    async fn parse_natural_language(&self, command: &str, _context: Option<&str>) -> Result<Vec<MaaTaskType>, String> {
        let command_lower = command.to_lowercase();
        let mut tasks = Vec::new();

        // 简单的关键词匹配
        if command_lower.contains("截图") || command_lower.contains("screenshot") {
            tasks.push(MaaTaskType::Screenshot);
        }
        
        if command_lower.contains("日常") || command_lower.contains("daily") {
            // 日常任务通常包括基建、招募等
            tasks.push(MaaTaskType::Infrast);
            tasks.push(MaaTaskType::Recruit);
        }
        
        if command_lower.contains("战斗") || command_lower.contains("fight") || command_lower.contains("作战") {
            tasks.push(MaaTaskType::StartFight);
        }
        
        if command_lower.contains("招募") || command_lower.contains("recruit") {
            tasks.push(MaaTaskType::Recruit);
        }
        
        if command_lower.contains("基建") || command_lower.contains("infrastructure") || command_lower.contains("infrast") {
            tasks.push(MaaTaskType::Infrast);
        }

        if tasks.is_empty() {
            return Err(format!("无法理解命令: '{}'。支持的命令包括：截图、日常、战斗、招募、基建", command));
        }

        Ok(tasks)
    }
}

// 实现ServerHandler以成为真正的MCP服务器
impl ServerHandler for MaaMcpServer {
    async fn initialize(&self, _request: InitializeRequestParam, _context: RequestContext<RoleServer>) -> Result<InitializeResult, McpError> {
        info!("MAA MCP服务器初始化成功");
        Ok(InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: rmcp::model::ServerCapabilities {
                tools: Some(rmcp::model::ToolsCapability { list_changed: Some(false) }),
                ..Default::default()
            },
            server_info: rmcp::model::Implementation {
                name: "maa-intelligent-server".to_string(),
                version: "0.1.0".to_string(),
            },
        })
    }

    async fn list_tools(&self, _request: Option<PaginatedRequestParam>, _context: RequestContext<RoleServer>) -> Result<ListToolsResult, McpError> {
        let tools = self.tool_router.list_tools();
        Ok(ListToolsResult {
            tools,
            next_cursor: None,
        })
    }

    async fn call_tool(&self, request: CallToolRequestParam, _context: RequestContext<RoleServer>) -> Result<CallToolResult, McpError> {
        self.tool_router.call_tool(self, request.into()).await
    }
}

/// 创建MAA MCP服务器实例
pub fn create_maa_mcp_server(maa_adapter: Arc<dyn MaaAdapterTrait + Send + Sync>) -> MaaMcpServer {
    info!("创建MAA MCP服务器");
    MaaMcpServer::new(maa_adapter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maa_adapter::{MaaAdapter, MaaConfig};

    #[tokio::test]
    async fn test_mcp_server_creation() {
        let config = MaaConfig::default();
        let maa_adapter = Arc::new(MaaAdapter::new(config).await.unwrap());
        let _server = create_maa_mcp_server(maa_adapter);
        // 服务器创建成功
    }

    #[tokio::test]
    async fn test_natural_language_parsing() {
        let config = MaaConfig::default();
        let maa_adapter = Arc::new(MaaAdapter::new(config).await.unwrap());
        let server = create_maa_mcp_server(maa_adapter);

        let tasks = server.parse_natural_language("帮我做日常", None).await.unwrap();
        assert!(tasks.len() >= 2); // 应该至少包含基建和招募

        let tasks = server.parse_natural_language("截图", None).await.unwrap();
        assert_eq!(tasks.len(), 1);
        assert!(matches!(tasks[0], MaaTaskType::Screenshot));
    }
}