//! MAA 任务状态管理模块
//! 
//! 管理异步MAA任务的状态追踪和查询

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use once_cell::sync::Lazy;
use tracing::{info, debug, warn};

/// 全局任务状态管理器
static GLOBAL_TASK_STATUS: Lazy<Arc<Mutex<HashMap<i32, MaaTaskStatus>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// MAA 任务状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaaTaskStatus {
    /// 任务ID
    pub task_id: i32,
    /// 任务类型 (如 "Infrast", "Fight", "Recruit")
    pub task_type: String,
    /// 任务参数
    pub parameters: Value,
    /// 当前状态
    pub status: TaskStatus,
    /// 任务创建时间
    pub created_at: DateTime<Utc>,
    /// 任务完成时间
    pub completed_at: Option<DateTime<Utc>>,
    /// 进度信息
    pub progress: Option<String>,
    /// 执行结果
    pub result: Option<Value>,
    /// 错误信息
    pub error: Option<String>,
}

/// 任务状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// 已创建，等待执行
    Pending,
    /// 正在执行中
    Running,
    /// 执行成功完成
    Completed,
    /// 执行失败
    Failed,
    /// 任务超时
    Timeout,
}

impl MaaTaskStatus {
    /// 创建新的任务状态
    pub fn new(task_id: i32, task_type: String, parameters: Value) -> Self {
        Self {
            task_id,
            task_type,
            parameters,
            status: TaskStatus::Pending,
            created_at: Utc::now(),
            completed_at: None,
            progress: None,
            result: None,
            error: None,
        }
    }
    
    /// 标记任务为运行中
    pub fn start(&mut self) {
        self.status = TaskStatus::Running;
    }
    
    /// 标记任务完成
    pub fn complete(&mut self, result: Value) {
        self.status = TaskStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.result = Some(result);
    }
    
    /// 标记任务失败
    pub fn fail(&mut self, error: String) {
        self.status = TaskStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.error = Some(error);
    }
    
    /// 更新任务进度
    pub fn update_progress(&mut self, progress: String) {
        self.progress = Some(progress);
    }
    
    /// 检查任务是否已完成（成功或失败）
    pub fn is_finished(&self) -> bool {
        matches!(self.status, TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Timeout)
    }
}

/// 注册新任务
pub fn register_task(task_id: i32, task_type: String, parameters: Value) {
    let mut tasks = GLOBAL_TASK_STATUS.lock().unwrap();
    let task_status = MaaTaskStatus::new(task_id, task_type, parameters);
    tasks.insert(task_id, task_status);
    debug!("注册任务状态: task_id={}", task_id);
}

/// 启动任务（设置为运行中状态）
pub fn start_task(task_id: i32) {
    let mut tasks = GLOBAL_TASK_STATUS.lock().unwrap();
    if let Some(task) = tasks.get_mut(&task_id) {
        task.start();
        // 任务开始执行: task_id={}
    }
}

/// 完成任务
pub fn complete_task(task_id: i32, result: Value) {
    let mut tasks = GLOBAL_TASK_STATUS.lock().unwrap();
    if let Some(task) = tasks.get_mut(&task_id) {
        task.complete(result);
        // 任务执行完成: task_id={}
    }
}

/// 任务执行失败
pub fn fail_task(task_id: i32, error: String) {
    let mut tasks = GLOBAL_TASK_STATUS.lock().unwrap();
    if let Some(task) = tasks.get_mut(&task_id) {
        let error_clone = error.clone();
        task.fail(error);
        warn!("任务执行失败: task_id={}, error={}", task_id, error_clone);
    }
}

/// 更新任务进度
pub fn update_task_progress(task_id: i32, progress: String) {
    let mut tasks = GLOBAL_TASK_STATUS.lock().unwrap();
    if let Some(task) = tasks.get_mut(&task_id) {
        let progress_clone = progress.clone();
        task.update_progress(progress);
        debug!("📈 任务进度更新: task_id={}, progress={}", task_id, progress_clone);
    }
}

/// 获取任务状态
pub fn get_task_status(task_id: i32) -> Option<MaaTaskStatus> {
    let tasks = GLOBAL_TASK_STATUS.lock().unwrap();
    tasks.get(&task_id).cloned()
}

/// 获取所有任务状态
pub fn get_all_tasks() -> Vec<MaaTaskStatus> {
    let tasks = GLOBAL_TASK_STATUS.lock().unwrap();
    tasks.values().cloned().collect()
}

/// 获取正在运行的任务
pub fn get_running_tasks() -> Vec<MaaTaskStatus> {
    let tasks = GLOBAL_TASK_STATUS.lock().unwrap();
    tasks.values()
        .filter(|task| task.status == TaskStatus::Running)
        .cloned()
        .collect()
}

/// 清理已完成的旧任务（保留最近24小时的）
pub fn cleanup_old_tasks() {
    let mut tasks = GLOBAL_TASK_STATUS.lock().unwrap();
    let cutoff_time = Utc::now() - chrono::Duration::hours(24);
    
    let old_task_ids: Vec<i32> = tasks.values()
        .filter(|task| task.is_finished() && task.created_at < cutoff_time)
        .map(|task| task.task_id)
        .collect();
    
    let old_count = old_task_ids.len();
    for task_id in &old_task_ids {
        tasks.remove(task_id);
    }
    
    if !old_task_ids.is_empty() {
        // 清理了 {} 个旧任务
    }
}

/// 处理MAA回调事件，更新任务状态
pub fn handle_maa_callback(task_id: i32, msg_code: i32, details: Value) {
    match msg_code {
        // TaskChain 开始
        10001 => {
            start_task(task_id);
        },
        // TaskChain 完成
        10002 => {
            complete_task(task_id, details);
        },
        // TaskChain 错误
        10000 => {
            let error_msg = details.get("what")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown error")
                .to_string();
            fail_task(task_id, error_msg);
        },
        // SubTask 进度更新
        20001 | 20002 | 20003 => {
            if let Some(task_name) = details.get("details").and_then(|d| d.get("task")).and_then(|t| t.as_str()) {
                update_task_progress(task_id, format!("执行子任务: {}", task_name));
            }
        },
        _ => {
            // 其他事件暂时忽略
        }
    }
}