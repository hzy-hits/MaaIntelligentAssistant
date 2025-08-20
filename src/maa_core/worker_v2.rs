//! 重构后的MAA工作者
//! 
//! 优化点：
//! 1. 简化任务状态管理，移到worker内部
//! 2. 支持同步/异步任务区别处理
//! 3. 集成SSE推送机制

use anyhow::{Result, anyhow};
use serde_json::{json, Value};
use tracing::{info, debug, warn, error};
use chrono::Utc;
use std::collections::HashMap;
use tokio::sync::broadcast;
use base64;

use super::{MaaCore, task_queue_v2::*};
use super::task_classification_v2::*;

/// SSE事件类型
#[derive(Debug, Clone)]
pub struct TaskProgressEvent {
    pub task_id: i32,
    pub task_type: String,
    pub event_type: String,  // "started", "progress", "completed", "failed"
    pub message: String,
    pub data: Option<Value>,
    pub timestamp: chrono::DateTime<Utc>,
}

/// 重构后的MAA工作者 - V2版本
/// 
/// 变更：
/// 1. 内部管理任务状态，不依赖全局状态
/// 2. 支持SSE事件推送
/// 3. 简化任务处理逻辑
pub struct MaaWorkerV2 {
    core: MaaCore,
    /// 内部任务状态映射
    task_statuses: HashMap<i32, TaskStatus>,
    /// SSE事件广播器
    pub event_broadcaster: broadcast::Sender<TaskProgressEvent>,
}

impl MaaWorkerV2 {
    /// 创建新的MAA工作者（返回事件广播器的发送端）
    pub fn new() -> (Self, broadcast::Sender<TaskProgressEvent>) {
        info!("创建MAA工作者实例V2");
        
        // 创建事件广播通道
        let (event_broadcaster, _event_receiver) = broadcast::channel(1000);
        
        let worker = Self {
            core: MaaCore::new(),
            task_statuses: HashMap::new(),
            event_broadcaster: event_broadcaster.clone(),
        };
        
        (worker, event_broadcaster)
    }
    
    /// 使用外部广播器创建MAA工作者（用于工厂模式）
    pub fn new_with_broadcaster(event_broadcaster: broadcast::Sender<TaskProgressEvent>) -> Self {
        info!("使用外部广播器创建MAA工作者实例V2");
        
        Self {
            core: MaaCore::new(),
            task_statuses: HashMap::new(),
            event_broadcaster,
        }
    }
    
    /// 启动MAA工作者主循环 - V2版本（单队列+优先级）
    pub async fn run(mut self, mut task_rx: MaaTaskReceiver) {
        info!("MAA工作者V2启动，开始处理统一优先级任务队列");
        
        while let Some(task) = task_rx.recv().await {
            debug!("收到MAA任务: {} (ID: {}, 优先级: {:?})", task.task_type, task.task_id, task.priority);
            
            // 处理任务
            let result = self.handle_task(task).await;
            if let Err(e) = result {
                error!("任务处理失败: {:?}", e);
            }
        }
        
        warn!("MAA工作者V2退出 - 任务队列已关闭");
    }
    
    /// 处理单个MAA任务 - 包含完整的SSE推送和状态管理
    async fn handle_task(&mut self, task: MaaTask) -> Result<()> {
        let task_id = task.task_id;
        let task_type = task.task_type.clone();
        let start_time = Utc::now();
        
        // 创建任务状态
        let mut status = TaskStatus::new(task_id, task_type.clone());
        status.mark_running();
        self.task_statuses.insert(task_id, status);
        
        // 发送任务开始事件
        let _ = self.event_broadcaster.send(TaskProgressEvent {
            task_id,
            task_type: task_type.clone(),
            event_type: "started".to_string(),
            message: format!("任务 {} 开始执行", get_task_type_description(&task_type)),
            data: Some(json!({
                "task_id": task_id,
                "priority": task.priority,
                "execution_mode": task.execution_mode,
                "estimated_duration": estimate_task_duration(&task_type)
            })),
            timestamp: start_time,
        });
        
        // 执行具体的MAA任务
        let result = self.execute_maa_task(&task).await;
        
        // 更新任务状态并发送完成事件
        if let Some(status) = self.task_statuses.get_mut(&task_id) {
            match &result {
                Ok(task_result) => {
                    if task_result.success {
                        status.mark_completed(task_result.result.clone().unwrap_or(json!({})));
                        
                        // 发送成功事件
                        let _ = self.event_broadcaster.send(TaskProgressEvent {
                            task_id,
                            task_type: task_type.clone(),
                            event_type: "completed".to_string(),
                            message: format!("任务 {} 执行成功", get_task_type_description(&task_type)),
                            data: task_result.result.clone(),
                            timestamp: Utc::now(),
                        });
                    } else {
                        let error = task_result.error.clone().unwrap_or("未知错误".to_string());
                        status.mark_failed(error.clone());
                        
                        // 发送失败事件
                        let _ = self.event_broadcaster.send(TaskProgressEvent {
                            task_id,
                            task_type: task_type.clone(),
                            event_type: "failed".to_string(),
                            message: format!("任务 {} 执行失败: {}", get_task_type_description(&task_type), error),
                            data: Some(json!({"error": error})),
                            timestamp: Utc::now(),
                        });
                    }
                },
                Err(e) => {
                    let error = format!("{}", e);
                    status.mark_failed(error.clone());
                    
                    // 发送错误事件
                    let _ = self.event_broadcaster.send(TaskProgressEvent {
                        task_id,
                        task_type: task_type.clone(),
                        event_type: "failed".to_string(),
                        message: format!("任务 {} 发生错误: {}", get_task_type_description(&task_type), error),
                        data: Some(json!({"error": error})),
                        timestamp: Utc::now(),
                    });
                }
            }
        }
        
        // 发送任务结果到响应通道
        match result {
            Ok(task_result) => {
                let _ = task.response_tx.send(task_result);
            },
            Err(e) => {
                let error_result = TaskResult {
                    success: false,
                    task_id,
                    result: None,
                    error: Some(format!("{}", e)),
                    completed_at: Utc::now(),
                    duration_seconds: (Utc::now() - start_time).num_milliseconds() as f64 / 1000.0,
                };
                let _ = task.response_tx.send(error_result);
            }
        }
        
        Ok(())
    }
    
    /// 执行具体的MAA任务 - 减少JSON序列化，直接处理参数
    async fn execute_maa_task(&mut self, task: &MaaTask) -> Result<TaskResult> {
        let start_time = Utc::now();
        
        // 确保MAA Core已初始化和连接
        if !self.core.is_initialized() {
            info!("初始化MAA Core");
            self.core.initialize()?;
        }
        
        // 连接到设备（如果尚未连接）
        if !self.core.is_connected() {
            let device_address = std::env::var("MAA_DEVICE_ADDRESS")
                .unwrap_or_else(|_| "127.0.0.1:1717".to_string());
            info!("连接到设备: {}", device_address);
            self.core.connect(&device_address)?;
        }
        
        // 根据任务类型执行不同的操作 - 优化：减少JSON序列化
        let result = match task.task_type.as_str() {
            "maa_take_screenshot" => {
                debug!("执行截图任务");
                match self.core.screenshot() {
                    Ok(image_data) => {
                        use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
                        let base64_data = BASE64_STANDARD.encode(&image_data);
                        Ok(json!({
                            "screenshot": base64_data,
                            "size": image_data.len(),
                            "format": "PNG",
                            "timestamp": Utc::now().to_rfc3339()
                        }))
                    },
                    Err(e) => Err(anyhow!("截图失败: {}", e))
                }
            },
            "maa_startup" => {
                debug!("执行游戏启动任务");
                let client_type = task.parameters.get("client_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Official");
                let start_app = task.parameters.get("start_app")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                
                // 优化：直接构造参数字符串，减少JSON序列化
                let params = format!(
                    r#"{{"enable": true, "client_type": "{}", "start_app": {}}}"#,
                    client_type, start_app
                );
                
                match self.core.execute_task("StartUp", &params) {
                    Ok(task_id) => Ok(json!({
                        "maa_task_id": task_id,
                        "client_type": client_type,
                        "start_app": start_app,
                        "status": "启动任务已提交到MAA Core"
                    })),
                    Err(e) => Err(anyhow!("启动失败: {}", e))
                }
            },
            "maa_combat_enhanced" => {
                debug!("执行战斗任务");
                let stage = task.parameters.get("stage")
                    .and_then(|v| v.as_str())
                    .unwrap_or("1-7");
                let times = task.parameters.get("times")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(1);
                
                // 优化：直接构造参数字符串
                let params = format!(
                    r#"{{"enable": true, "stage": "{}", "medicine": {}}}"#,
                    stage, times
                );
                
                match self.core.execute_task("Fight", &params) {
                    Ok(task_id) => Ok(json!({
                        "maa_task_id": task_id,
                        "stage": stage,
                        "times": times,
                        "status": "战斗任务已提交到MAA Core"
                    })),
                    Err(e) => Err(anyhow!("战斗任务失败: {}", e))
                }
            },
            "maa_infrastructure_enhanced" => {
                debug!("执行基建管理任务");
                let operation_mode = task.parameters.get("operation_mode")
                    .and_then(|v| v.as_str())
                    .unwrap_or("full_auto");
                
                // 根据操作模式构造MAA Core期望的参数格式
                let params = match operation_mode {
                    "full_auto" => r#"{"enable": true, "mode": 0, "facility": ["Mfg", "Trade", "Power", "Control", "Reception", "Office", "Dorm"], "drones": "Money"}"#.to_string(),
                    "collect_only" => r#"{"enable": true, "mode": 1, "facility": ["Mfg", "Trade", "Power", "Control", "Reception", "Office", "Dorm"]}"#.to_string(),
                    "custom" => {
                        // 从参数中提取自定义设施列表
                        let facilities = task.parameters.get("facilities")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                            .unwrap_or_else(|| vec!["Mfg", "Trade", "Power", "Control"]);
                        let facilities_json = facilities.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(",");
                        format!(r#"{{"enable": true, "mode": 0, "facility": [{}], "drones": "Money"}}"#, facilities_json)
                    },
                    _ => r#"{"enable": true, "mode": 0, "facility": ["Mfg", "Trade", "Power", "Control", "Reception", "Office", "Dorm"], "drones": "Money"}"#.to_string(),
                };
                
                match self.core.execute_task("Infrast", &params) {
                    Ok(task_id) => Ok(json!({
                        "maa_task_id": task_id,
                        "operation_mode": operation_mode,
                        "status": "基建管理任务已提交到MAA Core"
                    })),
                    Err(e) => Err(anyhow!("基建管理任务失败: {}", e))
                }
            },
            _ => {
                // 通用任务处理 - 优化：直接传递参数，避免重复序列化
                debug!("执行通用任务: {}", task.task_type);
                let params_str = task.parameters.to_string();
                match self.core.execute_task(&task.task_type, &params_str) {
                    Ok(task_id) => Ok(json!({
                        "maa_task_id": task_id,
                        "task_type": task.task_type,
                        "status": "任务已提交到MAA Core"
                    })),
                    Err(e) => Err(anyhow!("任务执行失败: {}", e))
                }
            }
        };
        
        let end_time = Utc::now();
        let duration = (end_time - start_time).num_milliseconds() as f64 / 1000.0;
        
        match result {
            Ok(data) => Ok(TaskResult {
                success: true,
                task_id: task.task_id,
                result: Some(data),
                error: None,
                completed_at: end_time,
                duration_seconds: duration,
            }),
            Err(e) => Ok(TaskResult {
                success: false,
                task_id: task.task_id,
                result: None,
                error: Some(format!("{}", e)),
                completed_at: end_time,
                duration_seconds: duration,
            })
        }
    }
    
    /// 获取任务状态（内部状态管理）
    pub fn get_task_status(&self, task_id: i32) -> Option<&TaskStatus> {
        self.task_statuses.get(&task_id)
    }
    
    /// 获取所有任务状态
    pub fn get_all_task_statuses(&self) -> &HashMap<i32, TaskStatus> {
        &self.task_statuses
    }
    
    /// 清理旧的已完成任务状态
    pub fn cleanup_old_tasks(&mut self, max_age_minutes: i64) {
        let cutoff_time = Utc::now() - chrono::Duration::minutes(max_age_minutes);
        self.task_statuses.retain(|_, status| {
            status.completed_at.is_none() || 
            status.completed_at.unwrap() > cutoff_time
        });
        
        info!("清理了 {} 分钟前的旧任务状态", max_age_minutes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};
    
    #[tokio::test]
    async fn test_worker_v2_creation() {
        let (worker, broadcaster) = MaaWorkerV2::new();
        
        // 验证工作者和广播器都正确创建
        assert_eq!(worker.task_statuses.len(), 0);
        
        // 测试发送事件
        let event = TaskProgressEvent {
            task_id: 1,
            task_type: "test".to_string(),
            event_type: "started".to_string(),
            message: "测试事件".to_string(),
            data: None,
            timestamp: Utc::now(),
        };
        
        let result = broadcaster.send(event);
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_task_status_management() {
        let (mut worker, _broadcaster) = MaaWorkerV2::new();
        
        // 添加任务状态
        let status = TaskStatus::new(1, "test_task".to_string());
        worker.task_statuses.insert(1, status);
        
        // 验证状态存在
        assert!(worker.get_task_status(1).is_some());
        assert_eq!(worker.get_all_task_statuses().len(), 1);
        
        // 测试清理
        worker.cleanup_old_tasks(0); // 清理所有任务
        // 由于任务还未完成，不应该被清理
        assert_eq!(worker.get_all_task_statuses().len(), 1);
    }
}