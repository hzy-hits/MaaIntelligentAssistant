//! MAA Worker 工厂 - 解决 Send 问题的正确方法
//! 
//! 这个模块提供了启动MAA Worker的工厂方法，正确处理非Send类型的线程安全问题

use tokio::sync::broadcast;
use tracing::info;
use super::worker_v2::{MaaWorkerV2, TaskProgressEvent};
use super::task_queue_v2::MaaTaskReceiver;

/// 启动MAA Worker的工厂函数
/// 
/// 这个函数在专用线程中创建和运行MAA Worker，正确处理非Send类型
/// 返回一个可以用于SSE的事件广播器
pub fn spawn_maa_worker(task_receiver: MaaTaskReceiver) -> broadcast::Sender<TaskProgressEvent> {
    info!("准备在专用线程中启动MAA Worker V2");
    
    // 创建事件广播通道（Send + Sync）
    let (event_broadcaster, _) = broadcast::channel(1000);
    let broadcaster_clone = event_broadcaster.clone();
    
    // 在专用线程中运行MAA Worker
    std::thread::spawn(move || {
        info!("MAA Worker专用线程已启动");
        
        // 创建tokio运行时（单线程，LocalSet）
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("无法创建MAA Worker tokio runtime");
        
        // 使用LocalSet确保非Send类型的安全运行
        let local_set = tokio::task::LocalSet::new();
        
        local_set.spawn_local(async move {
            info!("开始在LocalSet中初始化MAA Worker V2");
            
            // 在这里创建MAA Worker V2（非Send类型安全）
            let maa_worker = MaaWorkerV2::new_with_broadcaster(broadcaster_clone);
            
            info!("MAA Worker V2已在线程本地创建，开始运行");
            
            // 运行Worker主循环
            maa_worker.run(task_receiver).await;
            
            info!("MAA Worker V2运行结束");
        });
        
        // 阻塞运行LocalSet
        rt.block_on(local_set);
        
        info!("MAA Worker专用线程已退出");
    });
    
    info!("MAA Worker专用线程已启动，返回事件广播器");
    
    // 返回可以在主线程使用的广播器（Send + Sync）
    event_broadcaster
}

/// 测试工厂函数
#[cfg(test)]
mod tests {
    use super::*;
    use crate::maa_core::task_queue_v2::create_maa_task_channel;
    
    #[tokio::test]
    async fn test_worker_factory() {
        // 创建任务通道
        let (_sender, receiver) = create_maa_task_channel();
        
        // 启动Worker
        let _broadcaster = spawn_maa_worker(receiver);
        
        // Worker应该在后台运行
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}