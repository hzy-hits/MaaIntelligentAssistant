//! Server-Sent Events (SSE) 推送系统
//! 
//! 用于实时推送异步任务的执行进度和结果给前端

use axum::{
    response::Sse,
    http::HeaderMap,
};
use axum::response::sse::{Event, KeepAlive};
use tokio_stream::{Stream, StreamExt};
use tokio::sync::broadcast;
use serde_json::{json, Value};
use tracing::{info, debug};
use std::time::Duration;
use futures::stream;
use std::convert::Infallible;
use chrono::Utc;

use crate::maa_core::worker_v2::TaskProgressEvent;

/// SSE事件管理器
#[derive(Clone)]
pub struct SseManager {
    /// 任务事件广播器
    task_event_tx: broadcast::Sender<TaskProgressEvent>,
}

impl SseManager {
    /// 创建新的SSE管理器
    pub fn new(task_event_tx: broadcast::Sender<TaskProgressEvent>) -> Self {
        info!("创建SSE管理器");
        Self {
            task_event_tx,
        }
    }
    
    /// 创建任务进度SSE流
    pub fn create_task_progress_stream(&self) -> impl Stream<Item = Result<Event, Infallible>> + Send + 'static {
        let mut event_rx = self.task_event_tx.subscribe();
        
        // 创建组合流：心跳 + 任务事件
        let heartbeat_stream = tokio_stream::wrappers::IntervalStream::new(
            tokio::time::interval(Duration::from_secs(30))
        ).map(|_| {
            debug!("发送SSE心跳");
            Ok(Event::default()
                .event("heartbeat")
                .data(json!({
                    "timestamp": Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                    "message": "连接正常"
                }).to_string()))
        });
        
        let task_event_stream = async_stream::stream! {
            while let Ok(task_event) = event_rx.recv().await {
                debug!("接收到任务事件: {:?}", task_event);
                
                // 转换为SSE事件
                let sse_event = Event::default()
                    .event(&task_event.event_type)
                    .id(format!("{}-{}", task_event.task_id, task_event.timestamp.timestamp_millis()))
                    .data(json!({
                        "task_id": task_event.task_id,
                        "task_type": task_event.task_type,
                        "event_type": task_event.event_type,
                        "message": task_event.message,
                        "data": task_event.data,
                        "timestamp": task_event.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string()
                    }).to_string());
                
                yield Ok(sse_event);
            }
        };
        
        // 合并心跳和任务事件流
        stream::select(heartbeat_stream, task_event_stream)
    }
    
    /// 创建特定任务的SSE流
    pub fn create_single_task_stream(&self, task_id: i32) -> impl Stream<Item = Result<Event, Infallible>> + Send + 'static {
        let mut event_rx = self.task_event_tx.subscribe();
        
        async_stream::stream! {
            // 发送初始连接事件
            let init_event = Event::default()
                .event("connected")
                .data(json!({
                    "task_id": task_id,
                    "message": format!("已连接到任务 {} 的进度流", task_id),
                    "timestamp": Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()
                }).to_string());
            yield Ok(init_event);
            
            // 过滤并发送特定任务的事件
            while let Ok(task_event) = event_rx.recv().await {
                if task_event.task_id == task_id {
                    debug!("发送任务 {} 的进度事件: {}", task_id, task_event.event_type);
                    
                    let sse_event = Event::default()
                        .event(&task_event.event_type)
                        .id(format!("{}-{}", task_event.task_id, task_event.timestamp.timestamp_millis()))
                        .data(json!({
                            "task_id": task_event.task_id,
                            "task_type": task_event.task_type,
                            "event_type": task_event.event_type,
                            "message": task_event.message,
                            "data": task_event.data,
                            "timestamp": task_event.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string()
                        }).to_string());
                    
                    yield Ok(sse_event);
                    
                    // 如果任务完成或失败，发送结束事件后结束流
                    if task_event.event_type == "completed" || task_event.event_type == "failed" {
                        debug!("任务 {} 结束，关闭SSE流", task_id);
                        
                        let end_event = Event::default()
                            .event("stream_end")
                            .data(json!({
                                "task_id": task_id,
                                "message": "任务已结束，SSE连接将关闭",
                                "timestamp": Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()
                            }).to_string());
                        yield Ok(end_event);
                        
                        break;
                    }
                }
            }
        }
    }
    
    /// 手动发送任务事件（用于测试）
    pub fn send_task_event(&self, event: TaskProgressEvent) -> Result<(), broadcast::error::SendError<TaskProgressEvent>> {
        self.task_event_tx.send(event).map(|_| ())
    }
}

/// 创建通用任务进度SSE响应
pub fn create_task_progress_sse(sse_manager: SseManager) -> Sse<impl Stream<Item = Result<Event, Infallible>> + Send + 'static> {
    info!("创建任务进度SSE流");
    
    Sse::new(sse_manager.create_task_progress_stream())
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(30))
                .text("keep-alive")
        )
}

/// 创建特定任务SSE响应
pub fn create_single_task_sse(
    sse_manager: SseManager, 
    task_id: i32
) -> Sse<impl Stream<Item = Result<Event, Infallible>> + Send + 'static> {
    info!("创建任务 {} 的SSE流", task_id);
    
    Sse::new(sse_manager.create_single_task_stream(task_id))
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keep-alive")
        )
}

/// 设置SSE响应头
pub fn set_sse_headers(headers: &mut HeaderMap) {
    headers.insert("Cache-Control", "no-cache".parse().unwrap());
    headers.insert("Connection", "keep-alive".parse().unwrap());
    headers.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
    headers.insert("Access-Control-Allow-Headers", "Cache-Control".parse().unwrap());
}

/// SSE事件类型定义
pub mod events {
    use super::*;
    
    /// 创建任务开始事件
    pub fn create_task_started_event(task_id: i32, task_type: &str, message: &str) -> TaskProgressEvent {
        TaskProgressEvent {
            task_id,
            task_type: task_type.to_string(),
            event_type: "started".to_string(),
            message: message.to_string(),
            data: Some(json!({"status": "started"})),
            timestamp: Utc::now(),
        }
    }
    
    /// 创建任务进度事件
    pub fn create_task_progress_event(
        task_id: i32, 
        task_type: &str, 
        message: &str, 
        progress_data: Option<Value>
    ) -> TaskProgressEvent {
        TaskProgressEvent {
            task_id,
            task_type: task_type.to_string(),
            event_type: "progress".to_string(),
            message: message.to_string(),
            data: progress_data,
            timestamp: Utc::now(),
        }
    }
    
    /// 创建任务完成事件
    pub fn create_task_completed_event(
        task_id: i32, 
        task_type: &str, 
        result_data: Value
    ) -> TaskProgressEvent {
        TaskProgressEvent {
            task_id,
            task_type: task_type.to_string(),
            event_type: "completed".to_string(),
            message: "任务执行完成".to_string(),
            data: Some(result_data),
            timestamp: Utc::now(),
        }
    }
    
    /// 创建任务失败事件
    pub fn create_task_failed_event(
        task_id: i32, 
        task_type: &str, 
        error: &str
    ) -> TaskProgressEvent {
        TaskProgressEvent {
            task_id,
            task_type: task_type.to_string(),
            event_type: "failed".to_string(),
            message: format!("任务执行失败: {}", error),
            data: Some(json!({"error": error})),
            timestamp: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};
    use tokio_stream::StreamExt;

    #[tokio::test]
    async fn test_sse_manager_creation() {
        let (tx, _rx) = broadcast::channel(100);
        let manager = SseManager::new(tx);
        
        // 测试创建流
        let mut stream = Box::pin(manager.create_task_progress_stream());
        
        // 应该能够接收心跳事件
        if let Ok(Some(event)) = timeout(Duration::from_secs(1), stream.next()).await {
            // 心跳事件应该成功创建
            assert!(event.is_ok());
        }
    }

    #[tokio::test]
    async fn test_task_event_filtering() {
        let (tx, _rx) = broadcast::channel(100);
        let manager = SseManager::new(tx.clone());
        
        // 创建任务1的专用流
        let mut task1_stream = Box::pin(manager.create_single_task_stream(1));
        
        // 发送任务1的事件
        let event1 = TaskProgressEvent {
            task_id: 1,
            task_type: "test_task".to_string(),
            event_type: "started".to_string(),
            message: "任务开始".to_string(),
            data: None,
            timestamp: Utc::now(),
        };
        
        let _ = tx.send(event1);
        
        // 应该能接收到任务1的事件
        if let Ok(Some(received)) = timeout(Duration::from_millis(100), task1_stream.next()).await {
            assert!(received.is_ok());
        }
    }
}