//! Stub implementation for MAA FFI when MAA Core is not available
//!
//! This module provides a stub implementation that mimics the MAA FFI interface
//! for development and testing purposes when the actual MAA Core is not available.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tracing::{debug, info, warn};
use chrono::Utc;

use crate::maa_adapter::{MaaError, MaaResult, CallbackMessage};

/// Stub MAA FFI wrapper for development/testing
pub struct MaaFFIStub {
    /// Resource path
    _resource_path: String,
    
    /// Connection parameters
    connection_params: Option<StubConnectionParams>,
    
    /// Callback sender for async message handling
    callback_sender: Option<tokio::sync::mpsc::UnboundedSender<CallbackMessage>>,
    
    /// Active tasks tracking
    active_tasks: Arc<Mutex<HashMap<i32, String>>>,
    
    /// Next task ID
    next_task_id: Arc<Mutex<i32>>,
    
    /// Running state
    is_running: Arc<Mutex<bool>>,
}

/// Connection parameters for stub
#[derive(Debug, Clone)]
pub struct StubConnectionParams {
    pub adb_path: String,
    pub device_address: String,
    pub config: Option<String>,
}

impl MaaFFIStub {
    /// Create a new MAA FFI stub
    pub fn new(resource_path: String) -> MaaResult<Self> {
        debug!("Creating MAA FFI stub with resource path: {}", resource_path);
        
        info!("MAA resource loaded successfully (stub implementation)");
        
        Ok(Self {
            _resource_path: resource_path,
            connection_params: None,
            callback_sender: None,
            active_tasks: Arc::new(Mutex::new(HashMap::new())),
            next_task_id: Arc::new(Mutex::new(1)),
            is_running: Arc::new(Mutex::new(false)),
        })
    }

    /// Create MAA stub with callback support
    pub fn with_callback(
        resource_path: String, 
        callback_sender: tokio::sync::mpsc::UnboundedSender<CallbackMessage>
    ) -> MaaResult<Self> {
        debug!("Creating MAA FFI stub with callback support");
        
        info!("MAA resource loaded successfully with callback support (stub implementation)");
        
        Ok(Self {
            _resource_path: resource_path,
            connection_params: None,
            callback_sender: Some(callback_sender),
            active_tasks: Arc::new(Mutex::new(HashMap::new())),
            next_task_id: Arc::new(Mutex::new(1)),
            is_running: Arc::new(Mutex::new(false)),
        })
    }

    /// Connect to device (stub)
    pub fn connect(&mut self, adb_path: &str, device_address: &str, config: Option<&str>) -> MaaResult<i32> {
        debug!("Stub: Connecting to device: {} via {}", device_address, adb_path);
        
        // Store connection parameters
        self.connection_params = Some(StubConnectionParams {
            adb_path: adb_path.to_string(),
            device_address: device_address.to_string(),
            config: config.map(|s| s.to_string()),
        });
        
        let connect_id = 42; // Stub connection ID
        info!("Stub: Connection initiated with ID: {}", connect_id);
        Ok(connect_id)
    }

    /// Check if MAA is currently running (stub)
    pub fn is_running(&self) -> bool {
        *self.is_running.lock().unwrap_or_else(|_| {
            warn!("Failed to acquire is_running lock, assuming not running");
            panic!("Mutex poisoned")
        })
    }

    /// Take a screenshot (stub)
    pub fn screenshot(&self) -> MaaResult<Vec<u8>> {
        debug!("Stub: Taking screenshot");
        
        // Return a simple dummy image (1x1 RGB pixel)
        let dummy_image = vec![255u8, 128u8, 64u8]; // One RGB pixel
        
        debug!("Stub: Screenshot taken, size: {} bytes", dummy_image.len());
        Ok(dummy_image)
    }

    /// Click at specified coordinates (stub)
    pub fn click(&self, x: i32, y: i32) -> MaaResult<i32> {
        debug!("Stub: Clicking at ({}, {})", x, y);
        
        let click_id = 101; // Stub click ID
        debug!("Stub: Click initiated with ID: {}", click_id);
        Ok(click_id)
    }

    /// Create a new task (stub)
    pub fn create_task(&mut self, task_type: &str, params: &str) -> MaaResult<i32> {
        debug!("Stub: Creating task: {} with params: {}", task_type, params);
        
        let task_id = {
            let mut id_guard = self.next_task_id.lock().map_err(|_| {
                MaaError::synchronization("create_task", "Failed to acquire task ID lock")
            })?;
            let id = *id_guard;
            *id_guard += 1;
            id
        };
        
        // Track the task
        if let Ok(mut tasks) = self.active_tasks.lock() {
            tasks.insert(task_id, task_type.to_string());
        }
        
        info!("Stub: Task created with ID: {}", task_id);
        Ok(task_id)
    }

    /// Set task parameters (stub)
    pub fn set_task_params(&self, task_id: i32, params: &str) -> MaaResult<()> {
        debug!("Stub: Setting params for task {}: {}", task_id, params);
        Ok(())
    }

    /// Start task execution (stub)
    pub fn start(&self) -> MaaResult<()> {
        debug!("Stub: Starting MAA task execution");
        
        *self.is_running.lock().map_err(|_| {
            MaaError::synchronization("start", "Failed to acquire running lock")
        })? = true;
        
        // Simulate async callback if available
        if let Some(ref sender) = self.callback_sender {
            let sender_clone = sender.clone();
            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                
                let _ = sender_clone.send(CallbackMessage {
                    task_id: 1,
                    msg_type: "TaskChainStart".to_string(),
                    content: r#"{"stage": "start"}"#.to_string(),
                    timestamp: Utc::now(),
                });
                
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                
                let _ = sender_clone.send(CallbackMessage {
                    task_id: 1,
                    msg_type: "TaskChainCompleted".to_string(),
                    content: r#"{"stage": "completed"}"#.to_string(),
                    timestamp: Utc::now(),
                });
            });
        }
        
        Ok(())
    }

    /// Stop task execution (stub)
    pub fn stop(&self) -> MaaResult<()> {
        debug!("Stub: Stopping MAA task execution");
        
        *self.is_running.lock().map_err(|_| {
            MaaError::synchronization("stop", "Failed to acquire running lock")
        })? = false;
        
        Ok(())
    }

    /// Get MAA UUID (stub)
    pub fn get_uuid(&mut self) -> MaaResult<String> {
        let uuid = "stub-uuid-12345".to_string();
        Ok(uuid)
    }

    /// Get current target device (stub)
    pub fn get_target(&self) -> Option<String> {
        self.connection_params.as_ref().map(|params| params.device_address.clone())
    }

    /// Get all active tasks (stub)
    pub fn get_tasks(&self) -> MaaResult<Vec<i32>> {
        let tasks = self.active_tasks.lock().map_err(|_| {
            MaaError::synchronization("get_tasks", "Failed to acquire tasks lock")
        })?;
        
        Ok(tasks.keys().copied().collect())
    }

    /// Get MAA version (stub)
    pub fn get_version() -> MaaResult<String> {
        Ok("v4.0.0-stub".to_string())
    }

    /// Set static option (stub)
    pub fn set_static_option(_key: i32, value: &str) -> MaaResult<()> {
        debug!("Stub: Setting static option to: {}", value);
        Ok(())
    }

    /// Set instance option (stub)
    pub fn set_option(&mut self, _key: i32, value: &str) -> MaaResult<()> {
        debug!("Stub: Setting instance option to: {}", value);
        Ok(())
    }

    /// Log message via MAA (stub)
    pub fn log(level: &str, message: &str) -> MaaResult<()> {
        info!("Stub MAA Log [{}]: {}", level, message);
        Ok(())
    }
    
    /// Check if connected to device (stub)
    pub fn is_connected(&self) -> bool {
        self.connection_params.is_some()
    }
    
    /// Go back to home screen (stub)
    pub fn back_to_home(&self) -> MaaResult<()> {
        debug!("Stub: Going back to home screen");
        Ok(())
    }
}

// Stub is always safe to Send/Sync since it doesn't use real FFI
unsafe impl Send for MaaFFIStub {}
unsafe impl Sync for MaaFFIStub {}

impl Drop for MaaFFIStub {
    fn drop(&mut self) {
        debug!("Dropping MAA FFI stub");
        
        // Stop any running tasks
        if self.is_running() {
            let _ = self.stop();
        }
        
        // Clear active tasks
        if let Ok(mut tasks) = self.active_tasks.lock() {
            tasks.clear();
        }
        
        debug!("MAA FFI stub dropped");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[test]
    fn test_stub_creation() {
        let resource_path = "./test_resources".to_string();
        let stub = MaaFFIStub::new(resource_path);
        assert!(stub.is_ok());
    }

    #[tokio::test]
    async fn test_stub_with_callback() {
        let resource_path = "./test_resources".to_string();
        let (tx, _rx) = mpsc::unbounded_channel();
        
        let stub = MaaFFIStub::with_callback(resource_path, tx);
        assert!(stub.is_ok());
    }

    #[test]
    fn test_stub_operations() {
        let resource_path = "./test_resources".to_string();
        let mut stub = MaaFFIStub::new(resource_path).unwrap();
        
        // Test connection
        let connect_result = stub.connect("adb", "127.0.0.1:5555", None);
        assert!(connect_result.is_ok());
        
        // Test screenshot
        let screenshot = stub.screenshot();
        assert!(screenshot.is_ok());
        assert!(!screenshot.unwrap().is_empty());
        
        // Test click
        let click_result = stub.click(100, 200);
        assert!(click_result.is_ok());
        
        // Test task creation
        let task_result = stub.create_task("Screenshot", "{}");
        assert!(task_result.is_ok());
        
        // Test start/stop
        assert!(stub.start().is_ok());
        assert!(stub.is_running());
        assert!(stub.stop().is_ok());
        assert!(!stub.is_running());
    }

    #[test]
    fn test_version_info() {
        let version = MaaFFIStub::get_version();
        assert!(version.is_ok());
        assert!(version.unwrap().contains("stub"));
    }
}