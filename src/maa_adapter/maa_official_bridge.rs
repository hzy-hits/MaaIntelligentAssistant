//! MAA官方绑定桥接模块
//! 
//! 提供对MAA官方Rust绑定的统一接口

use std::sync::{Arc, Mutex};
use tracing::{debug, info};
use crate::maa_adapter::{MaaError, MaaResult};

// 当启用 with-maa-core feature 时，使用真实的 MAA 绑定
#[cfg(feature = "with-maa-core")]
mod real_maa {
    pub use maa_sys::{Maa, Error as MaaFFIError};
    
    impl From<MaaFFIError> for crate::maa_adapter::MaaError {
        fn from(err: MaaFFIError) -> Self {
            match err {
                MaaFFIError::Unknown => crate::maa_adapter::MaaError::ffi("unknown", "Unknown MAA FFI error"),
                MaaFFIError::TooLargeAlloc => crate::maa_adapter::MaaError::ffi("alloc", "Too large allocation"),
                MaaFFIError::Null => crate::maa_adapter::MaaError::invalid_parameter("null", "Null error"),
                MaaFFIError::Utf8 => crate::maa_adapter::MaaError::invalid_parameter("utf8", "UTF8 error"),
            }
        }
    }
}

// 当未启用时，使用 stub 实现
#[cfg(not(feature = "with-maa-core"))]
mod stub_maa {
    use std::collections::HashMap;
    
    #[derive(Debug)]
    pub enum MaaFFIError {
        Unknown,
    }
    
    impl std::fmt::Display for MaaFFIError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "MAA FFI Error (stub mode)")
        }
    }
    
    impl From<MaaFFIError> for crate::maa_adapter::MaaError {
        fn from(_: MaaFFIError) -> Self {
            crate::maa_adapter::MaaError::ffi("stub", "MAA stub error")
        }
    }
    
    #[derive(Debug, Clone)]
    pub struct Task {
        pub id: i32,
        pub type_: String,
        pub params: String,
    }
    
    #[derive(Debug)]
    pub struct Maa {
        tasks: HashMap<i32, Task>,
        task_counter: i32,
    }
    
    impl Maa {
        pub fn new() -> Self {
            tracing::warn!("Using MAA stub implementation. Real MAA functions will be mocked.");
            Self {
                tasks: HashMap::new(),
                task_counter: 0,
            }
        }
        
        pub fn with_callback_and_custom_arg(
            _call_back: unsafe extern "C" fn(i32, *const i8, *mut std::os::raw::c_void),
            _custom_arg: *mut std::os::raw::c_void,
        ) -> Self {
            Self::new()
        }
        
        pub fn load_resource(_path: &str) -> Result<(), MaaFFIError> {
            tracing::debug!("Stub: load_resource({})", _path);
            Ok(())
        }
        
        pub fn get_version() -> Result<String, MaaFFIError> {
            Ok("stub-0.1.0".to_string())
        }
        
        pub fn connect(&mut self, _adb_path: &str, _address: &str, _config: Option<&str>) -> Result<i32, MaaFFIError> {
            tracing::debug!("Stub: connect({}, {})", _adb_path, _address);
            Ok(1)
        }
        
        pub fn click(&self, x: i32, y: i32) -> Result<i32, MaaFFIError> {
            tracing::debug!("Stub: click({}, {})", x, y);
            Ok(1)
        }
        
        pub fn screenshot(&self) -> Result<Vec<u8>, MaaFFIError> {
            tracing::debug!("Stub: screenshot()");
            // 返回一个小的假图像数据
            Ok(vec![0u8; 1920 * 1080 * 4])
        }
        
        pub fn create_task(&mut self, type_: &str, params: &str) -> Result<i32, MaaFFIError> {
            self.task_counter += 1;
            let task_id = self.task_counter;
            self.tasks.insert(task_id, Task {
                id: task_id,
                type_: type_.to_string(),
                params: params.to_string(),
            });
            tracing::debug!("Stub: create_task({}, {}) -> {}", type_, params, task_id);
            Ok(task_id)
        }
        
        pub fn start(&self) -> Result<(), MaaFFIError> {
            tracing::debug!("Stub: start()");
            Ok(())
        }
        
        pub fn stop(&self) -> Result<(), MaaFFIError> {
            tracing::debug!("Stub: stop()");
            Ok(())
        }
        
        pub fn get_tasks(&mut self) -> Result<&HashMap<i32, Task>, MaaFFIError> {
            Ok(&self.tasks)
        }
    }
}

// 重新导出统一接口
#[cfg(feature = "with-maa-core")]
pub use real_maa::*;

#[cfg(not(feature = "with-maa-core"))]
pub use stub_maa::*;

/// 线程安全的MAA包装器
pub struct SafeMaaWrapper {
    #[cfg(feature = "with-maa-core")]
    inner: Arc<Mutex<real_maa::Maa>>,
    #[cfg(not(feature = "with-maa-core"))]
    inner: Arc<Mutex<stub_maa::Maa>>,
}

impl SafeMaaWrapper {
    /// 创建新的MAA实例（不带回调）
    pub fn new() -> Self {
        #[cfg(feature = "with-maa-core")]
        {
            info!("Initializing real MAA Core instance");
            Self {
                inner: Arc::new(Mutex::new(real_maa::Maa::new())),
            }
        }
        
        #[cfg(not(feature = "with-maa-core"))]
        {
            info!("Initializing MAA stub instance");
            Self {
                inner: Arc::new(Mutex::new(stub_maa::Maa::new())),
            }
        }
    }
    
    /// 带回调创建MAA实例
    pub fn with_callback_and_custom_arg(
        call_back: unsafe extern "C" fn(i32, *const i8, *mut std::os::raw::c_void),
        custom_arg: *mut std::os::raw::c_void,
    ) -> Self {
        #[cfg(feature = "with-maa-core")]
        {
            info!("Initializing real MAA Core instance with callback");
            Self {
                inner: Arc::new(Mutex::new(real_maa::Maa::with_callback_and_custom_arg(call_back, custom_arg))),
            }
        }
        
        #[cfg(not(feature = "with-maa-core"))]
        {
            info!("Initializing MAA stub instance with callback");
            Self {
                inner: Arc::new(Mutex::new(stub_maa::Maa::with_callback_and_custom_arg(call_back, custom_arg))),
            }
        }
    }
    
    /// 加载资源
    pub fn load_resource(path: &str) -> MaaResult<()> {
        #[cfg(feature = "with-maa-core")]
        {
            real_maa::Maa::load_resource(path)
                .map_err(|e| e.into())
        }
        
        #[cfg(not(feature = "with-maa-core"))]
        {
            stub_maa::Maa::load_resource(path)
                .map_err(|e| e.into())
        }
    }
    
    /// 获取版本
    pub fn get_version() -> MaaResult<String> {
        #[cfg(feature = "with-maa-core")]
        {
            real_maa::Maa::get_version()
                .map_err(|e| e.into())
        }
        
        #[cfg(not(feature = "with-maa-core"))]
        {
            stub_maa::Maa::get_version()
                .map_err(|e| e.into())
        }
    }
    
    /// 连接设备
    pub fn connect_safe(&self, adb_path: &str, address: &str, config: Option<&str>) -> MaaResult<i32> {
        let mut maa = self.inner.lock()
            .map_err(|_| MaaError::ffi("lock", "Failed to acquire mutex lock"))?;
        maa.connect(adb_path, address, config)
            .map_err(|e| e.into())
    }
    
    /// 点击
    pub fn click(&self, x: i32, y: i32) -> MaaResult<i32> {
        let maa = self.inner.lock()
            .map_err(|_| MaaError::ffi("lock", "Failed to acquire mutex lock"))?;
        maa.click(x, y)
            .map_err(|e| e.into())
    }
    
    /// 截图
    pub fn screenshot(&self) -> MaaResult<Vec<u8>> {
        let maa = self.inner.lock()
            .map_err(|_| MaaError::ffi("lock", "Failed to acquire mutex lock"))?;
        maa.screenshot()
            .map_err(|e| e.into())
    }
    
    /// 创建任务
    pub fn create_task(&self, type_: &str, params: &str) -> MaaResult<i32> {
        let mut maa = self.inner.lock()
            .map_err(|_| MaaError::ffi("lock", "Failed to acquire mutex lock"))?;
        maa.create_task(type_, params)
            .map_err(|e| e.into())
    }
    
    /// 开始执行
    pub fn start(&self) -> MaaResult<()> {
        let maa = self.inner.lock()
            .map_err(|_| MaaError::ffi("lock", "Failed to acquire mutex lock"))?;
        maa.start()
            .map_err(|e| e.into())
    }
    
    /// 停止执行
    pub fn stop(&self) -> MaaResult<()> {
        let maa = self.inner.lock()
            .map_err(|_| MaaError::ffi("lock", "Failed to acquire mutex lock"))?;
        maa.stop()
            .map_err(|e| e.into())
    }
}

impl Drop for SafeMaaWrapper {
    fn drop(&mut self) {
        debug!("Dropping SafeMaaWrapper");
    }
}

// 线程安全标记
unsafe impl Send for SafeMaaWrapper {}
unsafe impl Sync for SafeMaaWrapper {}