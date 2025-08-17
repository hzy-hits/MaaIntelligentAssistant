//! MAA Status Tool
//!
//! This tool provides status information about MAA including connection state,
//! running tasks, and device information.

use std::sync::Arc;
use async_trait::async_trait;
use serde_json::{json, Value};
use tracing::{debug, info};

use crate::maa_adapter::{MaaAdapterTrait, MaaStatus};
use super::{McpTool, McpError, McpResult, validation, response};

/// MAA Status Tool for querying system status
pub struct MaaStatusTool {
    maa_adapter: Arc<dyn MaaAdapterTrait + Send + Sync>,
}

impl MaaStatusTool {
    /// Create a new MAA status tool
    pub fn new(maa_adapter: Arc<dyn MaaAdapterTrait + Send + Sync>) -> McpResult<Self> {
        Ok(Self { maa_adapter })
    }
    
    /// Get basic MAA status
    async fn get_basic_status(&self) -> McpResult<Value> {
        debug!("Getting basic MAA status");
        
        let status = self.maa_adapter.get_status().await
            .map_err(|e| McpError::maa_failed("get_status", e.to_string()))?;
        
        Ok(json!({
            "connection": match &status {
                MaaStatus::Idle => "idle",
                MaaStatus::Connecting => "connecting",
                MaaStatus::Connected => "connected",
                MaaStatus::Running { .. } => "running",
                MaaStatus::Completed { .. } => "completed",
                MaaStatus::Failed { .. } => "failed",
                MaaStatus::Disconnected { .. } => "disconnected",
            },
            "status": format!("{:?}", status),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
    
    /// Get detailed device information
    async fn get_device_info(&self) -> McpResult<Value> {
        debug!("Getting device information");
        
        // Try to get device info - this might not be available in all implementations
        let device_info = json!({
            "available": false,
            "message": "Device information not implemented yet",
            "note": "This would include ADB connection details, device model, etc."
        });
        
        Ok(device_info)
    }
    
    /// Get running tasks information
    async fn get_tasks_info(&self) -> McpResult<Value> {
        debug!("Getting tasks information");
        
        // For now, return basic task info
        // In a full implementation, this would query the actual task queue
        let tasks_info = json!({
            "running_tasks": [],
            "pending_tasks": [],
            "completed_tasks": [],
            "total_tasks": 0,
            "message": "Task tracking not fully implemented yet"
        });
        
        Ok(tasks_info)
    }
    
    /// Get performance metrics
    async fn get_metrics(&self) -> McpResult<Value> {
        debug!("Getting performance metrics");
        
        // Basic metrics - would be more detailed in full implementation
        let metrics = json!({
            "uptime": "N/A",
            "memory_usage": "N/A",
            "cpu_usage": "N/A",
            "tasks_completed": 0,
            "success_rate": 1.0,
            "message": "Detailed metrics not implemented yet"
        });
        
        Ok(metrics)
    }
}

#[async_trait]
impl McpTool for MaaStatusTool {
    async fn call(&self, params: Value) -> Result<Value, McpError> {
        debug!("MaaStatusTool::call with params: {}", params);
        
        // Optional parameters
        let include_device: bool = validation::validate_optional_param(&params, "include_device", "boolean")?
            .unwrap_or(false);
        let include_tasks: bool = validation::validate_optional_param(&params, "include_tasks", "boolean")?
            .unwrap_or(false);
        let include_metrics: bool = validation::validate_optional_param(&params, "include_metrics", "boolean")?
            .unwrap_or(false);
        
        info!("Getting MAA status (device: {}, tasks: {}, metrics: {})", 
              include_device, include_tasks, include_metrics);
        
        // Always get basic status
        let mut result = json!({
            "basic": self.get_basic_status().await?
        });
        
        // Conditionally include additional information
        if include_device {
            result["device"] = self.get_device_info().await?;
        }
        
        if include_tasks {
            result["tasks"] = self.get_tasks_info().await?;
        }
        
        if include_metrics {
            result["metrics"] = self.get_metrics().await?;
        }
        
        Ok(response::success(json!({
            "include_device": include_device,
            "include_tasks": include_tasks,
            "include_metrics": include_metrics,
            "status": result
        })))
    }
    
    fn get_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "include_device": {
                    "type": "boolean",
                    "description": "Include detailed device information",
                    "default": false
                },
                "include_tasks": {
                    "type": "boolean",
                    "description": "Include running task details",
                    "default": false
                },
                "include_metrics": {
                    "type": "boolean",
                    "description": "Include performance metrics",
                    "default": false
                }
            }
        })
    }
    
    fn get_name(&self) -> &'static str {
        "maa_status"
    }
    
    fn get_description(&self) -> &'static str {
        "Get the current status of MAA including connection state, running tasks, and device information"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maa_adapter::{MaaAdapter, MaaConfig};
    use std::sync::Arc;
    
    async fn create_test_tool() -> MaaStatusTool {
        let config = MaaConfig::default();
        let adapter = Arc::new(MaaAdapter::new(config).await.unwrap());
        MaaStatusTool::new(adapter).unwrap()
    }
    
    #[tokio::test]
    async fn test_basic_status() {
        let tool = create_test_tool().await;
        let status = tool.get_basic_status().await.unwrap();
        
        assert!(status["connection"].is_string());
        assert!(status["status"].is_string());
        assert!(status["timestamp"].is_string());
    }
    
    #[tokio::test]
    async fn test_device_info() {
        let tool = create_test_tool().await;
        let device_info = tool.get_device_info().await.unwrap();
        
        assert!(device_info.is_object());
        assert!(device_info["available"].is_boolean());
    }
    
    #[tokio::test]
    async fn test_tasks_info() {
        let tool = create_test_tool().await;
        let tasks_info = tool.get_tasks_info().await.unwrap();
        
        assert!(tasks_info["running_tasks"].is_array());
        assert!(tasks_info["pending_tasks"].is_array());
        assert!(tasks_info["completed_tasks"].is_array());
        assert!(tasks_info["total_tasks"].is_number());
    }
    
    #[tokio::test]
    async fn test_metrics() {
        let tool = create_test_tool().await;
        let metrics = tool.get_metrics().await.unwrap();
        
        assert!(metrics.is_object());
        assert!(metrics["success_rate"].is_number());
    }
    
    #[tokio::test]
    async fn test_tool_call_basic() {
        let tool = create_test_tool().await;
        
        let params = json!({});
        let result = tool.call(params).await.unwrap();
        
        assert_eq!(result["success"], true);
        assert!(result["data"]["status"]["basic"].is_object());
    }
    
    #[tokio::test]
    async fn test_tool_call_with_options() {
        let tool = create_test_tool().await;
        
        let params = json!({
            "include_device": true,
            "include_tasks": true,
            "include_metrics": true
        });
        
        let result = tool.call(params).await.unwrap();
        
        assert_eq!(result["success"], true);
        assert!(result["data"]["status"]["device"].is_object());
        assert!(result["data"]["status"]["tasks"].is_object());
        assert!(result["data"]["status"]["metrics"].is_object());
    }
    
    #[tokio::test]
    async fn test_tool_metadata() {
        let tool = create_test_tool().await;
        
        assert_eq!(tool.get_name(), "maa_status");
        assert!(!tool.get_description().is_empty());
        
        let schema = tool.get_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["include_device"].is_object());
    }
}