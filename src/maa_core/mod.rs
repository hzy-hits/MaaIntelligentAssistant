//! MAA Core 单例模块
//! 
//! 使用 thread_local 实现线程本地单例，解决 maa_sys::Assistant 不是 Send 的问题
//! 每个线程都有独立的 MAA Core 实例，简化并发访问

use std::path::PathBuf;
use std::os::raw::{c_char, c_void};
use std::ffi::CStr;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tracing::{info, debug, warn};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_json::{self, Value};
use anyhow::{Result, anyhow};
use tokio::sync::{oneshot, broadcast};
use once_cell::sync::Lazy;
use crate::config::CONFIG;
use crate::maa_core::worker_v2::TaskProgressEvent;

// 导出子模块
pub mod basic_ops;
pub mod task_status;
pub mod screenshot;
pub mod task_notification;

// V2 优化模块 - 简化架构
pub mod task_classification_v2;
pub mod task_queue_v2;
pub mod worker_v2;

/// 全局任务完成通知器
/// task_id -> oneshot sender
static GLOBAL_TASK_NOTIFIERS: Lazy<Arc<Mutex<HashMap<i32, oneshot::Sender<serde_json::Value>>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// 注册任务通知器
pub fn register_task_notifier(task_id: i32, sender: oneshot::Sender<serde_json::Value>) {
    let mut notifiers = GLOBAL_TASK_NOTIFIERS.lock().unwrap();
    notifiers.insert(task_id, sender);
    debug!("注册任务通知器: task_id={}", task_id);
}

/// 触发任务完成通知
pub fn notify_task_completion(task_id: i32, result: serde_json::Value) {
    let mut notifiers = GLOBAL_TASK_NOTIFIERS.lock().unwrap();
    if let Some(sender) = notifiers.remove(&task_id) {
        let _ = sender.send(result);
        info!("任务完成通知已发送: task_id={}", task_id);
    }
}

// 重新导出基础操作
pub use basic_ops::{
    connect_device, execute_fight, get_maa_status, take_screenshot, perform_click,
    smart_fight, execute_recruit, execute_infrastructure, execute_roguelike,
    execute_copilot, execute_startup, execute_awards, execute_credit_store,
    execute_depot_management, execute_operator_box, execute_sss_copilot,
    execute_reclamation, execute_closedown, execute_custom_task,
    execute_video_recognition, execute_system_management
};

/// 全局SSE事件广播器，用于MAA回调到Worker V2的通信
static mut GLOBAL_SSE_BROADCASTER: Option<broadcast::Sender<TaskProgressEvent>> = None;

/// 设置全局SSE广播器（由Worker V2调用）
pub fn set_global_sse_broadcaster(broadcaster: broadcast::Sender<TaskProgressEvent>) {
    unsafe {
        GLOBAL_SSE_BROADCASTER = Some(broadcaster);
    }
    info!("已设置全局SSE广播器用于MAA回调转发");
}

/// MAA 回调函数 - 处理任务完成事件 (遵循官方协议)
unsafe extern "C" fn maa_callback(
    msg: i32,
    details_raw: *const c_char,
    _arg: *mut c_void,
) {
    // 安全地转换C字符串
    let details_str = if details_raw.is_null() {
        "{}".to_string()
    } else {
        CStr::from_ptr(details_raw)
            .to_string_lossy()
            .to_string()
    };
    
    // 解析JSON详情
    let details_json: serde_json::Value = match serde_json::from_str(&details_str) {
        Ok(json) => json,
        Err(_) => {
            warn!("MAA回调JSON解析失败: {}", details_str);
            return;
        }
    };
    
    // 记录MAA事件
    info!("MAA回调事件: {} | JSON: {}", msg, details_str);
    
    // 处理重要事件 - 使用官方协议的消息代码
    match msg {
        // Global Info
        0 => {
            warn!("MAA内部错误: {}", details_str);
        },
        1 => {
            warn!("MAA初始化失败: {}", details_str);
        },
        2 => {
            // ConnectionInfo - 关键的连接事件处理
            if let Some(what) = details_json.get("what").and_then(|v| v.as_str()) {
                match what {
                    "ConnectFailed" => {
                        let why = details_json.get("why").and_then(|v| v.as_str()).unwrap_or("unknown");
                        warn!("连接失败: {} - 详情: {}", why, details_str);
                        // 不要因为连接失败就退出，这是正常的重试流程
                    },
                    "Connected" => {
                        info!("设备连接成功");
                    },
                    "UuidGot" => {
                        info!("获取设备UUID成功");
                    },
                    _ => {
                        debug!("连接信息: {} - {}", what, details_str);
                    }
                }
            }
        },
        3 => { // AllTasksCompleted
            info!("全部任务完成: {}", details_str);
            
            // 提取任务链信息
            let taskchain = details_json.get("taskchain")
                .and_then(|tc| tc.as_str())
                .unwrap_or("unknown");
            let finished_tasks = details_json.get("finished_tasks")
                .and_then(|ft| ft.as_array())
                .map(|arr| arr.len())
                .unwrap_or(0);
                
            // SSE通知
            forward_to_sse_global("all_tasks_completed", 
                                 format!("任务链 {} 全部完成，共执行{}个任务", taskchain, finished_tasks), 
                                 details_json.clone());
            
            // 通知所有已完成的任务
            if let Some(finished_tasks_array) = details_json.get("finished_tasks").and_then(|v| v.as_array()) {
                for task in finished_tasks_array {
                    if let Some(task_id) = task.as_i64() {
                        notify_task_completion(task_id as i32, details_json.clone());
                    }
                }
            }
        },
        4 => {
            // AsyncCallInfo - 异步调用信息
            debug!("异步调用信息: {}", details_str);
        },
        5 => { // Destroyed
            warn!("MAA实例已销毁: {}", details_str);
            forward_to_sse_global("maa_destroyed", 
                                 "MAA实例已销毁，需要重新初始化".to_string(), 
                                 details_json.clone());
        },
        
        // TaskChain Info
        10000 => {
            warn!("任务链错误: {}", details_str);
            // 更新任务状态
            if let Some(task_id) = details_json.get("taskid").and_then(|v| v.as_i64()) {
                let task_id = task_id as i32;
                task_status::handle_maa_callback(task_id, msg, details_json.clone());
                
                // 转发到SSE系统
                forward_to_sse(task_id, msg, details_json.clone());
            }
        },
        10001 => {
            info!("任务链开始: {}", details_str);
            // 更新任务状态
            if let Some(task_id) = details_json.get("taskid").and_then(|v| v.as_i64()) {
                let task_id = task_id as i32;
                task_status::handle_maa_callback(task_id, msg, details_json.clone());
                
                // 转发到SSE系统
                forward_to_sse(task_id, msg, details_json.clone());
            }
        },
        10002 => {
            info!("任务链完成: {}", details_str);
            // 更新任务状态和通知oneshot channel
            if let Some(task_id) = details_json.get("taskid").and_then(|v| v.as_i64()) {
                task_status::handle_maa_callback(task_id as i32, msg, details_json.clone());
                notify_task_completion(task_id as i32, details_json.clone());
            }
        },
        10003 => {
            debug!("任务链额外信息: {}", details_str);
        },
        10004 => {
            warn!("任务链手动停止: {}", details_str);
        },
        
        // SubTask Info
        20000 => {
            warn!("子任务错误: {}", details_str);
            if let Some(task_id) = details_json.get("taskid").and_then(|v| v.as_i64()) {
                let task_id = task_id as i32;
                task_status::handle_maa_callback(task_id, msg, details_json.clone());
                
                // 转发到SSE系统
                forward_to_sse(task_id, msg, details_json.clone());
            }
        },
        20001 => {
            debug!("子任务开始: {}", details_str);
            if let Some(task_id) = details_json.get("taskid").and_then(|v| v.as_i64()) {
                let task_id = task_id as i32;
                task_status::handle_maa_callback(task_id, msg, details_json.clone());
                
                // 转发到SSE系统
                forward_to_sse(task_id, msg, details_json.clone());
            }
        },
        20002 => {
            debug!("子任务完成: {}", details_str);
            if let Some(task_id) = details_json.get("taskid").and_then(|v| v.as_i64()) {
                let task_id = task_id as i32;
                task_status::handle_maa_callback(task_id, msg, details_json.clone());
                
                // 转发到SSE系统
                forward_to_sse(task_id, msg, details_json.clone());
            }
        },
        20003 => {
            debug!("子任务额外信息: {}", details_str);
            if let Some(task_id) = details_json.get("taskid").and_then(|v| v.as_i64()) {
                let task_id = task_id as i32;
                task_status::handle_maa_callback(task_id, msg, details_json.clone());
                
                // 转发到SSE系统
                forward_to_sse(task_id, msg, details_json.clone());
            }
        },
        20004 => {
            debug!("子任务手动停止: {}", details_str);
            if let Some(task_id) = details_json.get("taskid").and_then(|v| v.as_i64()) {
                let task_id = task_id as i32;
                task_status::handle_maa_callback(task_id, msg, details_json.clone());
                
                // 转发到SSE系统
                forward_to_sse(task_id, msg, details_json.clone());
            }
        },
        
        _ => {
            debug!("未知MAA事件代码: {} - {}", msg, details_str);
        }
    }
}

/// 将MAA回调事件转发到SSE系统
fn forward_to_sse(task_id: i32, msg_code: i32, details: Value) {
    unsafe {
        if let Some(ref broadcaster) = GLOBAL_SSE_BROADCASTER {
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
            
            let task_name = details.get("details")
                .and_then(|d| d.get("task"))
                .and_then(|t| t.as_str())
                .unwrap_or("unknown");
                
            let task_chain = details.get("taskchain")
                .and_then(|tc| tc.as_str())
                .unwrap_or("unknown");
                
            // 根据MAA Core官方协议生成用户友好的消息
            let message = match msg_code {
                10001 => {
                    let task_desc = get_taskchain_description(task_chain);
                    format!("{} 开始执行", task_desc)
                },
                10002 => {
                    let task_desc = get_taskchain_description(task_chain);
                    format!("{} 执行完成", task_desc)
                },
                10000 => {
                    let task_desc = get_taskchain_description(task_chain);
                    format!("{} 执行失败", task_desc)
                },
                20001 => {
                    let task_desc = get_subtask_description(task_name);
                    format!("开始: {}", task_desc)
                },
                20002 => {
                    let task_desc = get_subtask_description(task_name);
                    format!("完成: {}", task_desc)
                },
                20003 => {
                    // 子任务额外信息，包括掉落、设施信息等
                    if let Some(facility) = details.get("details").and_then(|d| d.get("facility")).and_then(|f| f.as_str()) {
                        format!("处理基建设施: {}", get_facility_name(facility))
                    } else if let Some(what) = details.get("what").and_then(|w| w.as_str()) {
                        match what {
                            "StageDrops" => "获得关卡掉落奖励".to_string(),
                            "RecruitTagsDetected" => "检测到公招标签".to_string(),
                            "RecruitSpecialTag" => "发现特殊公招标签".to_string(),
                            "ProductOfFacility" => "收集基建产物".to_string(),
                            _ => format!("{} 信息更新", get_subtask_description(task_name))
                        }
                    } else {
                        format!("{} 进度更新", get_subtask_description(task_name))
                    }
                },
                20000 => {
                    let task_desc = get_subtask_description(task_name);
                    format!("失败: {}", task_desc)
                },
                20004 => {
                    let task_desc = get_subtask_description(task_name);
                    format!("停止: {}", task_desc)
                },
                _ => format!("未知事件: {}", task_name)
            };
            
            let sse_event = TaskProgressEvent {
                task_id,
                task_type: task_chain.to_string(),
                event_type: event_type.to_string(),
                message,
                data: Some(details),
                timestamp: Utc::now(),
            };
            
            // 发送SSE事件（忽略发送失败，避免阻塞）
            let _ = broadcaster.send(sse_event);
            
            debug!("MAA回调事件已转发到SSE: task_id={}, event_type={}, msg_code={}", task_id, event_type, msg_code);
        } else {
            debug!("SSE广播器未设置，跳过MAA回调转发");
        }
    }
}

/// 转发全局事件到SSE系统
fn forward_to_sse_global(event_type: &str, message: String, details: Value) {
    unsafe {
        if let Some(ref broadcaster) = GLOBAL_SSE_BROADCASTER {
            let sse_event = TaskProgressEvent {
                task_id: 0, // 全局事件使用task_id=0
                task_type: "system".to_string(),
                event_type: event_type.to_string(),
                message,
                data: Some(details),
                timestamp: Utc::now(),
            };
            
            let _ = broadcaster.send(sse_event);
            debug!("全局事件已转发到SSE: event_type={}", event_type);
        } else {
            debug!("SSE广播器未设置，跳过全局事件转发");
        }
    }
}

/// 获取任务链的中文描述
fn get_taskchain_description(taskchain: &str) -> String {
    match taskchain {
        "StartUp" => "游戏启动".to_string(),
        "CloseDown" => "游戏关闭".to_string(),
        "Fight" => "自动战斗".to_string(),
        "Mall" => "信用商店".to_string(),
        "Recruit" => "公开招募".to_string(),
        "Infrast" => "基建管理".to_string(),
        "Award" => "奖励收集".to_string(),
        "Roguelike" => "集成战略(肉鸽)".to_string(),
        "Copilot" => "作业执行".to_string(),
        "SSSCopilot" => "保全派驻".to_string(),
        "Depot" => "仓库识别".to_string(),
        "OperBox" => "干员箱识别".to_string(),
        "Reclamation" => "生息演算".to_string(),
        "Custom" => "自定义任务".to_string(),
        "SingleStep" => "单步任务".to_string(),
        "VideoRecognition" => "视频识别".to_string(),
        "Debug" => "调试模式".to_string(),
        _ => format!("任务: {}", taskchain)
    }
}

/// 获取子任务的中文描述
fn get_subtask_description(subtask: &str) -> String {
    match subtask {
        // 战斗相关
        "StartButton2" => "开始战斗".to_string(),
        "MedicineConfirm" => "使用理智药".to_string(),
        "ExpiringMedicineConfirm" => "使用过期理智药".to_string(),
        "StoneConfirm" => "使用源石补充理智".to_string(),
        
        // 公招相关
        "RecruitRefreshConfirm" => "公招刷新标签".to_string(),
        "RecruitConfirm" => "确认公招".to_string(),
        "RecruitNowConfirm" => "使用加急许可证".to_string(),
        
        // 基建相关
        "InfrastDormDoubleConfirmButton" => "宿舍干员冲突确认".to_string(),
        "InfrastEnteredFlag" => "进入基建界面".to_string(),
        "InfrastEnterOperList" => "进入干员列表".to_string(),
        "ReturnButton" => "返回上级菜单".to_string(),
        "SwipeToTheLeft" => "向左滑动切换".to_string(),
        
        // 肉鸽相关
        "StartExplore" => "开始肉鸽探索".to_string(),
        "StageTraderInvestConfirm" => "投资源石锭".to_string(),
        "MissionCompletedFlag" => "战斗胜利".to_string(),
        "MissionFailedFlag" => "战斗失败".to_string(),
        "ExitThenAbandon" => "放弃探索".to_string(),
        
        // 其他
        "StartGameTask" => "启动游戏客户端".to_string(),
        "ReportToPenguinStats" => "上报企鹅数据".to_string(),
        "ReportToYituliu" => "上报一图流数据".to_string(),
        
        _ => subtask.to_string()
    }
}

/// 获取基建设施的中文名称
fn get_facility_name(facility: &str) -> String {
    match facility {
        "Mfg" => "制造站".to_string(),
        "Trade" => "贸易站".to_string(),
        "Power" => "发电站".to_string(),
        "Control" => "控制中枢".to_string(),
        "Reception" => "会客室".to_string(),
        "Office" => "办公室".to_string(),
        "Dorm" => "宿舍".to_string(),
        _ => facility.to_string()
    }
}

// V2架构：简化的任务队列系统
// 所有 MAA 操作都通过V2优化任务队列路由到专用的工作线程

// V2组件导出
pub use task_queue_v2::{
    MaaTask as MaaTaskV2, MaaTaskSender as MaaTaskSenderV2, MaaTaskReceiver as MaaTaskReceiverV2, 
    create_maa_task_channel_v2, TaskResult, TaskStatus as TaskStatusV2
};
pub use worker_v2::MaaWorkerV2;
pub use task_status::{MaaTaskStatus, TaskStatus, get_task_status, get_all_tasks, get_running_tasks, cleanup_old_tasks};
pub use screenshot::{ScreenshotInfo, save_maa_screenshot, get_screenshot_by_id, list_all_screenshots, cleanup_screenshots};
pub use task_classification_v2::{TaskExecutionMode, get_task_execution_mode, estimate_task_duration, is_synchronous_task};
pub use task_notification::{
    TaskStatusEvent, TaskStatus as NotificationTaskStatus, init_task_notification_system, 
    subscribe_task_events, notify_task_status, notify_task_started, notify_task_progress, 
    notify_task_completed, notify_task_failed, TaskStatusMonitor
};

/// MAA 状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaaStatus {
    /// 是否已初始化
    pub initialized: bool,
    /// 是否已连接设备
    pub connected: bool,
    /// 设备地址
    pub device_address: Option<String>,
    /// 是否正在运行任务
    pub running: bool,
    /// 当前任务列表
    pub active_tasks: Vec<i32>,
    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
    /// 版本信息
    pub version: Option<String>,
}

impl Default for MaaStatus {
    fn default() -> Self {
        Self {
            initialized: false,
            connected: false,
            device_address: None,
            running: false,
            active_tasks: Vec::new(),
            last_updated: Utc::now(),
            version: None,
        }
    }
}

/// 简化的 MAA Core 封装
pub struct MaaCore {
    /// MAA Assistant 实例
    assistant: Option<maa_sys::Assistant>,
    
    /// 当前状态
    status: MaaStatus,
    
    /// 资源路径
    resource_path: Option<String>,
    
}

impl MaaCore {
    /// 创建新的 MAA Core 实例
    pub fn new() -> Self {
        debug!("创建新的 MaaCore 实例");
        
        Self {
            assistant: None,
            status: MaaStatus::default(),
            resource_path: None,
        }
    }
    
    /// 初始化 MAA（加载库和资源）
    pub fn initialize(&mut self) -> Result<()> {
        if self.status.initialized {
            debug!("MAA 已经初始化，跳过");
            return Ok(());
        }
        
        info!("开始初始化 MAA Core");
        
        // 1. 查找 MAA Core 库文件
        let lib_path = self.find_maa_core_library()?;
        info!("找到 MAA Core 库: {}", lib_path.display());
        
        // 2. 加载库
        maa_sys::Assistant::load(&lib_path)
            .map_err(|e| anyhow!("加载 MAA Core 库失败: {:?}", e))?;
        
        // 3. 设置资源路径
        let resource_path = self.find_resource_path()?;
        info!("使用资源路径: {}", resource_path);
        
        // 4. 加载资源
        maa_sys::Assistant::load_resource(resource_path.as_str())
            .map_err(|e| anyhow!("加载 MAA 资源失败: {:?}", e))?;
        
        // 5. 创建 Assistant 实例 - 带回调处理
        let assistant = maa_sys::Assistant::new(Some(maa_callback), None);
        
        // 5.1. 为PlayCover预设TouchMode（必须在连接前设置）
        info!("预设TouchMode为{}以支持PlayCover", CONFIG.device.touch_mode_playcover);
        if let Err(e) = assistant.set_instance_option(maa_sys::InstanceOptionKey::TouchMode, CONFIG.device.touch_mode_playcover.as_str()) {
            warn!("预设TouchMode失败，继续初始化: {:?}", e);
        } else {
            info!("TouchMode预设为{}成功", CONFIG.device.touch_mode_playcover);
        }
        
        // 6. 获取版本信息
        let version = self.get_version_info();
        
        // 7. 更新状态
        self.assistant = Some(assistant);
        self.resource_path = Some(resource_path);
        self.status.initialized = true;
        self.status.version = version;
        self.status.last_updated = Utc::now();
        
        info!("MAA Core 初始化完成");
        Ok(())
    }
    
    /// 连接到设备
    pub fn connect(&mut self, address: &str) -> Result<i32> {
        // 确保已初始化
        if !self.status.initialized {
            self.initialize()?;
        }
        
        let assistant = self.assistant.as_mut()
            .ok_or_else(|| anyhow!("MAA Assistant 未初始化"))?;
        
        info!("连接到设备: {}", address);
        
        // 检测连接类型
        let is_playcover = address.contains("localhost:1717") || address.contains("127.0.0.1:1717");
        let (adb_path, config) = if is_playcover {
            // PlayCover 连接 - TouchMode已在初始化时设置
            info!("检测到 PlayCover 连接，使用预设的MacPlayTools配置");
            ("", "{}")
        } else {
            // ADB 连接
            info!("使用 ADB 连接");
            ("adb", "{}")
        };
        
        // 执行异步连接
        let connection_id = assistant.async_connect(adb_path, address, config, true)
            .map_err(|e| {
                if is_playcover {
                    anyhow!("PlayCover连接失败: {:?}\n请检查:\n1. PlayCover是否已安装明日方舟\n2. MaaTools是否已启用\n3. 游戏是否正在运行", e)
                } else {
                    anyhow!("ADB连接失败: {:?}\n请检查设备连接和ADB配置", e)
                }
            })?;
        
        // 更新状态
        self.status.connected = true;
        self.status.device_address = Some(address.to_string());
        self.status.last_updated = Utc::now();
        
        info!("成功连接到设备，连接ID: {}", connection_id);
        Ok(connection_id)
    }
    
    /// 执行任务
    pub fn execute_task(&mut self, task_type: &str, params: &str) -> Result<i32> {
        let assistant = self.assistant.as_mut()
            .ok_or_else(|| anyhow!("MAA Assistant 未初始化"))?;
        
        debug!("执行任务: {} with params: {}", task_type, params);
        
        // 创建任务
        let task_id = assistant.append_task(task_type, params)
            .map_err(|e| anyhow!("创建任务失败: {:?}", e))?;
        
        // 异步启动任务执行
        info!("任务已添加到队列，任务ID: {}", task_id);
        
        // 启动任务执行（非阻塞）
        match assistant.start() {
            Ok(_) => {
                info!("任务执行启动成功，任务ID: {}", task_id);
            },
            Err(e) => {
                warn!("任务启动失败但继续: {:?}", e);
                // 不直接返回错误，因为任务可能已经在队列中
            }
        }
        
        // 更新状态
        self.status.active_tasks.push(task_id);
        self.status.running = true;
        self.status.last_updated = Utc::now();
        
        info!("任务已提交，任务ID: {}", task_id);
        Ok(task_id)
    }
    
    /// 获取状态
    pub fn get_status(&mut self) -> MaaStatus {
        if let Some(assistant) = &self.assistant {
            // 更新运行状态
            self.status.running = assistant.running();
            self.status.connected = assistant.connected();
        }
        
        self.status.last_updated = Utc::now();
        self.status.clone()
    }
    
    /// 截图
    pub fn screenshot(&self) -> Result<Vec<u8>> {
        let assistant = self.assistant.as_ref()
            .ok_or_else(|| anyhow!("MAA Assistant 未初始化"))?;
        
        debug!("执行截图操作");
        
        let image_data = assistant.get_image()
            .map_err(|e| anyhow!("截图失败: {:?}", e))?;
        
        info!("截图完成，数据大小: {} bytes", image_data.len());
        Ok(image_data)
    }
    
    /// 点击操作
    pub fn click(&self, x: i32, y: i32) -> Result<i32> {
        let assistant = self.assistant.as_ref()
            .ok_or_else(|| anyhow!("MAA Assistant 未初始化"))?;
        
        debug!("执行点击操作: ({}, {})", x, y);
        
        let click_id = assistant.async_click(x, y, true)
            .map_err(|e| anyhow!("点击失败: {:?}", e))?;
        
        info!("点击操作完成，点击ID: {}", click_id);
        Ok(click_id)
    }
    
    /// 停止所有任务
    pub fn stop(&mut self) -> Result<()> {
        if let Some(assistant) = &mut self.assistant {
            assistant.stop()
                .map_err(|e| anyhow!("停止任务失败: {:?}", e))?;
            
            // 清空任务列表
            self.status.active_tasks.clear();
            self.status.running = false;
            self.status.last_updated = Utc::now();
            
            info!("已停止所有MAA任务");
        }
        
        Ok(())
    }
    
    // 私有辅助方法
    
    /// 查找 MAA Core 库文件
    fn find_maa_core_library(&self) -> Result<PathBuf> {
        // 从环境变量获取
        if let Ok(path) = std::env::var("MAA_CORE_LIB") {
            let path_buf = PathBuf::from(path);
            if path_buf.exists() {
                return Ok(path_buf);
            }
        }
        
        // 从配置文件获取备用路径
        #[cfg(target_os = "macos")]
        let known_paths = CONFIG.maa.fallback_lib_paths.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
        
        #[cfg(target_os = "linux")]
        let known_paths = vec![
            "/usr/local/lib/libMaaCore.so",
            "/usr/lib/libMaaCore.so",
            "./libMaaCore.so",
        ];
        
        #[cfg(target_os = "windows")]
        let known_paths = vec![
            "C:\\MAA\\MaaCore.dll",
            ".\\MaaCore.dll",
        ];
        
        for path in known_paths {
            let path_buf = PathBuf::from(path);
            if path_buf.exists() {
                info!("找到 MAA Core 库: {}", path_buf.display());
                return Ok(path_buf);
            }
        }
        
        Err(anyhow!("未找到 MAA Core 库文件。请设置 MAA_CORE_LIB 环境变量或安装 MAA.app"))
    }
    
    /// 查找资源路径
    fn find_resource_path(&self) -> Result<String> {
        // 从环境变量获取
        if let Ok(path) = std::env::var(&CONFIG.env_keys.resource_path) {
            info!("使用环境变量资源路径: {}", path);
            return Ok(path);
        }
        
        info!("未找到环境变量{}，使用备用路径", CONFIG.env_keys.resource_path);
        
        // 从配置文件获取备用资源路径
        let resource_paths = &CONFIG.maa.fallback_resource_paths;
        
        for path in resource_paths {
            if PathBuf::from(path).exists() {
                info!("找到备用资源路径: {}", path);
                return Ok(path.clone());
            }
        }
        
        warn!("未找到资源文件，使用默认路径");
        Ok(CONFIG.maa.default_resource_path.clone())
    }
    
    
    /// 获取版本信息
    fn get_version_info(&self) -> Option<String> {
        // 尝试获取MAA版本，如果失败就返回None
        match maa_sys::Assistant::get_version() {
            Ok(version) => Some(version),
            Err(_) => None,
        }
    }
    
    /// 检查是否已初始化
    pub fn is_initialized(&self) -> bool {
        self.status.initialized
    }
    
    /// 检查是否已连接设备
    pub fn is_connected(&self) -> bool {
        self.status.connected
    }
    
    /// 获取当前状态的只读引用
    pub fn get_status_ref(&self) -> &MaaStatus {
        &self.status
    }
}

impl Drop for MaaCore {
    fn drop(&mut self) {
        if self.status.initialized {
            info!("MAA Core 实例被销毁，安全清理资源");
            // 安全地停止任务，不传播错误
            if let Some(assistant) = &mut self.assistant {
                match assistant.stop() {
                    Ok(_) => info!("MAA任务已安全停止"),
                    Err(e) => warn!("停止MAA任务时出现警告(忽略): {:?}", e),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_maa_core_creation() {
        let core = MaaCore::new();
        assert!(!core.status.initialized);
        assert!(!core.status.connected);
        assert_eq!(core.task_counter, 0);
    }
    
    #[test]
    fn test_playcover_address_detection() {
        let core = MaaCore::new();
        
        assert!(core.is_playcover_address("localhost:1717"));
        assert!(core.is_playcover_address("127.0.0.1:1717"));
        assert!(!core.is_playcover_address("192.168.1.100:5555"));
        assert!(!core.is_playcover_address("emulator-5554"));
    }
    
    #[test] 
    fn test_status_default() {
        let status = MaaStatus::default();
        assert!(!status.initialized);
        assert!(!status.connected);
        assert!(status.device_address.is_none());
        assert!(!status.running);
        assert!(status.active_tasks.is_empty());
    }
}