//! Type definitions for MAA Adapter
//!
//! This module contains all the data structures used for MAA operations,
//! including configurations, task definitions, status types, and FFI-related types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use chrono::{DateTime, Utc};

/// Configuration for MAA Adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaaConfig {
    /// Path to MAA resource directory
    pub resource_path: String,
    
    /// ADB path for controller
    pub adb_path: String,
    
    /// Device address (for ADB connection)
    pub device_address: String,
    
    /// Connection type (ADB, Win32, etc.)
    pub connection_type: String,
    
    /// Additional configuration options
    pub options: HashMap<String, String>,
    
    /// Timeout for operations in milliseconds
    pub timeout_ms: u64,
    
    /// Maximum retry attempts
    pub max_retries: u32,
    
    /// Enable debug logging
    pub debug: bool,
}

impl Default for MaaConfig {
    fn default() -> Self {
        Self {
            resource_path: "./maa-official/resource".to_string(),
            adb_path: "adb".to_string(),
            device_address: "127.0.0.1:5555".to_string(),
            connection_type: "ADB".to_string(),
            options: HashMap::new(),
            timeout_ms: 30000,
            max_retries: 3,
            debug: false,
        }
    }
}

/// MAA operation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MaaStatus {
    /// Adapter is idle, no operations running
    Idle,
    
    /// Connecting to device
    Connecting,
    
    /// Connected and ready
    Connected,
    
    /// Running a task
    Running {
        /// ID of the running task
        task_id: i32,
        /// Progress percentage (0.0 - 1.0)
        progress: f32,
        /// Current operation description
        current_operation: String,
    },
    
    /// Task completed successfully
    Completed {
        /// ID of the completed task
        task_id: i32,
        /// Result data
        result: String,
        /// Completion timestamp
        completed_at: DateTime<Utc>,
    },
    
    /// Task failed
    Failed {
        /// ID of the failed task
        task_id: i32,
        /// Error message
        error: String,
        /// Failure timestamp
        failed_at: DateTime<Utc>,
    },
    
    /// Connection lost
    Disconnected {
        /// Reason for disconnection
        reason: String,
    },
}

/// MAA task definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaaTask {
    /// Unique task identifier
    pub id: i32,
    
    /// Task type
    pub task_type: MaaTaskType,
    
    /// Task parameters
    pub params: TaskParams,
    
    /// Task priority (higher number = higher priority)
    pub priority: u32,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Current status
    pub status: MaaStatus,
    
    /// Progress percentage (0.0 - 1.0)
    pub progress: f32,
    
    /// Error message if failed
    pub error: Option<String>,
}

/// Available MAA task types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MaaTaskType {
    /// Take a screenshot
    Screenshot,
    
    /// Click at coordinates
    Click { x: i32, y: i32 },
    
    /// Swipe gesture
    Swipe { 
        from_x: i32, 
        from_y: i32, 
        to_x: i32, 
        to_y: i32,
        duration: u32,
    },
    
    /// Start fight operation
    StartFight,
    
    /// Recruit operators
    Recruit,
    
    /// Infrastructure operations
    Infrast,
    
    /// Mall operations
    Mall,
    
    /// Award tasks
    Award,
    
    /// Roguelike/IS operations
    Roguelike,
    
    /// Daily missions
    Daily,
    
    /// Custom task with parameters
    Custom { 
        task_name: String,
        task_params: String,
    },
    
    /// Copilot task execution
    Copilot {
        stage_name: String,
        copilot_data: String,
    },
    
    /// SSS Copilot task execution
    SSSCopilot {
        stage_name: String,
        copilot_data: String,
    },
    
    /// Depot operations
    Depot,
    
    /// Operator box operations
    OperBox,
    
    /// Reclamation algorithm operations
    ReclamationAlgorithm,
    
    /// Single step task
    SingleStep,
    
    /// Video recognition task
    VideoRecognition,
    
    /// Debug task
    Debug,
    
    /// Close down game
    CloseDown,
    
    /// Start up game
    StartUp,
}

/// Task parameters container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskParams {
    /// Raw parameter string (JSON)
    pub raw: String,
    
    /// Parsed parameters
    pub parsed: HashMap<String, serde_json::Value>,
    
    /// Task-specific settings
    pub settings: HashMap<String, String>,
}

impl Default for TaskParams {
    fn default() -> Self {
        Self {
            raw: "{}".to_string(),
            parsed: HashMap::new(),
            settings: HashMap::new(),
        }
    }
}

/// MAA Controller handle wrapper
#[derive(Debug)]
pub struct MaaController {
    /// FFI handle pointer
    pub(crate) _handle: *mut c_void,
    
    /// Controller type
    pub controller_type: String,
    
    /// Connection parameters
    pub connection_params: String,
    
    /// Is connected flag
    pub connected: bool,
}

// SAFETY: MaaController is only accessed through synchronized methods
unsafe impl Send for MaaController {}
unsafe impl Sync for MaaController {}

/// MAA Resource handle wrapper
#[derive(Debug)]
pub struct MaaResource {
    /// FFI handle pointer
    pub(crate) _handle: *mut c_void,
    
    /// Resource path
    pub resource_path: String,
    
    /// Is loaded flag
    pub loaded: bool,
}

// SAFETY: MaaResource is only accessed through synchronized methods
unsafe impl Send for MaaResource {}
unsafe impl Sync for MaaResource {}

/// Device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Device name/identifier
    pub name: String,
    
    /// Screen resolution
    pub resolution: (u32, u32),
    
    /// DPI settings
    pub dpi: u32,
    
    /// Device capabilities
    pub capabilities: Vec<String>,
    
    /// Additional properties
    pub properties: HashMap<String, String>,
}

/// Callback function type for MAA operations
pub type MaaCallback = Arc<dyn Fn(i32, &str, &str) + Send + Sync>;

/// Callback message for internal communication
#[derive(Debug, Clone)]
pub struct CallbackMessage {
    /// Task ID that triggered the callback
    pub task_id: i32,
    
    /// Message type
    pub msg_type: String,
    
    /// Message content
    pub content: String,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Task execution context
#[derive(Debug)]
pub struct TaskContext {
    /// Task ID
    pub task_id: i32,
    
    /// Callback sender
    pub callback_tx: mpsc::UnboundedSender<CallbackMessage>,
    
    /// Cancellation token
    pub cancel_token: CancellationToken,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maa_config_default() {
        let config = MaaConfig::default();
        assert_eq!(config.timeout_ms, 30000);
        assert_eq!(config.max_retries, 3);
        assert!(!config.debug);
    }

    #[test]
    fn test_maa_status_serialization() {
        let status = MaaStatus::Running {
            task_id: 1,
            progress: 0.5,
            current_operation: "test".to_string(),
        };
        
        let serialized = serde_json::to_string(&status).unwrap();
        let deserialized: MaaStatus = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(status, deserialized);
    }

    #[test]
    fn test_task_params_default() {
        let params = TaskParams::default();
        assert_eq!(params.raw, "{}");
        assert!(params.parsed.is_empty());
        assert!(params.settings.is_empty());
    }

    #[test]
    fn test_maa_task_type_click() {
        let task_type = MaaTaskType::Click { x: 100, y: 200 };
        
        match task_type {
            MaaTaskType::Click { x, y } => {
                assert_eq!(x, 100);
                assert_eq!(y, 200);
            }
            _ => panic!("Expected Click task type"),
        }
    }

    #[test]
    fn test_callback_message_creation() {
        let msg = CallbackMessage {
            task_id: 1,
            msg_type: "progress".to_string(),
            content: "50%".to_string(),
            timestamp: Utc::now(),
        };
        
        assert_eq!(msg.task_id, 1);
        assert_eq!(msg.msg_type, "progress");
        assert_eq!(msg.content, "50%");
    }
}