//! 重构后的MAA任务队列系统
//! 
//! 优化点：
//! 1. 合并双队列为单队列+优先级
//! 2. 减少枚举variants，使用统一的任务结构
//! 3. 支持同步/异步执行模式

use anyhow::Result;
use serde_json::Value;
use tokio::sync::{oneshot, mpsc};
use serde::{Serialize, Deserialize};
use std::cmp::Ordering;
use chrono::{DateTime, Utc};

use super::task_classification_v2::{TaskPriority, TaskExecutionMode};

/// 统一的MAA任务结构
#[derive(Debug)]
pub struct MaaTask {
    /// 任务ID
    pub task_id: i32,
    /// 任务类型（对应Function Call名称）
    pub task_type: String,
    /// 任务参数（JSON格式）
    pub parameters: Value,
    /// 任务优先级
    pub priority: TaskPriority,
    /// 执行模式
    pub execution_mode: TaskExecutionMode,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 响应通道
    pub response_tx: oneshot::Sender<TaskResult>,
}

/// 任务执行结果
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskResult {
    /// 是否成功
    pub success: bool,
    /// 任务ID
    pub task_id: i32,
    /// 结果数据
    pub result: Option<Value>,
    /// 错误信息
    pub error: Option<String>,
    /// 完成时间
    pub completed_at: DateTime<Utc>,
    /// 执行耗时（秒）
    pub duration_seconds: f64,
}

/// 带优先级的任务包装器（用于优先队列）
#[derive(Debug)]
pub struct PriorityTask {
    pub task: MaaTask,
}

impl PriorityTask {
    pub fn new(task: MaaTask) -> Self {
        Self { task }
    }
}

// 实现优先队列排序：高优先级任务先执行
impl Ord for PriorityTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // 首先按优先级排序
        let priority_cmp = self.task.priority.cmp(&other.task.priority);
        if priority_cmp != Ordering::Equal {
            return priority_cmp;
        }
        
        // 相同优先级按创建时间排序（FIFO）
        other.task.created_at.cmp(&self.task.created_at)
    }
}

impl PartialOrd for PriorityTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PriorityTask {
    fn eq(&self, other: &Self) -> bool {
        self.task.priority == other.task.priority && self.task.created_at == other.task.created_at
    }
}

impl Eq for PriorityTask {}

/// MAA任务队列发送器 - V2版本（单队列+优先级）
#[derive(Clone)]
pub struct MaaTaskSender {
    task_tx: mpsc::UnboundedSender<PriorityTask>,
    task_counter: std::sync::Arc<std::sync::atomic::AtomicI32>,
}

/// MAA任务队列接收器 - V2版本
pub struct MaaTaskReceiver {
    task_rx: mpsc::UnboundedReceiver<PriorityTask>,
}

impl MaaTaskSender {
    /// 发送任务（自动分配优先级）
    pub fn send_task(
        &self,
        task_type: String,
        parameters: Value,
        priority: TaskPriority,
        execution_mode: TaskExecutionMode,
    ) -> Result<(i32, oneshot::Receiver<TaskResult>), mpsc::error::SendError<PriorityTask>> {
        // 生成任务ID
        let task_id = self.task_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        
        // 创建响应通道
        let (response_tx, response_rx) = oneshot::channel();
        
        // 构建任务
        let task = MaaTask {
            task_id,
            task_type,
            parameters,
            priority,
            execution_mode,
            created_at: Utc::now(),
            response_tx,
        };
        
        // 发送到队列
        self.task_tx.send(PriorityTask::new(task))?;
        
        Ok((task_id, response_rx))
    }
    
    /// 便捷方法：发送同步任务
    pub fn send_sync_task(
        &self,
        task_type: String,
        parameters: Value,
    ) -> Result<(i32, oneshot::Receiver<TaskResult>), mpsc::error::SendError<PriorityTask>> {
        self.send_task(task_type, parameters, TaskPriority::High, TaskExecutionMode::Synchronous)
    }
    
    /// 便捷方法：发送异步任务
    pub fn send_async_task(
        &self,
        task_type: String,
        parameters: Value,
    ) -> Result<(i32, oneshot::Receiver<TaskResult>), mpsc::error::SendError<PriorityTask>> {
        self.send_task(task_type, parameters, TaskPriority::Normal, TaskExecutionMode::Asynchronous)
    }
}

impl MaaTaskReceiver {
    /// 接收下一个优先级任务
    pub async fn recv(&mut self) -> Option<MaaTask> {
        match self.task_rx.recv().await {
            Some(priority_task) => Some(priority_task.task),
            None => None,
        }
    }
}

/// 创建V2版本的MAA任务通道（单队列+优先级）
pub fn create_maa_task_channel_v2() -> (MaaTaskSender, MaaTaskReceiver) {
    let (task_tx, task_rx) = mpsc::unbounded_channel();
    let task_counter = std::sync::Arc::new(std::sync::atomic::AtomicI32::new(1));
    
    let sender = MaaTaskSender {
        task_tx,
        task_counter,
    };
    
    let receiver = MaaTaskReceiver {
        task_rx,
    };
    
    (sender, receiver)
}

/// 任务状态跟踪 - V2版本（Worker内部管理）
#[derive(Debug, Clone)]
pub struct TaskStatus {
    pub task_id: i32,
    pub task_type: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<Value>,
    pub error: Option<String>,
}

impl TaskStatus {
    pub fn new(task_id: i32, task_type: String) -> Self {
        Self {
            task_id,
            task_type,
            status: "pending".to_string(),
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            result: None,
            error: None,
        }
    }
    
    pub fn mark_running(&mut self) {
        self.status = "running".to_string();
        self.started_at = Some(Utc::now());
    }
    
    pub fn mark_completed(&mut self, result: Value) {
        self.status = "completed".to_string();
        self.completed_at = Some(Utc::now());
        self.result = Some(result);
    }
    
    pub fn mark_failed(&mut self, error: String) {
        self.status = "failed".to_string();
        self.completed_at = Some(Utc::now());
        self.error = Some(error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};
    
    #[tokio::test]
    async fn test_task_priority_ordering() {
        let (sender, mut receiver) = create_maa_task_channel_v2();
        
        // 发送不同优先级的任务
        let _ = sender.send_async_task("low_priority".to_string(), serde_json::json!({}));
        let _ = sender.send_sync_task("high_priority".to_string(), serde_json::json!({}));
        
        // 高优先级任务应该先被接收到
        let first_task = receiver.recv().await.unwrap();
        assert_eq!(first_task.task_type, "high_priority");
        assert_eq!(first_task.priority, TaskPriority::High);
        
        let second_task = receiver.recv().await.unwrap();
        assert_eq!(second_task.task_type, "low_priority");
        assert_eq!(second_task.priority, TaskPriority::Normal);
    }
    
    #[tokio::test]
    async fn test_task_result_channel() {
        let (sender, mut receiver) = create_maa_task_channel_v2();
        
        // 发送任务并获取响应通道
        let (task_id, response_rx) = sender.send_sync_task(
            "test_task".to_string(), 
            serde_json::json!({"test": "data"})
        ).unwrap();
        
        // 模拟Worker接收任务
        let task = receiver.recv().await.unwrap();
        assert_eq!(task.task_id, task_id);
        
        // 模拟任务完成
        let result = TaskResult {
            success: true,
            task_id,
            result: Some(serde_json::json!({"completed": true})),
            error: None,
            completed_at: Utc::now(),
            duration_seconds: 1.0,
        };
        
        // 发送结果
        let _ = task.response_tx.send(result);
        
        // 验证能接收到结果
        let received_result = timeout(Duration::from_millis(100), response_rx).await.unwrap().unwrap();
        assert!(received_result.success);
        assert_eq!(received_result.task_id, task_id);
    }
}