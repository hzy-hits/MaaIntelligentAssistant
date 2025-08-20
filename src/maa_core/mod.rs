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
use serde_json;
use anyhow::{Result, anyhow};
use tokio::sync::oneshot;
use once_cell::sync::Lazy;
use crate::config::CONFIG;

// 导出子模块
pub mod basic_ops;
pub mod task_queue;
pub mod worker;
pub mod task_status;
pub mod screenshot;
pub mod task_classification;
pub mod task_notification;

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
        3 => {
            info!("全部任务完成");
            // 通知所有已完成的任务
            if let Some(finished_tasks) = details_json.get("finished_tasks").and_then(|v| v.as_array()) {
                for task in finished_tasks {
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
        5 => {
            info!("MAA实例已销毁");
        },
        
        // TaskChain Info
        10000 => {
            warn!("任务链错误: {}", details_str);
            // 更新任务状态
            if let Some(task_id) = details_json.get("taskid").and_then(|v| v.as_i64()) {
                task_status::handle_maa_callback(task_id as i32, msg, details_json.clone());
            }
        },
        10001 => {
            info!("任务链开始: {}", details_str);
            // 更新任务状态
            if let Some(task_id) = details_json.get("taskid").and_then(|v| v.as_i64()) {
                task_status::handle_maa_callback(task_id as i32, msg, details_json.clone());
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
                task_status::handle_maa_callback(task_id as i32, msg, details_json.clone());
            }
        },
        20001 => {
            debug!("子任务开始: {}", details_str);
            if let Some(task_id) = details_json.get("taskid").and_then(|v| v.as_i64()) {
                task_status::handle_maa_callback(task_id as i32, msg, details_json.clone());
            }
        },
        20002 => {
            debug!("子任务完成: {}", details_str);
            if let Some(task_id) = details_json.get("taskid").and_then(|v| v.as_i64()) {
                task_status::handle_maa_callback(task_id as i32, msg, details_json.clone());
            }
        },
        20003 => {
            debug!("子任务额外信息: {}", details_str);
            if let Some(task_id) = details_json.get("taskid").and_then(|v| v.as_i64()) {
                task_status::handle_maa_callback(task_id as i32, msg, details_json.clone());
            }
        },
        20004 => {
            debug!("子任务手动停止: {}", details_str);
            if let Some(task_id) = details_json.get("taskid").and_then(|v| v.as_i64()) {
                task_status::handle_maa_callback(task_id as i32, msg, details_json.clone());
            }
        },
        
        _ => {
            debug!("未知MAA事件代码: {} - {}", msg, details_str);
        }
    }
}

// 移除了 thread_local 实现
// 现在所有 MAA 操作都通过任务队列路由到专用的工作线程

// 重新导出任务队列相关类型
pub use task_queue::{MaaTask, MaaTaskSender, MaaTaskReceiver, create_maa_task_channel};
pub use worker::MaaWorker;
pub use task_status::{MaaTaskStatus, TaskStatus, get_task_status, get_all_tasks, get_running_tasks, cleanup_old_tasks};
pub use screenshot::{ScreenshotInfo, save_maa_screenshot, get_screenshot_by_id, list_all_screenshots, cleanup_screenshots};
pub use task_classification::{TaskExecutionMode, get_task_execution_mode, estimate_task_duration};
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