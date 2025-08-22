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
// use super::task_classification_v2::*; // 未使用的导入已移除

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
    
    /// 处理MAA Core回调事件并转发到SSE
    pub fn handle_maa_callback(&mut self, task_id: i32, msg_code: i32, details: Value) {
        let event_type = match msg_code {
            10001 => "taskchain_started",
            10002 => "taskchain_completed", 
            10000 => "taskchain_failed",
            20001 => "subtask_started",
            20002 => "subtask_completed",
            20003 => "subtask_info",
            20000 => "subtask_failed",
            _ => "unknown"
        };
        
        // 提取任务详情
        let task_name = details.get("details")
            .and_then(|d| d.get("task"))
            .and_then(|t| t.as_str())
            .unwrap_or("unknown");
            
        let task_chain = details.get("taskchain")
            .and_then(|tc| tc.as_str())
            .unwrap_or("unknown");
            
        let message = match msg_code {
            10001 => format!("任务链 {} 开始执行", task_chain),
            10002 => format!("任务链 {} 执行完成", task_chain),
            10000 => format!("任务链 {} 执行失败", task_chain),
            20001 => format!("开始执行子任务: {}", task_name),
            20002 => format!("子任务 {} 完成", task_name),
            20003 => {
                if let Some(facility) = details.get("details").and_then(|d| d.get("facility")).and_then(|f| f.as_str()) {
                    format!("处理设施: {}", facility)
                } else {
                    format!("子任务信息: {}", task_name)
                }
            },
            20000 => format!("子任务 {} 失败", task_name),
            _ => format!("未知事件: {}", task_name)
        };
        
        // 更新内部任务状态
        if let Some(status) = self.task_statuses.get_mut(&task_id) {
            match msg_code {
                10002 => {
                    // 任务链完成，标记为成功完成
                    status.mark_completed(details.clone());
                },
                10000 | 20000 => {
                    // 任务失败
                    let error = details.get("what")
                        .and_then(|v| v.as_str())
                        .unwrap_or("未知错误")
                        .to_string();
                    status.mark_failed(error);
                },
                _ => {
                    // 进度更新 - 使用status字段存储进度信息
                    status.status = format!("running: {}", message);
                }
            }
        }
        
        // 转发到SSE系统
        let sse_event = TaskProgressEvent {
            task_id,
            task_type: task_chain.to_string(),
            event_type: event_type.to_string(),
            message,
            data: Some(details),
            timestamp: Utc::now(),
        };
        
        // 发送SSE事件（忽略发送失败，避免阻塞）
        let _ = self.event_broadcaster.send(sse_event);
        
        debug!("MAA回调事件已转发到SSE: task_id={}, event_type={}, msg_code={}", task_id, event_type, msg_code);
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
        
        // 不再手动发送started事件 - 由MAA Core回调统一处理
        info!("开始执行任务: {} (task_id: {})", task_type, task_id);
        
        // 执行具体的MAA任务
        let result = self.execute_maa_task(&task).await;
        
        // 更新任务状态 - 不再手动发送完成/失败事件，由MAA Core回调统一处理
        if let Some(status) = self.task_statuses.get_mut(&task_id) {
            match &result {
                Ok(task_result) => {
                    if task_result.success {
                        status.mark_completed(task_result.result.clone().unwrap_or(json!({})));
                        info!("任务 {} 执行成功 (task_id: {})", task_type, task_id);
                    } else {
                        let error = task_result.error.clone().unwrap_or("未知错误".to_string());
                        status.mark_failed(error.clone());
                        warn!("任务 {} 执行失败 (task_id: {}): {}", task_type, task_id, error);
                    }
                },
                Err(e) => {
                    let error = format!("{}", e);
                    status.mark_failed(error.clone());
                    error!("任务 {} 发生错误 (task_id: {}): {}", task_type, task_id, error);
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
            "maa_recruit_enhanced" => {
                debug!("执行公开招募任务");
                let max_times = task.parameters.get("max_times")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(4);
                let expedite = task.parameters.get("expedite")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let skip_robot = task.parameters.get("skip_robot")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                
                let params = format!(
                    r#"{{"enable": true, "select": [3, 4, 5, 6], "confirm": [3, 4, 5, 6], "times": {}, "set_time": true, "expedite": {}, "skip_robot": {}}}"#,
                    max_times, expedite, skip_robot
                );
                
                match self.core.execute_task("Recruit", &params) {
                    Ok(task_id) => Ok(json!({
                        "maa_task_id": task_id,
                        "max_times": max_times,
                        "expedite": expedite,
                        "skip_robot": skip_robot,
                        "status": "招募任务已提交到MAA Core"
                    })),
                    Err(e) => Err(anyhow!("招募任务失败: {}", e))
                }
            },
            "maa_rewards_enhanced" => {
                debug!("执行奖励收集任务");
                let award_type = task.parameters.get("award_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("all");
                
                let params = r#"{"enable": true}"#;
                
                match self.core.execute_task("Award", params) {
                    Ok(task_id) => Ok(json!({
                        "maa_task_id": task_id,
                        "award_type": award_type,
                        "status": "奖励收集任务已提交到MAA Core"
                    })),
                    Err(e) => Err(anyhow!("奖励收集失败: {}", e))
                }
            },
            "maa_closedown" => {
                debug!("执行游戏关闭任务");
                let params = r#"{"enable": true}"#;
                
                match self.core.execute_task("CloseDown", params) {
                    Ok(task_id) => Ok(json!({
                        "maa_task_id": task_id,
                        "status": "游戏关闭任务已提交到MAA Core"
                    })),
                    Err(e) => Err(anyhow!("游戏关闭失败: {}", e))
                }
            },
            "maa_roguelike_enhanced" => {
                debug!("执行集成战略任务");
                let theme = task.parameters.get("theme")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Phantom");
                let mode = task.parameters.get("mode")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                
                let params = format!(
                    r#"{{"enable": true, "theme": "{}", "mode": {}}}"#,
                    theme, mode
                );
                
                match self.core.execute_task("Roguelike", &params) {
                    Ok(task_id) => Ok(json!({
                        "maa_task_id": task_id,
                        "theme": theme,
                        "mode": mode,
                        "status": "集成战略任务已提交到MAA Core"
                    })),
                    Err(e) => Err(anyhow!("集成战略任务失败: {}", e))
                }
            },
            "maa_copilot_enhanced" => {
                debug!("执行作业任务");
                let filename = task.parameters.get("filename")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                let params = format!(
                    r#"{{"enable": true, "filename": "{}"}}"#,
                    filename
                );
                
                match self.core.execute_task("Copilot", &params) {
                    Ok(task_id) => Ok(json!({
                        "maa_task_id": task_id,
                        "filename": filename,
                        "status": "作业任务已提交到MAA Core"
                    })),
                    Err(e) => Err(anyhow!("作业任务失败: {}", e))
                }
            },
            "maa_sss_copilot" => {
                debug!("执行保全派驻任务");
                let filename = task.parameters.get("filename")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                let params = format!(
                    r#"{{"enable": true, "filename": "{}"}}"#,
                    filename
                );
                
                match self.core.execute_task("SSSCopilot", &params) {
                    Ok(task_id) => Ok(json!({
                        "maa_task_id": task_id,
                        "filename": filename,
                        "status": "保全派驻任务已提交到MAA Core"
                    })),
                    Err(e) => Err(anyhow!("保全派驻任务失败: {}", e))
                }
            },
            "maa_reclamation" => {
                debug!("执行生息演算任务");
                let theme = task.parameters.get("theme")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Fire");
                let mode = task.parameters.get("mode")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                
                let params = format!(
                    r#"{{"enable": true, "theme": "{}", "mode": {}}}"#,
                    theme, mode
                );
                
                match self.core.execute_task("Reclamation", &params) {
                    Ok(task_id) => Ok(json!({
                        "maa_task_id": task_id,
                        "theme": theme,
                        "mode": mode,
                        "status": "生息演算任务已提交到MAA Core"
                    })),
                    Err(e) => Err(anyhow!("生息演算任务失败: {}", e))
                }
            },
            "maa_credit_store_enhanced" => {
                debug!("执行信用商店任务");
                let buy_first = task.parameters.get("buy_first")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                    .unwrap_or_else(|| vec!["龙门币", "赤金"]);
                
                let buy_first_json = buy_first.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(",");
                let params = format!(
                    r#"{{"enable": true, "buy_first": [{}]}}"#,
                    buy_first_json
                );
                
                match self.core.execute_task("Mall", &params) {
                    Ok(task_id) => Ok(json!({
                        "maa_task_id": task_id,
                        "buy_first": buy_first,
                        "status": "信用商店任务已提交到MAA Core"
                    })),
                    Err(e) => Err(anyhow!("信用商店任务失败: {}", e))
                }
            },
            "maa_depot_management" => {
                debug!("执行仓库管理任务");
                let params = r#"{"enable": true}"#;
                
                match self.core.execute_task("Depot", params) {
                    Ok(task_id) => Ok(json!({
                        "maa_task_id": task_id,
                        "status": "仓库管理任务已提交到MAA Core"
                    })),
                    Err(e) => Err(anyhow!("仓库管理任务失败: {}", e))
                }
            },
            "maa_operator_box" => {
                debug!("执行干员管理任务");
                let params = r#"{"enable": true}"#;
                
                match self.core.execute_task("OperBox", params) {
                    Ok(task_id) => Ok(json!({
                        "maa_task_id": task_id,
                        "status": "干员管理任务已提交到MAA Core"
                    })),
                    Err(e) => Err(anyhow!("干员管理任务失败: {}", e))
                }
            },
            "maa_custom_task" => {
                debug!("执行自定义任务");
                let task_name = task.parameters.get("task_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("CustomTask");
                let custom_params = task.parameters.get("params")
                    .cloned()
                    .unwrap_or(json!({}));
                
                let params_str = custom_params.to_string();
                
                match self.core.execute_task(task_name, &params_str) {
                    Ok(task_id) => Ok(json!({
                        "maa_task_id": task_id,
                        "task_name": task_name,
                        "status": "自定义任务已提交到MAA Core"
                    })),
                    Err(e) => Err(anyhow!("自定义任务失败: {}", e))
                }
            },
            "maa_video_recognition" => {
                debug!("执行视频识别任务");
                let video_path = task.parameters.get("video_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                let params = format!(
                    r#"{{"enable": true, "filename": "{}"}}"#,
                    video_path
                );
                
                match self.core.execute_task("VideoRecognition", &params) {
                    Ok(task_id) => Ok(json!({
                        "maa_task_id": task_id,
                        "video_path": video_path,
                        "status": "视频识别任务已提交到MAA Core"
                    })),
                    Err(e) => Err(anyhow!("视频识别任务失败: {}", e))
                }
            },
            "maa_system_management" => {
                debug!("执行系统管理任务");
                let operation = task.parameters.get("operation")
                    .and_then(|v| v.as_str())
                    .unwrap_or("status");
                
                match operation {
                    "restart" => {
                        let params = r#"{"enable": true}"#;
                        match self.core.execute_task("StartUp", params) {
                            Ok(task_id) => Ok(json!({
                                "maa_task_id": task_id,
                                "operation": "restart",
                                "status": "系统重启任务已提交到MAA Core"
                            })),
                            Err(e) => Err(anyhow!("系统重启失败: {}", e))
                        }
                    },
                    "stop" => {
                        let params = r#"{"enable": true}"#;
                        match self.core.execute_task("CloseDown", params) {
                            Ok(task_id) => Ok(json!({
                                "maa_task_id": task_id,
                                "operation": "stop",
                                "status": "系统停止任务已提交到MAA Core"
                            })),
                            Err(e) => Err(anyhow!("系统停止失败: {}", e))
                        }
                    },
                    _ => {
                        Ok(json!({
                            "operation": operation,
                            "status": "系统状态查询完成",
                            "maa_initialized": self.core.is_initialized(),
                            "maa_connected": self.core.is_connected()
                        }))
                    }
                }
            },
            "maa_get_task_list" => {
                debug!("获取任务列表");
                use crate::maa_core::basic_ops::get_tasks_list;
                match get_tasks_list().await {
                    Ok(tasks_data) => Ok(tasks_data),
                    Err(e) => Err(anyhow!("获取任务列表失败: {}", e))
                }
            },
            "maa_adjust_task_params" => {
                debug!("动态调整任务参数");
                use crate::maa_core::basic_ops::{set_task_params, adjust_task_strategy};
                
                let task_id = task.parameters.get("task_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(1) as i32;
                let strategy = task.parameters.get("strategy")
                    .and_then(|v| v.as_str())
                    .unwrap_or("reduce_difficulty");
                    
                if strategy == "custom" {
                    // 自定义参数调整
                    let custom_params = task.parameters.get("custom_params")
                        .cloned()
                        .unwrap_or_else(|| json!({}));
                    match set_task_params(task_id, custom_params).await {
                        Ok(result) => Ok(result),
                        Err(e) => Err(anyhow!("自定义参数调整失败: {}", e))
                    }
                } else {
                    // 智能策略调整
                    let context = task.parameters.get("context")
                        .cloned()
                        .unwrap_or_else(|| json!({}));
                    match adjust_task_strategy(task_id, strategy, context).await {
                        Ok(result) => Ok(result),
                        Err(e) => Err(anyhow!("智能策略调整失败: {}", e))
                    }
                }
            },
            "maa_emergency_home" => {
                debug!("紧急返回主界面");
                use crate::maa_core::basic_ops::back_to_home;
                
                let reason = task.parameters.get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("user_request");
                
                info!("紧急返回主界面，原因: {}", reason);
                match back_to_home().await {
                    Ok(result) => Ok(result),
                    Err(e) => Err(anyhow!("紧急返回失败: {}", e))
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