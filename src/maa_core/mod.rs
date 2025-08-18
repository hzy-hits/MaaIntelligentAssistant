//! MAA Core 单例模块
//! 
//! 使用 thread_local 实现线程本地单例，解决 maa_sys::Assistant 不是 Send 的问题
//! 每个线程都有独立的 MAA Core 实例，简化并发访问

use std::path::PathBuf;
use std::cell::RefCell;
use tracing::{info, debug, error, warn};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow};

// 导出子模块
pub mod basic_ops;

// 重新导出基础操作
pub use basic_ops::{
    connect_device, execute_fight, get_maa_status, take_screenshot, perform_click,
    smart_fight, execute_recruit, execute_infrastructure, execute_roguelike,
    execute_copilot, execute_startup, execute_awards
};

/// 线程本地的 MAA Core 单例
/// 由于 maa_sys::Assistant 不是 Send，我们使用线程本地存储
thread_local! {
    static MAA_CORE: RefCell<Option<MaaCore>> = RefCell::new(None);
}

/// 获取或创建当前线程的 MAA Core 实例
pub fn with_maa_core<F, R>(f: F) -> Result<R>
where
    F: FnOnce(&mut MaaCore) -> Result<R>,
{
    MAA_CORE.with(|core_ref| {
        let mut core_opt = core_ref.borrow_mut();
        if core_opt.is_none() {
            debug!("创建新的线程本地MAA Core实例");
            *core_opt = Some(MaaCore::new());
        }
        let core = core_opt.as_mut().unwrap();
        f(core)
    })
}

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
    
    /// 活跃任务ID追踪
    task_counter: i32,
}

impl MaaCore {
    /// 创建新的 MAA Core 实例
    pub fn new() -> Self {
        debug!("创建新的 MaaCore 实例");
        
        Self {
            assistant: None,
            status: MaaStatus::default(),
            resource_path: None,
            task_counter: 0,
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
        
        // 5. 创建 Assistant 实例
        let assistant = maa_sys::Assistant::new(None, None);
        
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
            // PlayCover 连接
            info!("检测到 PlayCover 连接");
            ("", r#"{"touch_mode": "MacPlayTools"}"#)
        } else {
            // ADB 连接
            info!("使用 ADB 连接");
            ("adb", "{}")
        };
        
        // 执行异步连接
        let connection_id = assistant.async_connect(adb_path, address, config, true)
            .map_err(|e| anyhow!("连接失败: {:?}", e))?;
        
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
        
        // 启动任务执行
        assistant.start()
            .map_err(|e| anyhow!("启动任务失败: {:?}", e))?;
        
        // 更新状态
        self.status.active_tasks.push(task_id);
        self.status.running = true;
        self.status.last_updated = Utc::now();
        
        info!("任务执行开始，任务ID: {}", task_id);
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
        
        // 已知路径列表（基于之前的发现）
        #[cfg(target_os = "macos")]
        let known_paths = vec![
            "/Applications/MAA.app/Contents/Frameworks/libMaaCore.dylib",
            "/Users/ivena/Library/Application Support/com.loong.maa/lib/libMaaCore.dylib",
            "/usr/local/lib/libMaaCore.dylib",
            "./libMaaCore.dylib",
        ];
        
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
        if let Ok(path) = std::env::var("MAA_RESOURCE_PATH") {
            return Ok(path);
        }
        
        // 使用项目中的maa-official子模块
        let resource_paths = vec![
            "./maa-official/resource",
            "./resource", 
            "../resource",
            "/Users/ivena/Desktop/Fairy/maa/maa-remote-server/maa-official/resource",
        ];
        
        for path in resource_paths {
            if PathBuf::from(path).exists() {
                return Ok(path.to_string());
            }
        }
        
        warn!("未找到资源文件，使用默认路径");
        Ok("./resource".to_string())
    }
    
    /// 检测是否为 PlayCover 地址
    fn is_playcover_address(&self, address: &str) -> bool {
        address.contains("localhost:1717") || address.contains("127.0.0.1:1717")
    }
    
    /// 获取版本信息
    fn get_version_info(&self) -> Option<String> {
        // 尝试获取MAA版本，如果失败就返回None
        match maa_sys::Assistant::get_version() {
            Ok(version) => Some(version),
            Err(_) => None,
        }
    }
}

impl Drop for MaaCore {
    fn drop(&mut self) {
        if self.status.initialized {
            info!("MAA Core 实例被销毁，清理资源");
            if let Err(e) = self.stop() {
                error!("清理MAA资源时出错: {}", e);
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