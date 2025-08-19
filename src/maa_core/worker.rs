use anyhow::{Result, anyhow};
use serde_json::{json, Value};
use tracing::{info, debug, warn, error};
use chrono::Utc;

use super::{MaaCore, task_queue::{MaaTask, MaaTaskReceiver}};

/// MAA工作线程
/// 
/// 这是整个系统中唯一拥有MAA Core实例的线程
/// 所有MAA操作都通过消息队列路由到这里执行
/// 确保线程安全和状态一致性
pub struct MaaWorker {
    core: MaaCore,
}

impl MaaWorker {
    /// 创建新的MAA工作者
    pub fn new() -> Self {
        info!("创建MAA工作者实例");
        Self {
            core: MaaCore::new(),
        }
    }
    
    /// 启动MAA工作者主循环
    /// 
    /// 这个函数会一直运行，处理从任务队列接收到的所有MAA任务
    pub async fn run(mut self, mut task_rx: MaaTaskReceiver) {
        info!("🚀 MAA工作者启动，开始处理任务队列");
        
        while let Some(task) = task_rx.recv().await {
            debug!("📨 收到MAA任务: {:?}", std::mem::discriminant(&task));
            
            let result = self.handle_task(task).await;
            if let Err(e) = result {
                error!("❌ 任务处理失败: {:?}", e);
            }
        }
        
        warn!("⚠️ MAA工作者退出 - 任务队列已关闭");
    }
    
    /// 处理单个MAA任务
    async fn handle_task(&mut self, task: MaaTask) -> Result<()> {
        match task {
            MaaTask::Startup { client_type, start_app, close_app, response_tx } => {
                let result = self.handle_startup(&client_type, start_app, close_app).await;
                let _ = response_tx.send(result);
            }
            
            MaaTask::Connect { address, response_tx } => {
                let result = self.handle_connect(&address);
                let _ = response_tx.send(result);
            }
            
            MaaTask::Combat { stage, medicine, stone, times, response_tx } => {
                let result = self.handle_combat(&stage, medicine, stone, times).await;
                let _ = response_tx.send(result);
            }
            
            MaaTask::Recruit { max_times, expedite, skip_robot, response_tx } => {
                let result = self.handle_recruit(max_times, expedite, skip_robot).await;
                let _ = response_tx.send(result);
            }
            
            MaaTask::Infrastructure { facility, drones, threshold, response_tx } => {
                let result = self.handle_infrastructure(&facility, &drones, threshold).await;
                let _ = response_tx.send(result);
            }
            
            MaaTask::Roguelike { theme, mode, starts_count, response_tx } => {
                let result = self.handle_roguelike(&theme, mode, starts_count).await;
                let _ = response_tx.send(result);
            }
            
            MaaTask::Copilot { filename, formation, response_tx } => {
                let result = self.handle_copilot(&filename, formation).await;
                let _ = response_tx.send(result);
            }
            
            MaaTask::SssCopilot { filename, loop_times, response_tx } => {
                let result = self.handle_sss_copilot(&filename, loop_times).await;
                let _ = response_tx.send(result);
            }
            
            MaaTask::Reclamation { theme, mode, response_tx } => {
                let result = self.handle_reclamation(&theme, mode).await;
                let _ = response_tx.send(result);
            }
            
            MaaTask::Rewards { award, mail, recruit, orundum, response_tx } => {
                let result = self.handle_rewards(award, mail, recruit, orundum).await;
                let _ = response_tx.send(result);
            }
            
            MaaTask::CreditStore { credit_fight, response_tx } => {
                let result = self.handle_credit_store(credit_fight).await;
                let _ = response_tx.send(result);
            }
            
            MaaTask::DepotManagement { enable, response_tx } => {
                let result = self.handle_depot_management(enable).await;
                let _ = response_tx.send(result);
            }
            
            MaaTask::OperatorBox { enable, response_tx } => {
                let result = self.handle_operator_box(enable).await;
                let _ = response_tx.send(result);
            }
            
            MaaTask::Closedown { response_tx } => {
                let result = self.handle_closedown().await;
                let _ = response_tx.send(result);
            }
            
            MaaTask::CustomTask { task_type, params, response_tx } => {
                let result = self.handle_custom_task(&task_type, &params).await;
                let _ = response_tx.send(result);
            }
            
            MaaTask::VideoRecognition { video_path, response_tx } => {
                let result = self.handle_video_recognition(&video_path).await;
                let _ = response_tx.send(result);
            }
            
            MaaTask::SystemManagement { action, response_tx } => {
                let result = self.handle_system_management(&action).await;
                let _ = response_tx.send(result);
            }
            
            MaaTask::GetStatus { response_tx } => {
                let result = self.handle_get_status().await;
                let _ = response_tx.send(result);
            }
            
            MaaTask::TakeScreenshot { response_tx } => {
                let result = self.handle_take_screenshot();
                let _ = response_tx.send(result);
            }
            
            MaaTask::PerformClick { x, y, response_tx } => {
                let result = self.handle_perform_click(x, y);
                let _ = response_tx.send(result);
            }
            
            MaaTask::StopAllTasks { response_tx } => {
                let result = self.handle_stop_all_tasks();
                let _ = response_tx.send(result);
            }
        }
        
        Ok(())
    }
    
    /// 处理游戏启动任务
    async fn handle_startup(&mut self, client_type: &str, start_app: bool, close_app: bool) -> Result<Value> {
        info!("🚀 处理游戏启动任务: client={}, start_app={}, close_app={}", client_type, start_app, close_app);
        
        // 确保MAA已初始化
        if !self.core.is_initialized() {
            info!("MAA未初始化，开始初始化...");
            self.core.initialize()?;
        }
        
        // 确保设备已连接
        if !self.core.is_connected() {
            let address = std::env::var("MAA_DEVICE_ADDRESS").unwrap_or_else(|_| "127.0.0.1:1717".to_string());
            info!("设备未连接，连接到: {}", address);
            self.core.connect(&address)?;
        }
        
        // 创建启动任务参数
        let params = json!({
            "enable": true,
            "client_type": client_type,
            "start_app": start_app,
            "close_app": close_app
        });
        
        let params_str = serde_json::to_string(&params)
            .map_err(|e| anyhow!("序列化任务参数失败: {}", e))?;
        
        debug!("StartUp任务参数: {}", params_str);
        
        let task_id = self.core.execute_task("StartUp", &params_str)?;
        
        Ok(json!({
            "task_id": task_id,
            "client_type": client_type,
            "start_app": start_app,
            "close_app": close_app,
            "status": "started"
        }))
    }
    
    /// 处理设备连接任务
    fn handle_connect(&mut self, address: &str) -> Result<i32> {
        info!("🔌 处理设备连接任务: {}", address);
        
        // 确保MAA已初始化
        if !self.core.is_initialized() {
            info!("MAA未初始化，开始初始化...");
            self.core.initialize()?;
        }
        
        self.core.connect(address)
    }
    
    /// 处理战斗刷图任务
    async fn handle_combat(&mut self, stage: &str, medicine: i32, stone: i32, times: i32) -> Result<Value> {
        info!("⚔️ 处理战斗任务: {} x {}, medicine={}, stone={}", stage, times, medicine, stone);
        
        // 确保MAA已初始化和连接
        self.ensure_ready().await?;
        
        let params = json!({
            "enable": true,
            "stage": stage,
            "medicine": medicine,
            "stone": stone,
            "times": times
        });
        
        let params_str = serde_json::to_string(&params)
            .map_err(|e| anyhow!("序列化任务参数失败: {}", e))?;
        
        debug!("Fight任务参数: {}", params_str);
        
        let task_id = self.core.execute_task("Fight", &params_str)?;
        
        Ok(json!({
            "task_id": task_id,
            "stage": stage,
            "medicine": medicine,
            "stone": stone,
            "times": times,
            "status": "started"
        }))
    }
    
    /// 处理公开招募任务
    async fn handle_recruit(&mut self, max_times: i32, expedite: bool, skip_robot: bool) -> Result<Value> {
        info!("🎯 处理招募任务: times={}, expedite={}, skip_robot={}", max_times, expedite, skip_robot);
        
        self.ensure_ready().await?;
        
        let params = json!({
            "enable": true,
            "refresh": true,
            "select": [4, 5, 6],
            "confirm": [3, 4, 5, 6],
            "times": max_times,
            "set_time": true,
            "expedite": expedite,
            "skip_robot": skip_robot
        });
        
        let params_str = serde_json::to_string(&params)
            .map_err(|e| anyhow!("序列化任务参数失败: {}", e))?;
        
        debug!("Recruit任务参数: {}", params_str);
        
        let task_id = self.core.execute_task("Recruit", &params_str)?;
        
        Ok(json!({
            "task_id": task_id,
            "max_times": max_times,
            "expedite": expedite,
            "skip_robot": skip_robot,
            "status": "started"
        }))
    }
    
    /// 确保MAA已准备就绪（初始化+连接）
    async fn ensure_ready(&mut self) -> Result<()> {
        if !self.core.is_initialized() {
            info!("MAA未初始化，开始初始化...");
            self.core.initialize()?;
        }
        
        if !self.core.is_connected() {
            let address = std::env::var("MAA_DEVICE_ADDRESS").unwrap_or_else(|_| "127.0.0.1:1717".to_string());
            info!("设备未连接，连接到: {}", address);
            self.core.connect(&address)?;
        }
        
        Ok(())
    }
    
    // 其他任务处理方法的简化实现...
    async fn handle_infrastructure(&mut self, _facility: &[String], _drones: &str, _threshold: f64) -> Result<Value> {
        self.ensure_ready().await?;
        Ok(json!({"status": "infrastructure_stub"}))
    }
    
    async fn handle_roguelike(&mut self, _theme: &str, _mode: i32, _starts_count: i32) -> Result<Value> {
        self.ensure_ready().await?;
        Ok(json!({"status": "roguelike_stub"}))
    }
    
    async fn handle_copilot(&mut self, _filename: &str, _formation: bool) -> Result<Value> {
        self.ensure_ready().await?;
        Ok(json!({"status": "copilot_stub"}))
    }
    
    async fn handle_sss_copilot(&mut self, _filename: &str, _loop_times: i32) -> Result<Value> {
        self.ensure_ready().await?;
        Ok(json!({"status": "sss_copilot_stub"}))
    }
    
    async fn handle_reclamation(&mut self, _theme: &str, _mode: i32) -> Result<Value> {
        self.ensure_ready().await?;
        Ok(json!({"status": "reclamation_stub"}))
    }
    
    async fn handle_rewards(&mut self, _award: bool, _mail: bool, _recruit: bool, _orundum: bool) -> Result<Value> {
        self.ensure_ready().await?;
        Ok(json!({"status": "rewards_stub"}))
    }
    
    async fn handle_credit_store(&mut self, _credit_fight: bool) -> Result<Value> {
        self.ensure_ready().await?;
        Ok(json!({"status": "credit_store_stub"}))
    }
    
    async fn handle_depot_management(&mut self, _enable: bool) -> Result<Value> {
        self.ensure_ready().await?;
        Ok(json!({"status": "depot_management_stub"}))
    }
    
    async fn handle_operator_box(&mut self, _enable: bool) -> Result<Value> {
        self.ensure_ready().await?;
        Ok(json!({"status": "operator_box_stub"}))
    }
    
    async fn handle_closedown(&mut self) -> Result<Value> {
        self.ensure_ready().await?;
        Ok(json!({"status": "closedown_stub"}))
    }
    
    async fn handle_custom_task(&mut self, _task_type: &str, _params: &str) -> Result<Value> {
        self.ensure_ready().await?;
        Ok(json!({"status": "custom_task_stub"}))
    }
    
    async fn handle_video_recognition(&mut self, _video_path: &str) -> Result<Value> {
        self.ensure_ready().await?;
        Ok(json!({"status": "video_recognition_stub"}))
    }
    
    async fn handle_system_management(&mut self, _action: &str) -> Result<Value> {
        self.ensure_ready().await?;
        Ok(json!({"status": "system_management_stub"}))
    }
    
    async fn handle_get_status(&mut self) -> Result<Value> {
        let status = self.core.get_status();
        Ok(json!({
            "maa_status": status,
            "timestamp": Utc::now(),
            "connected": self.core.is_connected(),
            "running": false
        }))
    }
    
    fn handle_take_screenshot(&mut self) -> Result<Vec<u8>> {
        info!("📸 执行截图操作");
        self.core.screenshot()
    }
    
    fn handle_perform_click(&mut self, x: i32, y: i32) -> Result<i32> {
        info!("👆 执行点击操作: ({}, {})", x, y);
        self.core.click(x, y)
    }
    
    fn handle_stop_all_tasks(&mut self) -> Result<()> {
        info!("⏹️ 停止所有MAA任务");
        // 实现停止逻辑
        Ok(())
    }
}