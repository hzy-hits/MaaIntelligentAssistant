//! Callback handling for MAA FFI operations
//!
//! This module provides a safe bridge between MAA's C-style callbacks and Rust's async world.
//! It converts FFI callbacks into tokio channels, enabling async/await patterns while maintaining
//! thread safety and proper error handling.

use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use chrono::Utc;
use tracing::{debug, error, warn, trace};

use crate::maa_adapter::{types::CallbackMessage, MaaError, MaaResult};

/// Callback handler that manages FFI callbacks and converts them to async messages
#[derive(Debug)]
pub struct CallbackHandler {
    /// Channel sender for callback messages
    tx: mpsc::UnboundedSender<CallbackMessage>,
    
    /// Channel receiver for callback messages
    rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<CallbackMessage>>>>,
    
    /// Active task contexts
    active_tasks: Arc<Mutex<HashMap<i32, TaskCallbackContext>>>,
    
    /// Global callback statistics
    stats: Arc<Mutex<CallbackStats>>,
}

/// Context for individual task callbacks
#[derive(Debug)]
struct TaskCallbackContext {
    /// Task ID
    _task_id: i32,
    
    /// Task type for logging
    _task_type: String,
    
    /// Creation timestamp
    _created_at: chrono::DateTime<chrono::Utc>,
    
    /// Message count for this task
    message_count: u32,
}

/// Statistics for callback operations
#[derive(Debug, Default)]
pub struct CallbackStats {
    /// Total messages received
    total_messages: u64,
    
    /// Messages by type
    messages_by_type: HashMap<String, u64>,
    
    /// Error count
    error_count: u64,
    
    /// Last message timestamp
    last_message_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl CallbackHandler {
    /// Create a new callback handler
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        
        Self {
            tx,
            rx: Arc::new(Mutex::new(Some(rx))),
            active_tasks: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(CallbackStats::default())),
        }
    }

    /// Take the receiver for processing messages
    /// This can only be called once per handler instance
    pub fn take_receiver(&self) -> Option<mpsc::UnboundedReceiver<CallbackMessage>> {
        self.rx.lock().unwrap().take()
    }

    /// Get a sender clone for use in FFI callbacks
    pub fn get_sender(&self) -> mpsc::UnboundedSender<CallbackMessage> {
        self.tx.clone()
    }

    /// Register a new task for callback tracking
    pub fn register_task(&self, task_id: i32, task_type: String) -> MaaResult<()> {
        let mut tasks = self.active_tasks.lock().map_err(|_| {
            MaaError::synchronization("register_task", "Failed to acquire tasks lock")
        })?;

        let context = TaskCallbackContext {
            _task_id: task_id,
            _task_type: task_type.clone(),
            _created_at: Utc::now(),
            message_count: 0,
        };

        tasks.insert(task_id, context);
        
        debug!("Registered task {} for callbacks: {}", task_id, task_type);
        Ok(())
    }

    /// Unregister a task from callback tracking
    pub fn unregister_task(&self, task_id: i32) -> MaaResult<()> {
        let mut tasks = self.active_tasks.lock().map_err(|_| {
            MaaError::synchronization("unregister_task", "Failed to acquire tasks lock")
        })?;

        if let Some(context) = tasks.remove(&task_id) {
            debug!(
                "Unregistered task {} after {} messages", 
                task_id, 
                context.message_count
            );
        } else {
            warn!("Attempted to unregister unknown task: {}", task_id);
        }

        Ok(())
    }

    /// Send a callback message
    pub fn send_message(&self, message: CallbackMessage) -> MaaResult<()> {
        // Update statistics
        self.update_stats(&message)?;
        
        // Update task context
        self.update_task_context(&message)?;
        
        // Send the message
        self.tx.send(message).map_err(|e| {
            MaaError::callback(
                format!("Failed to send callback message: {}", e),
                "mpsc_send".to_string(),
            )
        })?;

        Ok(())
    }

    /// Get callback statistics
    pub fn get_stats(&self) -> MaaResult<CallbackStats> {
        let stats = self.stats.lock().map_err(|_| {
            MaaError::synchronization("get_stats", "Failed to acquire stats lock")
        })?;

        Ok(CallbackStats {
            total_messages: stats.total_messages,
            messages_by_type: stats.messages_by_type.clone(),
            error_count: stats.error_count,
            last_message_at: stats.last_message_at,
        })
    }

    /// Get active task count
    pub fn get_active_task_count(&self) -> MaaResult<usize> {
        let tasks = self.active_tasks.lock().map_err(|_| {
            MaaError::synchronization("get_active_task_count", "Failed to acquire tasks lock")
        })?;

        Ok(tasks.len())
    }

    /// Update callback statistics
    fn update_stats(&self, message: &CallbackMessage) -> MaaResult<()> {
        let mut stats = self.stats.lock().map_err(|_| {
            MaaError::synchronization("update_stats", "Failed to acquire stats lock")
        })?;

        stats.total_messages += 1;
        stats.last_message_at = Some(message.timestamp);
        
        *stats.messages_by_type.entry(message.msg_type.clone()).or_insert(0) += 1;
        
        if message.msg_type == "error" {
            stats.error_count += 1;
        }

        Ok(())
    }

    /// Update task callback context
    fn update_task_context(&self, message: &CallbackMessage) -> MaaResult<()> {
        let mut tasks = self.active_tasks.lock().map_err(|_| {
            MaaError::synchronization("update_task_context", "Failed to acquire tasks lock")
        })?;

        if let Some(context) = tasks.get_mut(&message.task_id) {
            context.message_count += 1;
        }

        Ok(())
    }
}

/// C-style callback function for MAA FFI
/// 
/// # Safety
/// This function is called from C code and must handle all potential panics and errors.
/// The user_data pointer must be a valid CallbackHandler sender.
pub unsafe extern "C" fn maa_callback_handler(
    message: *const c_char,
    details_json: *const c_char,
    user_data: *mut c_void,
) {
    // Prevent panics from crossing FFI boundary
    let result = std::panic::catch_unwind(|| {
        callback_handler_impl(message, details_json, user_data)
    });

    if let Err(panic_info) = result {
        error!("Panic in MAA callback handler: {:?}", panic_info);
    }
}

/// Implementation of the callback handler
/// 
/// # Safety
/// This function assumes the pointers are valid and the strings are null-terminated
unsafe fn callback_handler_impl(
    message: *const c_char,
    details_json: *const c_char,
    user_data: *mut c_void,
) {
    // Validate pointers
    if message.is_null() || user_data.is_null() {
        error!("Null pointer passed to MAA callback handler");
        return;
    }

    // Extract the sender from user_data
    let sender = &*(user_data as *const mpsc::UnboundedSender<CallbackMessage>);

    // Convert C strings to Rust strings
    let message_str = match CStr::from_ptr(message).to_str() {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to convert message to UTF-8: {}", e);
            return;
        }
    };

    let details_str = if details_json.is_null() {
        "{}"
    } else {
        match CStr::from_ptr(details_json).to_str() {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to convert details to UTF-8: {}", e);
                "{}"
            }
        }
    };

    trace!("Received MAA callback: {} | {}", message_str, details_str);

    // Parse the message and details
    let (task_id, msg_type) = parse_callback_message(message_str);
    
    // Create callback message
    let callback_msg = CallbackMessage {
        task_id,
        msg_type,
        content: details_str.to_string(),
        timestamp: Utc::now(),
    };

    // Send the message
    if let Err(e) = sender.send(callback_msg) {
        error!("Failed to send callback message: {}", e);
    }
}

/// Parse MAA callback message to extract task ID and message type
fn parse_callback_message(message: &str) -> (i32, String) {
    // MAA callback messages typically follow patterns like:
    // "Task.1.Completed" or "Task.123.Progress" or "Connection.Established"
    
    let parts: Vec<&str> = message.split('.').collect();
    
    if parts.len() >= 3 && parts[0] == "Task" {
        // Task-specific message
        if let Ok(task_id) = parts[1].parse::<i32>() {
            let msg_type = parts[2..].join(".");
            (task_id, msg_type)
        } else {
            warn!("Failed to parse task ID from message: {}", message);
            (0, message.to_string())
        }
    } else {
        // Global message (no specific task)
        (0, message.to_string())
    }
}

/// Helper function to create a callback user_data pointer
pub fn create_callback_user_data(
    sender: mpsc::UnboundedSender<CallbackMessage>
) -> Box<mpsc::UnboundedSender<CallbackMessage>> {
    Box::new(sender)
}

/// Helper function to extract sender from user_data pointer
/// 
/// # Safety
/// The user_data pointer must be a valid Box<mpsc::UnboundedSender<CallbackMessage>>
pub unsafe fn extract_callback_sender(
    user_data: *mut c_void
) -> Option<mpsc::UnboundedSender<CallbackMessage>> {
    if user_data.is_null() {
        return None;
    }

    let sender_box = Box::from_raw(user_data as *mut mpsc::UnboundedSender<CallbackMessage>);
    let sender = (*sender_box).clone();
    
    // Prevent the box from being dropped
    let _ = Box::into_raw(sender_box);
    
    Some(sender)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[test]
    fn test_callback_handler_creation() {
        let handler = CallbackHandler::new();
        assert_eq!(handler.get_active_task_count().unwrap(), 0);
    }

    #[test]
    fn test_task_registration() {
        let handler = CallbackHandler::new();
        
        handler.register_task(1, "test_task".to_string()).unwrap();
        assert_eq!(handler.get_active_task_count().unwrap(), 1);
        
        handler.unregister_task(1).unwrap();
        assert_eq!(handler.get_active_task_count().unwrap(), 0);
    }

    #[tokio::test]
    async fn test_message_sending() {
        let handler = CallbackHandler::new();
        let mut rx = handler.take_receiver().unwrap();
        
        let message = CallbackMessage {
            task_id: 1,
            msg_type: "test".to_string(),
            content: "test content".to_string(),
            timestamp: Utc::now(),
        };
        
        handler.send_message(message.clone()).unwrap();
        
        let received = timeout(Duration::from_millis(100), rx.recv()).await.unwrap().unwrap();
        assert_eq!(received.task_id, message.task_id);
        assert_eq!(received.msg_type, message.msg_type);
        assert_eq!(received.content, message.content);
    }

    #[test]
    fn test_parse_callback_message() {
        // Test task-specific message
        let (task_id, msg_type) = parse_callback_message("Task.123.Completed");
        assert_eq!(task_id, 123);
        assert_eq!(msg_type, "Completed");
        
        // Test task message with multiple parts
        let (task_id, msg_type) = parse_callback_message("Task.456.Progress.Update");
        assert_eq!(task_id, 456);
        assert_eq!(msg_type, "Progress.Update");
        
        // Test global message
        let (task_id, msg_type) = parse_callback_message("Connection.Established");
        assert_eq!(task_id, 0);
        assert_eq!(msg_type, "Connection.Established");
        
        // Test invalid task ID
        let (task_id, msg_type) = parse_callback_message("Task.invalid.Completed");
        assert_eq!(task_id, 0);
        assert_eq!(msg_type, "Task.invalid.Completed");
    }

    #[test]
    fn test_callback_stats() {
        let handler = CallbackHandler::new();
        
        let message1 = CallbackMessage {
            task_id: 1,
            msg_type: "progress".to_string(),
            content: "{}".to_string(),
            timestamp: Utc::now(),
        };
        
        let message2 = CallbackMessage {
            task_id: 2,
            msg_type: "error".to_string(),
            content: "{}".to_string(),
            timestamp: Utc::now(),
        };
        
        handler.send_message(message1).unwrap();
        handler.send_message(message2).unwrap();
        
        let stats = handler.get_stats().unwrap();
        assert_eq!(stats.total_messages, 2);
        assert_eq!(stats.error_count, 1);
        assert_eq!(*stats.messages_by_type.get("progress").unwrap(), 1);
        assert_eq!(*stats.messages_by_type.get("error").unwrap(), 1);
    }

    #[test]
    fn test_user_data_helpers() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let user_data = create_callback_user_data(tx.clone());
        let user_data_ptr = Box::into_raw(user_data) as *mut c_void;
        
        unsafe {
            let extracted = extract_callback_sender(user_data_ptr).unwrap();
            // The extracted sender should be able to send messages
            extracted.send(CallbackMessage {
                task_id: 1,
                msg_type: "test".to_string(),
                content: "test".to_string(),
                timestamp: Utc::now(),
            }).unwrap();
            
            // Clean up
            let _cleanup = Box::from_raw(user_data_ptr as *mut mpsc::UnboundedSender<CallbackMessage>);
        }
    }
}