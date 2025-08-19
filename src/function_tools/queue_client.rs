use anyhow::Result;
use serde_json::Value;
use tokio::sync::oneshot;
use tracing::{debug, error};

use crate::maa_core::{MaaTask, MaaTaskSender};

/// MAA任务队列客户端
/// 
/// 提供简化的接口来向MAA工作线程发送任务并等待响应
#[derive(Clone)]
pub struct MaaQueueClient {
    task_sender: MaaTaskSender,
}

impl MaaQueueClient {
    /// 创建新的队列客户端
    pub fn new(task_sender: MaaTaskSender) -> Self {
        Self { task_sender }
    }
    
    /// 发送启动任务
    pub async fn startup(&self, client_type: String, start_app: bool, close_app: bool) -> Result<Value> {
        let (response_tx, response_rx) = oneshot::channel();
        
        let task = MaaTask::Startup {
            client_type,
            start_app,
            close_app,
            response_tx,
        };
        
        self.send_task(task, response_rx).await
    }
    
    /// 发送连接任务
    pub async fn connect(&self, address: String) -> Result<i32> {
        let (response_tx, response_rx) = oneshot::channel();
        
        let task = MaaTask::Connect {
            address,
            response_tx,
        };
        
        self.send_task(task, response_rx).await
    }
    
    /// 发送战斗任务
    pub async fn combat(&self, stage: String, medicine: i32, stone: i32, times: i32) -> Result<Value> {
        let (response_tx, response_rx) = oneshot::channel();
        
        let task = MaaTask::Combat {
            stage,
            medicine,
            stone,
            times,
            response_tx,
        };
        
        self.send_task(task, response_rx).await
    }
    
    /// 发送招募任务
    pub async fn recruit(&self, max_times: i32, expedite: bool, skip_robot: bool) -> Result<Value> {
        let (response_tx, response_rx) = oneshot::channel();
        
        let task = MaaTask::Recruit {
            max_times,
            expedite,
            skip_robot,
            response_tx,
        };
        
        self.send_task(task, response_rx).await
    }
    
    /// 发送基建任务
    pub async fn infrastructure(&self, facility: Vec<String>, drones: String, threshold: f64) -> Result<Value> {
        let (response_tx, response_rx) = oneshot::channel();
        
        let task = MaaTask::Infrastructure {
            facility,
            drones,
            threshold,
            response_tx,
        };
        
        self.send_task(task, response_rx).await
    }
    
    /// 发送肉鸽任务
    pub async fn roguelike(&self, theme: String, mode: i32, starts_count: i32) -> Result<Value> {
        let (response_tx, response_rx) = oneshot::channel();
        
        let task = MaaTask::Roguelike {
            theme,
            mode,
            starts_count,
            response_tx,
        };
        
        self.send_task(task, response_rx).await
    }
    
    /// 发送作业任务
    pub async fn copilot(&self, filename: String, formation: bool) -> Result<Value> {
        let (response_tx, response_rx) = oneshot::channel();
        
        let task = MaaTask::Copilot {
            filename,
            formation,
            response_tx,
        };
        
        self.send_task(task, response_rx).await
    }
    
    /// 发送保全派驻任务
    pub async fn sss_copilot(&self, filename: String, loop_times: i32) -> Result<Value> {
        let (response_tx, response_rx) = oneshot::channel();
        
        let task = MaaTask::SssCopilot {
            filename,
            loop_times,
            response_tx,
        };
        
        self.send_task(task, response_rx).await
    }
    
    /// 发送生息演算任务
    pub async fn reclamation(&self, theme: String, mode: i32) -> Result<Value> {
        let (response_tx, response_rx) = oneshot::channel();
        
        let task = MaaTask::Reclamation {
            theme,
            mode,
            response_tx,
        };
        
        self.send_task(task, response_rx).await
    }
    
    /// 发送奖励领取任务
    pub async fn rewards(&self, award: bool, mail: bool, recruit: bool, orundum: bool) -> Result<Value> {
        let (response_tx, response_rx) = oneshot::channel();
        
        let task = MaaTask::Rewards {
            award,
            mail,
            recruit,
            orundum,
            response_tx,
        };
        
        self.send_task(task, response_rx).await
    }
    
    /// 发送信用商店任务
    pub async fn credit_store(&self, credit_fight: bool) -> Result<Value> {
        let (response_tx, response_rx) = oneshot::channel();
        
        let task = MaaTask::CreditStore {
            credit_fight,
            response_tx,
        };
        
        self.send_task(task, response_rx).await
    }
    
    /// 发送仓库管理任务
    pub async fn depot_management(&self, enable: bool) -> Result<Value> {
        let (response_tx, response_rx) = oneshot::channel();
        
        let task = MaaTask::DepotManagement {
            enable,
            response_tx,
        };
        
        self.send_task(task, response_rx).await
    }
    
    /// 发送干员管理任务
    pub async fn operator_box(&self, enable: bool) -> Result<Value> {
        let (response_tx, response_rx) = oneshot::channel();
        
        let task = MaaTask::OperatorBox {
            enable,
            response_tx,
        };
        
        self.send_task(task, response_rx).await
    }
    
    /// 发送关闭游戏任务
    pub async fn closedown(&self) -> Result<Value> {
        let (response_tx, response_rx) = oneshot::channel();
        
        let task = MaaTask::Closedown {
            response_tx,
        };
        
        self.send_task(task, response_rx).await
    }
    
    /// 发送自定义任务
    pub async fn custom_task(&self, task_type: String, params: String) -> Result<Value> {
        let (response_tx, response_rx) = oneshot::channel();
        
        let task = MaaTask::CustomTask {
            task_type,
            params,
            response_tx,
        };
        
        self.send_task(task, response_rx).await
    }
    
    /// 发送视频识别任务
    pub async fn video_recognition(&self, video_path: String) -> Result<Value> {
        let (response_tx, response_rx) = oneshot::channel();
        
        let task = MaaTask::VideoRecognition {
            video_path,
            response_tx,
        };
        
        self.send_task(task, response_rx).await
    }
    
    /// 发送系统管理任务
    pub async fn system_management(&self, action: String) -> Result<Value> {
        let (response_tx, response_rx) = oneshot::channel();
        
        let task = MaaTask::SystemManagement {
            action,
            response_tx,
        };
        
        self.send_task(task, response_rx).await
    }
    
    /// 获取MAA状态
    pub async fn get_status(&self) -> Result<Value> {
        let (response_tx, response_rx) = oneshot::channel();
        
        let task = MaaTask::GetStatus {
            response_tx,
        };
        
        self.send_task(task, response_rx).await
    }
    
    /// 截图
    pub async fn take_screenshot(&self) -> Result<Vec<u8>> {
        let (response_tx, response_rx) = oneshot::channel();
        
        let task = MaaTask::TakeScreenshot {
            response_tx,
        };
        
        self.send_task(task, response_rx).await
    }
    
    /// 点击
    pub async fn perform_click(&self, x: i32, y: i32) -> Result<i32> {
        let (response_tx, response_rx) = oneshot::channel();
        
        let task = MaaTask::PerformClick {
            x,
            y,
            response_tx,
        };
        
        self.send_task(task, response_rx).await
    }
    
    /// 停止所有任务
    pub async fn stop_all_tasks(&self) -> Result<()> {
        let (response_tx, response_rx) = oneshot::channel();
        
        let task = MaaTask::StopAllTasks {
            response_tx,
        };
        
        self.send_task(task, response_rx).await
    }
    
    /// 通用的任务发送和响应处理
    async fn send_task<T>(&self, task: MaaTask, response_rx: oneshot::Receiver<Result<T>>) -> Result<T> {
        // 发送任务到MAA工作线程
        if let Err(e) = self.task_sender.send(task) {
            error!("无法发送任务到MAA工作线程: {:?}", e);
            return Err(anyhow::anyhow!("MAA工作线程不可用"));
        }
        
        // 等待响应
        match response_rx.await {
            Ok(result) => {
                debug!("收到MAA工作线程响应");
                result
            }
            Err(e) => {
                error!("等待MAA工作线程响应失败: {:?}", e);
                Err(anyhow::anyhow!("MAA工作线程响应超时或关闭"))
            }
        }
    }
}