//! MAA任务状态通知系统
//! 
//! 基于tokio broadcast实现的事件驱动任务状态更新

use tokio::sync::broadcast;
use serde::{Serialize, Deserialize};
use std::sync::OnceLock;
use tracing::{info, debug, warn};

/// 任务状态更新事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatusEvent {
    pub task_id: i32,
    pub task_type: String,
    pub status: TaskStatus,
    pub message: String,
    pub progress: Option<f32>, // 0.0-1.0
    pub details: Option<serde_json::Value>,
    pub timestamp: String,
}

/// 任务状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    /// 任务已提交，等待执行
    Pending,
    /// 任务正在执行中
    Running,
    /// 任务成功完成
    Success,
    /// 任务执行失败
    Failed,
    /// 任务被取消
    Cancelled,
    /// 任务暂停
    Paused,
}

/// 全局任务状态广播通道
static TASK_NOTIFIER: OnceLock<broadcast::Sender<TaskStatusEvent>> = OnceLock::new();

/// 初始化任务通知系统
pub fn init_task_notification_system() -> broadcast::Receiver<TaskStatusEvent> {
    let (tx, rx) = broadcast::channel(1000); // 缓冲1000个事件
    
    match TASK_NOTIFIER.set(tx) {
        Ok(()) => {
            info!("任务通知系统初始化完成，缓冲区大小: 1000");
            rx
        }
        Err(_) => {
            warn!("任务通知系统已初始化，返回新的接收器");
            get_task_notifier().subscribe()
        }
    }
}

/// 获取任务通知发送器
pub fn get_task_notifier() -> &'static broadcast::Sender<TaskStatusEvent> {
    TASK_NOTIFIER.get_or_init(|| {
        warn!("任务通知系统未初始化，自动创建");
        let (tx, _) = broadcast::channel(1000);
        tx
    })
}

/// 创建新的任务状态接收器
pub fn subscribe_task_events() -> broadcast::Receiver<TaskStatusEvent> {
    get_task_notifier().subscribe()
}

/// 发送任务状态更新事件
pub fn notify_task_status(event: TaskStatusEvent) {
    let notifier = get_task_notifier();
    let task_id = event.task_id;
    let status = event.status.clone();
    
    match notifier.send(event) {
        Ok(subscriber_count) => {
            debug!("任务状态通知已发送: task_id={}, status={:?}, 订阅者数量={}", 
                   task_id, status, subscriber_count);
        }
        Err(e) => {
            warn!("发送任务状态通知失败: task_id={}, error={}", task_id, e);
        }
    }
}

/// 便捷函数：发送任务开始事件
pub fn notify_task_started(task_id: i32, task_type: String, message: String) {
    let event = TaskStatusEvent {
        task_id,
        task_type,
        status: TaskStatus::Running,
        message,
        progress: Some(0.0),
        details: None,
        timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    };
    notify_task_status(event);
}

/// 便捷函数：发送任务进度事件
pub fn notify_task_progress(task_id: i32, task_type: String, message: String, progress: f32) {
    let event = TaskStatusEvent {
        task_id,
        task_type,
        status: TaskStatus::Running,
        message,
        progress: Some(progress.clamp(0.0, 1.0)),
        details: None,
        timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    };
    notify_task_status(event);
}

/// 便捷函数：发送任务完成事件
pub fn notify_task_completed(task_id: i32, task_type: String, message: String, details: Option<serde_json::Value>) {
    let event = TaskStatusEvent {
        task_id,
        task_type,
        status: TaskStatus::Success,
        message,
        progress: Some(1.0),
        details,
        timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    };
    notify_task_status(event);
}

/// 便捷函数：发送任务失败事件
pub fn notify_task_failed(task_id: i32, task_type: String, message: String, error_details: Option<serde_json::Value>) {
    let event = TaskStatusEvent {
        task_id,
        task_type,
        status: TaskStatus::Failed,
        message,
        progress: None,
        details: error_details,
        timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    };
    notify_task_status(event);
}

/// 任务状态监听器，用于持续监听MAA Core的状态变化
pub struct TaskStatusMonitor {
    receiver: broadcast::Receiver<TaskStatusEvent>,
}

impl TaskStatusMonitor {
    /// 创建新的任务状态监听器
    pub fn new() -> Self {
        Self {
            receiver: subscribe_task_events(),
        }
    }
    
    /// 等待下一个任务状态事件
    pub async fn next_event(&mut self) -> Result<TaskStatusEvent, broadcast::error::RecvError> {
        self.receiver.recv().await
    }
    
    /// 等待特定任务的特定状态
    pub async fn wait_for_task_status(&mut self, task_id: i32, target_status: TaskStatus) -> Option<TaskStatusEvent> {
        loop {
            match self.next_event().await {
                Ok(event) => {
                    if event.task_id == task_id && event.status == target_status {
                        return Some(event);
                    }
                }
                Err(e) => {
                    warn!("等待任务状态事件失败: {}", e);
                    return None;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_task_notification_system() {
        let mut receiver = init_task_notification_system();
        
        // 发送测试事件
        notify_task_started(1, "test_task".to_string(), "任务开始".to_string());
        
        // 接收事件
        let event = receiver.recv().await.unwrap();
        assert_eq!(event.task_id, 1);
        assert_eq!(event.status, TaskStatus::Running);
        assert_eq!(event.message, "任务开始");
    }

    #[tokio::test]
    async fn test_task_status_monitor() {
        let _receiver = init_task_notification_system();
        let mut monitor = TaskStatusMonitor::new();
        
        // 在后台发送事件
        tokio::spawn(async {
            sleep(Duration::from_millis(100)).await;
            notify_task_completed(2, "test_task".to_string(), "任务完成".to_string(), None);
        });
        
        // 等待任务完成
        let event = monitor.wait_for_task_status(2, TaskStatus::Success).await.unwrap();
        assert_eq!(event.task_id, 2);
        assert_eq!(event.status, TaskStatus::Success);
    }
}