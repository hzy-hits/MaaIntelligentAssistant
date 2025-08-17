//! Real MAA FFI implementation using maa-sys
//!
//! This module provides the actual MAA Core integration using the official maa-sys crate.
//! It handles initialization, resource loading, connection management, and task execution.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::ffi::c_void;

use tracing::{debug, info, warn, error};
use chrono::Utc;

use crate::maa_adapter::{MaaError, MaaResult, CallbackMessage};

// Use the real maa-sys Assistant
use maa_sys::Assistant;
use maa_types::{InstanceOptionKey, StaticOptionKey};

/// Real MAA FFI implementation
pub struct MaaFFIReal {
    /// MAA Assistant instance
    assistant: Assistant,
    
    /// Resource path
    resource_path: String,
    
    /// Connection parameters
    connection_params: Option<ConnectionParams>,
    
    /// Callback sender for async message handling
    callback_sender: Option<tokio::sync::mpsc::UnboundedSender<CallbackMessage>>,
    
    /// Active tasks tracking
    active_tasks: Arc<Mutex<HashMap<i32, String>>>,
    
    /// Device UUID
    device_uuid: Option<String>,
}

/// Connection parameters for MAA
#[derive(Debug, Clone)]
pub struct ConnectionParams {
    pub adb_path: String,
    pub device_address: String,
    pub config: Option<String>,
}

impl MaaFFIReal {
    /// Create a new MAA FFI instance
    pub fn new(resource_path: String) -> MaaResult<Self> {
        debug!("Creating real MAA FFI instance with resource: {}", resource_path);
        
        // Load MaaCore library first
        let library_path = Self::get_maa_core_path()?;
        info!("Loading MaaCore from: {}", library_path.display());
        
        Assistant::load(&library_path)
            .map_err(|e| MaaError::ffi("load_library", format!("Failed to load MaaCore: {:?}", e)))?;
        
        // Load resource
        info!("Loading MAA resource from: {}", resource_path);
        Assistant::load_resource(resource_path.as_str())
            .map_err(|e| MaaError::ffi("load_resource", format!("Failed to load resource: {:?}", e)))?;
        
        // Create Assistant instance without callback
        let assistant = Assistant::new(None, None);
        
        info!("MAA FFI instance created successfully");
        
        Ok(Self {
            assistant,
            resource_path,
            connection_params: None,
            callback_sender: None,
            active_tasks: Arc::new(Mutex::new(HashMap::new())),
            device_uuid: None,
        })
    }
    
    /// Create MAA FFI instance with callback support
    pub fn with_callback(
        resource_path: String,
        callback_sender: tokio::sync::mpsc::UnboundedSender<CallbackMessage>,
    ) -> MaaResult<Self> {
        debug!("Creating real MAA FFI instance with callback support");
        
        // Load MaaCore library
        let library_path = Self::get_maa_core_path()?;
        info!("Loading MaaCore from: {}", library_path.display());
        
        Assistant::load(&library_path)
            .map_err(|e| MaaError::ffi("load_library", format!("Failed to load MaaCore: {:?}", e)))?;
        
        // Load resource
        info!("Loading MAA resource from: {}", resource_path);
        Assistant::load_resource(resource_path.as_str())
            .map_err(|e| MaaError::ffi("load_resource", format!("Failed to load resource: {:?}", e)))?;
        
        // Create callback sender box for FFI
        let sender_box = Box::new(callback_sender.clone());
        let sender_ptr = Box::into_raw(sender_box) as *mut c_void;
        
        // Create Assistant instance with callback
        let assistant = Assistant::new(Some(maa_callback_bridge), Some(sender_ptr));
        
        info!("MAA FFI instance created successfully with callback");
        
        Ok(Self {
            assistant,
            resource_path,
            connection_params: None,
            callback_sender: Some(callback_sender),
            active_tasks: Arc::new(Mutex::new(HashMap::new())),
            device_uuid: None,
        })
    }
    
    /// Get MaaCore library path
    fn get_maa_core_path() -> MaaResult<std::path::PathBuf> {
        // Try known paths based on platform
        #[cfg(target_os = "macos")]
        let known_paths = vec![
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
            let path_buf = std::path::PathBuf::from(path);
            if path_buf.exists() {
                return Ok(path_buf);
            }
        }
        
        Err(MaaError::configuration("maa_core_path", "MaaCore library not found in known locations"))
    }
    
    /// Connect to device
    pub fn connect(&mut self, adb_path: &str, device_address: &str, config: Option<&str>) -> MaaResult<i32> {
        debug!("Connecting to device: {} via {}", device_address, adb_path);
        
        let config_str = config.unwrap_or("{}");
        
        let connect_id = self.assistant.async_connect(adb_path, device_address, config_str, true)
            .map_err(|e| MaaError::ffi("async_connect", format!("Failed to connect: {:?}", e)))?;
        
        // Store connection parameters
        self.connection_params = Some(ConnectionParams {
            adb_path: adb_path.to_string(),
            device_address: device_address.to_string(),
            config: config.map(|s| s.to_string()),
        });
        
        info!("Connection initiated with ID: {}", connect_id);
        Ok(connect_id)
    }
    
    /// Check if MAA is currently running
    pub fn is_running(&self) -> bool {
        self.assistant.running()
    }
    
    /// Check if connected to device
    pub fn is_connected(&self) -> bool {
        self.assistant.connected()
    }
    
    /// Take a screenshot
    pub fn screenshot(&self) -> MaaResult<Vec<u8>> {
        debug!("Taking screenshot via real MAA FFI");
        
        // Take fresh screenshot
        self.assistant.get_fresh_image()
            .map_err(|e| MaaError::ffi("get_fresh_image", format!("Failed to take screenshot: {:?}", e)))
    }
    
    /// Click at specified coordinates
    pub fn click(&self, x: i32, y: i32) -> MaaResult<i32> {
        debug!("Clicking at ({}, {}) via real MAA FFI", x, y);
        
        self.assistant.async_click(x, y, true)
            .map_err(|e| MaaError::ffi("async_click", format!("Failed to click: {:?}", e)))
    }
    
    /// Create a new task
    pub fn create_task(&mut self, task_type: &str, params: &str) -> MaaResult<i32> {
        debug!("Creating task: {} with params: {}", task_type, params);
        
        let task_id = self.assistant.append_task(task_type, params)
            .map_err(|e| MaaError::ffi("append_task", format!("Failed to create task: {:?}", e)))?;
        
        // Track the task
        if let Ok(mut tasks) = self.active_tasks.lock() {
            tasks.insert(task_id, task_type.to_string());
        }
        
        info!("Task created with ID: {}", task_id);
        Ok(task_id)
    }
    
    /// Set task parameters
    pub fn set_task_params(&self, task_id: i32, params: &str) -> MaaResult<()> {
        debug!("Setting params for task {}: {}", task_id, params);
        
        self.assistant.set_task_params(task_id, params)
            .map_err(|e| MaaError::ffi("set_task_params", format!("Failed to set task params: {:?}", e)))
    }
    
    /// Start task execution
    pub fn start(&self) -> MaaResult<()> {
        debug!("Starting MAA task execution");
        
        self.assistant.start()
            .map_err(|e| MaaError::ffi("start", format!("Failed to start MAA: {:?}", e)))
    }
    
    /// Stop task execution
    pub fn stop(&self) -> MaaResult<()> {
        debug!("Stopping MAA task execution");
        
        self.assistant.stop()
            .map_err(|e| MaaError::ffi("stop", format!("Failed to stop MAA: {:?}", e)))
    }
    
    /// Get device UUID
    pub fn get_uuid(&mut self) -> MaaResult<String> {
        if let Some(ref uuid) = self.device_uuid {
            return Ok(uuid.clone());
        }
        
        let uuid = self.assistant.get_uuid()
            .map_err(|e| MaaError::ffi("get_uuid", format!("Failed to get UUID: {:?}", e)))?;
        
        self.device_uuid = Some(uuid.clone());
        Ok(uuid)
    }
    
    /// Get current target device
    pub fn get_target(&self) -> Option<String> {
        self.connection_params.as_ref().map(|p| p.device_address.clone())
    }
    
    /// Get all active tasks
    pub fn get_tasks(&self) -> MaaResult<Vec<i32>> {
        let tasks = self.active_tasks.lock()
            .map_err(|_| MaaError::synchronization("get_tasks", "Failed to acquire tasks lock"))?;
        
        Ok(tasks.keys().copied().collect())
    }
    
    /// Get MAA version
    pub fn get_version() -> MaaResult<String> {
        Assistant::get_version()
            .map_err(|e| MaaError::ffi("get_version", format!("Failed to get version: {:?}", e)))
    }
    
    /// Set static option
    pub fn set_static_option(key: StaticOptionKey, value: &str) -> MaaResult<()> {
        Assistant::set_static_option(key, value)
            .map_err(|e| MaaError::ffi("set_static_option", format!("Failed to set static option: {:?}", e)))
    }
    
    /// Set instance option
    pub fn set_instance_option(&self, key: InstanceOptionKey, value: &str) -> MaaResult<()> {
        self.assistant.set_instance_option(key, value)
            .map_err(|e| MaaError::ffi("set_instance_option", format!("Failed to set instance option: {:?}", e)))
    }
    
    /// Log message via MAA
    pub fn log(level: &str, message: &str) -> MaaResult<()> {
        Assistant::log(level, message)
            .map_err(|e| MaaError::ffi("log", format!("Failed to log message: {:?}", e)))
    }
    
    /// Go back to home screen
    pub fn back_to_home(&self) -> MaaResult<()> {
        self.assistant.back_to_home()
            .map_err(|e| MaaError::ffi("back_to_home", format!("Failed to go back to home: {:?}", e)))
    }
}

/// MAA callback bridge function - converts C callback to Rust async message
unsafe extern "C" fn maa_callback_bridge(
    msg: std::os::raw::c_int,
    detail_json: *const std::os::raw::c_char,
    custom_arg: *mut c_void,
) {
    if custom_arg.is_null() || detail_json.is_null() {
        return;
    }
    
    // Recover sender from custom_arg
    let sender = &*(custom_arg as *const tokio::sync::mpsc::UnboundedSender<CallbackMessage>);
    
    // Parse JSON detail
    let detail_str = match std::ffi::CStr::from_ptr(detail_json).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => {
            error!("Failed to parse callback detail JSON");
            return;
        }
    };
    
    // Extract task ID from detail
    let task_id = extract_task_id_from_detail(&detail_str);
    
    // Create callback message
    let message = CallbackMessage {
        task_id,
        msg_type: message_type_from_id(msg),
        content: detail_str,
        timestamp: Utc::now(),
    };
    
    // Send message (ignore errors in callback to avoid panics)
    let _ = sender.send(message);
}

/// Extract task ID from detail JSON
fn extract_task_id_from_detail(detail: &str) -> i32 {
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(detail) {
        if let Some(task_id) = json.get("taskid").and_then(|v| v.as_i64()) {
            return task_id as i32;
        }
        if let Some(task_id) = json.get("id").and_then(|v| v.as_i64()) {
            return task_id as i32;
        }
        if let Some(task_id) = json.get("task_id").and_then(|v| v.as_i64()) {
            return task_id as i32;
        }
    }
    0 // Default task_id
}

/// Convert MAA message ID to human-readable type
fn message_type_from_id(msg_id: std::os::raw::c_int) -> String {
    match msg_id {
        0 => "InternalError".to_string(),
        1 => "InitFailed".to_string(),        // 修正：应该是 InitFailed 不是 InitCompleted
        2 => "ConnectionInfo".to_string(),
        3 => "AllTasksCompleted".to_string(),
        4 => "AsyncCallInfo".to_string(),     // 添加：异步调用信息
        5 => "Destroyed".to_string(),         // 添加：实例销毁
        10000 => "TaskChainError".to_string(),
        10001 => "TaskChainStart".to_string(),
        10002 => "TaskChainCompleted".to_string(),
        10003 => "TaskChainExtraInfo".to_string(),
        10004 => "TaskChainStopped".to_string(),
        20000 => "SubTaskError".to_string(),
        20001 => "SubTaskStart".to_string(),
        20002 => "SubTaskCompleted".to_string(),
        20003 => "SubTaskExtraInfo".to_string(),
        20004 => "SubTaskStopped".to_string(),
        30000 => "ReportRequest".to_string(),  // 添加：报告请求
        _ => format!("Unknown({})", msg_id),
    }
}

// MAA is thread-safe
unsafe impl Send for MaaFFIReal {}
unsafe impl Sync for MaaFFIReal {}

impl Drop for MaaFFIReal {
    fn drop(&mut self) {
        debug!("Dropping real MAA FFI instance");
        
        // Stop any running tasks
        if self.is_running() {
            let _ = self.stop();
        }
        
        // Clear active tasks
        if let Ok(mut tasks) = self.active_tasks.lock() {
            tasks.clear();
        }
        
        debug!("Real MAA FFI instance dropped");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_conversion() {
        assert_eq!(message_type_from_id(1), "InitCompleted");
        assert_eq!(message_type_from_id(10001), "TaskChainStart");
        assert_eq!(message_type_from_id(20002), "SubTaskCompleted");
        assert_eq!(message_type_from_id(99999), "Unknown(99999)");
    }

    #[test]
    fn test_task_id_extraction() {
        let json_with_taskid = r#"{"taskid": 123, "status": "completed"}"#;
        assert_eq!(extract_task_id_from_detail(json_with_taskid), 123);
        
        let json_with_id = r#"{"id": 456, "progress": 0.5}"#;
        assert_eq!(extract_task_id_from_detail(json_with_id), 456);
        
        let invalid_json = "not json";
        assert_eq!(extract_task_id_from_detail(invalid_json), 0);
    }

    #[test]
    fn test_maa_core_path_detection() {
        // This test checks if the path detection logic works
        // It may fail if MaaCore is not installed, which is expected in some environments
        match MaaFFIReal::get_maa_core_path() {
            Ok(path) => {
                assert!(path.exists());
                info!("Found MaaCore at: {}", path.display());
            }
            Err(e) => {
                warn!("MaaCore not found (expected in test environment): {}", e);
                // This is expected in environments without MAA installed
            }
        }
    }
}