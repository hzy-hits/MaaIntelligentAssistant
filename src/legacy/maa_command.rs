//! MAA Command Tool
//!
//! This tool processes natural language commands and converts them to MAA operations.
//! It supports commands like "do daily tasks", "farm stage 1-7", "recruit operators", etc.

use std::sync::Arc;
use async_trait::async_trait;
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::maa_adapter::{MaaAdapterTrait, MaaTaskType, TaskParams};
use super::{McpTool, McpError, McpResult, validation, response};

/// MAA Command Tool for processing natural language commands
pub struct MaaCommandTool {
    maa_adapter: Arc<dyn MaaAdapterTrait + Send + Sync>,
}

impl MaaCommandTool {
    /// Create a new MAA command tool
    pub fn new(maa_adapter: Arc<dyn MaaAdapterTrait + Send + Sync>) -> McpResult<Self> {
        Ok(Self { maa_adapter })
    }
    
    /// Parse natural language command into MAA task
    async fn parse_command(&self, command: &str, _context: Option<&str>) -> McpResult<Vec<MaaTaskType>> {
        let command_lower = command.to_lowercase();
        let mut tasks = Vec::new();
        
        // Simple command parsing logic
        if command_lower.contains("daily") || command_lower.contains("日常") {
            tasks.push(MaaTaskType::Daily);
        } else if command_lower.contains("farm") || command_lower.contains("刷") {
            // Extract stage if mentioned
            if let Some(_stage) = self.extract_stage(&command_lower) {
                tasks.push(MaaTaskType::StartFight);
                // Could add stage-specific parameters here
            } else {
                tasks.push(MaaTaskType::StartFight);
            }
        } else if command_lower.contains("recruit") || command_lower.contains("招募") {
            tasks.push(MaaTaskType::Recruit);
        } else if command_lower.contains("base") || command_lower.contains("基建") {
            tasks.push(MaaTaskType::Infrast);
        } else if command_lower.contains("screenshot") || command_lower.contains("截图") {
            tasks.push(MaaTaskType::Screenshot);
        } else if command_lower.contains("stop") || command_lower.contains("停止") {
            // For stop commands, we don't create a task but handle it differently
            return Ok(vec![]);
        } else {
            // Default to daily tasks for unknown commands
            debug!("Unknown command, defaulting to Daily: {}", command);
            tasks.push(MaaTaskType::Daily);
        }
        
        Ok(tasks)
    }
    
    /// Extract stage identifier from command
    fn extract_stage(&self, command: &str) -> Option<String> {
        // Simple regex to extract stage numbers like "1-7", "H5-3", etc.
        let stage_patterns = [
            r"\b(\d+-\d+)\b",           // 1-7, 4-3, etc.
            r"\b(H\d+-\d+)\b",          // H5-3, H8-4, etc.
            r"\b(S\d+-\d+)\b",          // S3-1, etc.
            r"\b(CE-\d+)\b",            // CE-5, etc.
            r"\b(CA-\d+)\b",            // CA-5, etc.
            r"\b(AP-\d+)\b",            // AP-5, etc.
        ];
        
        for pattern in &stage_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(captures) = re.captures(command) {
                    if let Some(stage) = captures.get(1) {
                        return Some(stage.as_str().to_string());
                    }
                }
            }
        }
        
        None
    }
    
    /// Execute parsed tasks
    async fn execute_tasks(&self, tasks: Vec<MaaTaskType>, execute: bool) -> McpResult<Value> {
        if !execute {
            return Ok(json!({
                "planned_tasks": tasks,
                "status": "planned",
                "message": "Tasks planned but not executed"
            }));
        }
        
        let mut results = Vec::new();
        
        for task in tasks {
            debug!("Executing MAA task: {:?}", task);
            
            // First create the task, then start it
            match self.maa_adapter.create_task(task.clone(), TaskParams::default()).await {
                Ok(task_id) => {
                    // Now start the created task
                    match self.maa_adapter.start_task(task_id).await {
                        Ok(()) => {
                            results.push(json!({
                                "task": format!("{:?}", task),
                                "task_id": task_id,
                                "status": "started",
                                "message": "Task started successfully"
                            }));
                        }
                        Err(e) => {
                            error!("Failed to start task {}: {}", task_id, e);
                            results.push(json!({
                                "task": format!("{:?}", task),
                                "task_id": task_id,
                                "status": "failed",
                                "error": format!("Failed to start: {}", e)
                            }));
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to create task {:?}: {}", task, e);
                    results.push(json!({
                        "task": format!("{:?}", task),
                        "status": "failed",
                        "error": format!("Failed to create: {}", e)
                    }));
                }
            }
        }
        
        Ok(json!({
            "executed_tasks": results,
            "status": "executed",
            "total_tasks": results.len()
        }))
    }
}

#[async_trait]
impl McpTool for MaaCommandTool {
    async fn call(&self, params: Value) -> Result<Value, McpError> {
        debug!("MaaCommandTool::call with params: {}", params);
        
        // Validate required parameters
        let command: String = validation::validate_non_empty_string(&params, "command")?;
        
        // Optional parameters
        let context: Option<String> = validation::validate_optional_param(&params, "context", "string")?;
        let execute: bool = validation::validate_optional_param(&params, "execute", "boolean")?
            .unwrap_or(true);
        let timeout_seconds: u32 = validation::validate_optional_param(&params, "timeout_seconds", "number")?
            .unwrap_or(300);
        
        info!("Processing command: '{}' (execute: {})", command, execute);
        
        // Parse command into tasks
        let tasks = self.parse_command(&command, context.as_deref()).await?;
        
        // Execute or plan tasks
        let result = self.execute_tasks(tasks, execute).await?;
        
        Ok(response::success(json!({
            "command": command,
            "context": context,
            "execute": execute,
            "timeout_seconds": timeout_seconds,
            "result": result
        })))
    }
    
    fn get_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "Natural language command to execute (e.g., 'help me with daily tasks', 'farm 1-7 10 times')"
                },
                "context": {
                    "type": "string",
                    "description": "Optional context to help with command interpretation"
                },
                "execute": {
                    "type": "boolean",
                    "description": "Whether to execute the command immediately or just plan it",
                    "default": true
                },
                "timeout_seconds": {
                    "type": "integer",
                    "description": "Maximum time to wait for completion (seconds)",
                    "default": 300,
                    "minimum": 1,
                    "maximum": 3600
                }
            },
            "required": ["command"]
        })
    }
    
    fn get_name(&self) -> &'static str {
        "maa_command"
    }
    
    fn get_description(&self) -> &'static str {
        "Process natural language commands and convert them to MAA operations. Supports commands like 'do daily tasks', 'farm stage 1-7', 'recruit operators', etc."
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maa_adapter::{MaaAdapter, MaaConfig};
    use std::sync::Arc;
    
    async fn create_test_tool() -> MaaCommandTool {
        let config = MaaConfig::default();
        let adapter = Arc::new(MaaAdapter::new(config).await.unwrap());
        MaaCommandTool::new(adapter).unwrap()
    }
    
    #[tokio::test]
    async fn test_command_parsing() {
        let tool = create_test_tool().await;
        
        // Test daily command
        let tasks = tool.parse_command("help me with daily tasks", None).await.unwrap();
        assert!(!tasks.is_empty());
        
        // Test farm command
        let tasks = tool.parse_command("farm stage 1-7", None).await.unwrap();
        assert!(!tasks.is_empty());
        
        // Test recruit command
        let tasks = tool.parse_command("recruit operators", None).await.unwrap();
        assert!(!tasks.is_empty());
    }
    
    #[tokio::test]
    async fn test_stage_extraction() {
        let tool = create_test_tool().await;
        
        assert_eq!(tool.extract_stage("farm 1-7"), Some("1-7".to_string()));
        assert_eq!(tool.extract_stage("run H5-3"), Some("H5-3".to_string()));
        assert_eq!(tool.extract_stage("do CE-5"), Some("CE-5".to_string()));
        assert_eq!(tool.extract_stage("no stage here"), None);
    }
    
    #[tokio::test]
    async fn test_tool_call() {
        let tool = create_test_tool().await;
        
        let params = json!({
            "command": "help me with daily tasks",
            "execute": false
        });
        
        let result = tool.call(params).await.unwrap();
        assert_eq!(result["success"], true);
        assert!(result["data"]["result"]["status"] == "planned");
    }
    
    #[tokio::test]
    async fn test_invalid_parameters() {
        let tool = create_test_tool().await;
        
        // Missing command
        let params = json!({
            "execute": true
        });
        
        let result = tool.call(params).await;
        assert!(result.is_err());
        
        // Empty command
        let params = json!({
            "command": ""
        });
        
        let result = tool.call(params).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_tool_metadata() {
        let tool = create_test_tool().await;
        
        assert_eq!(tool.get_name(), "maa_command");
        assert!(!tool.get_description().is_empty());
        
        let schema = tool.get_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["command"].is_object());
        assert!(schema["required"].as_array().unwrap().contains(&json!("command")));
    }
}