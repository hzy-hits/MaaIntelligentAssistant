//! Error types for MAA Adapter
//!
//! This module defines a comprehensive error handling system for all MAA operations.
//! It provides structured error types that can be easily converted, logged, and handled
//! by higher-level components.

use thiserror::Error;

/// Result type alias for MAA operations
pub type MaaResult<T> = Result<T, MaaError>;

/// Comprehensive error type for MAA operations
#[derive(Error, Debug, Clone)]
pub enum MaaError {
    /// Connection-related errors
    #[error("Connection failed: {message}")]
    Connection {
        message: String,
        #[source]
        source: Option<Box<MaaError>>,
    },

    /// Task execution errors
    #[error("Task execution failed: task_id={task_id}, reason={reason}")]
    TaskExecution {
        task_id: i32,
        reason: String,
        details: Option<String>,
    },

    /// FFI-related errors
    #[error("FFI operation failed: {operation} - {message}")]
    Ffi {
        operation: String,
        message: String,
        error_code: Option<i32>,
    },

    /// Configuration errors
    #[error("Invalid configuration: {field} - {message}")]
    Configuration {
        field: String,
        message: String,
        provided_value: Option<String>,
    },

    /// Resource management errors
    #[error("Resource error: {resource_type} - {message}")]
    Resource {
        resource_type: String,
        message: String,
        path: Option<String>,
    },

    /// Timeout errors
    #[error("Operation timed out: {operation} after {timeout_ms}ms")]
    Timeout {
        operation: String,
        timeout_ms: u64,
    },

    /// Serialization/Deserialization errors
    #[error("Serialization error: {message}")]
    Serialization {
        message: String,
    },

    /// IO-related errors
    #[error("IO error during {operation}: {message}")]
    Io {
        operation: String,
        message: String,
    },

    /// Invalid state errors
    #[error("Invalid state: {current_state} - {message}")]
    InvalidState {
        current_state: String,
        message: String,
        expected_state: Option<String>,
    },

    /// Parameter validation errors
    #[error("Invalid parameter: {parameter} - {message}")]
    InvalidParameter {
        parameter: String,
        message: String,
        provided_value: Option<String>,
    },

    /// Callback handling errors
    #[error("Callback error: {message}")]
    Callback {
        message: String,
        callback_type: String,
    },

    /// Thread synchronization errors
    #[error("Synchronization error: {operation} - {message}")]
    Synchronization {
        operation: String,
        message: String,
    },

    /// Device-related errors
    #[error("Device error: {device} - {message}")]
    Device {
        device: String,
        message: String,
        error_code: Option<String>,
    },

    /// Screenshot/Image processing errors
    #[error("Image processing error: {operation} - {message}")]
    ImageProcessing {
        operation: String,
        message: String,
        dimensions: Option<(u32, u32)>,
    },

    /// Task queue errors
    #[error("Task queue error: {message}")]
    TaskQueue {
        message: String,
        queue_size: Option<usize>,
    },

    /// Internal adapter errors
    #[error("Internal adapter error: {component} - {message}")]
    Internal {
        component: String,
        message: String,
        debug_info: Option<String>,
    },
}

impl MaaError {
    /// Create a new connection error
    pub fn connection<S: Into<String>>(message: S) -> Self {
        Self::Connection {
            message: message.into(),
            source: None,
        }
    }

    /// Create a new connection error with source
    pub fn connection_with_source<S: Into<String>>(message: S, source: MaaError) -> Self {
        Self::Connection {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a new task execution error
    pub fn task_execution<S: Into<String>>(task_id: i32, reason: S) -> Self {
        Self::TaskExecution {
            task_id,
            reason: reason.into(),
            details: None,
        }
    }

    /// Create a new task execution error with details
    pub fn task_execution_with_details<S1, S2>(task_id: i32, reason: S1, details: S2) -> Self 
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self::TaskExecution {
            task_id,
            reason: reason.into(),
            details: Some(details.into()),
        }
    }

    /// Create a new FFI error
    pub fn ffi<S1, S2>(operation: S1, message: S2) -> Self 
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self::Ffi {
            operation: operation.into(),
            message: message.into(),
            error_code: None,
        }
    }

    /// Create a new FFI error with error code
    pub fn ffi_with_code<S1, S2>(operation: S1, message: S2, error_code: i32) -> Self 
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self::Ffi {
            operation: operation.into(),
            message: message.into(),
            error_code: Some(error_code),
        }
    }

    /// Create a new configuration error
    pub fn configuration<S1, S2>(field: S1, message: S2) -> Self 
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self::Configuration {
            field: field.into(),
            message: message.into(),
            provided_value: None,
        }
    }

    /// Create a new timeout error
    pub fn timeout<S: Into<String>>(operation: S, timeout_ms: u64) -> Self {
        Self::Timeout {
            operation: operation.into(),
            timeout_ms,
        }
    }

    /// Create a new invalid state error
    pub fn invalid_state<S1, S2>(current_state: S1, message: S2) -> Self 
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self::InvalidState {
            current_state: current_state.into(),
            message: message.into(),
            expected_state: None,
        }
    }

    /// Create a new invalid parameter error
    pub fn invalid_parameter<S1, S2>(parameter: S1, message: S2) -> Self 
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self::InvalidParameter {
            parameter: parameter.into(),
            message: message.into(),
            provided_value: None,
        }
    }

    /// Create a new device error
    pub fn device<S1, S2>(device: S1, message: S2) -> Self 
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self::Device {
            device: device.into(),
            message: message.into(),
            error_code: None,
        }
    }

    /// Create a new internal error
    pub fn internal<S1, S2>(component: S1, message: S2) -> Self 
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self::Internal {
            component: component.into(),
            message: message.into(),
            debug_info: None,
        }
    }

    /// Create a new callback error
    pub fn callback<S1, S2>(message: S1, callback_type: S2) -> Self 
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self::Callback {
            message: message.into(),
            callback_type: callback_type.into(),
        }
    }

    /// Create a new synchronization error
    pub fn synchronization<S1, S2>(operation: S1, message: S2) -> Self 
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self::Synchronization {
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// Check if this is a recoverable error
    pub fn is_recoverable(&self) -> bool {
        match self {
            MaaError::Connection { .. } => true,
            MaaError::Timeout { .. } => true,
            MaaError::Device { .. } => true,
            MaaError::Synchronization { .. } => true,
            MaaError::TaskQueue { .. } => true,
            _ => false,
        }
    }

    /// Check if this error should trigger a retry
    pub fn should_retry(&self) -> bool {
        match self {
            MaaError::Connection { .. } => true,
            MaaError::Timeout { .. } => true,
            MaaError::Device { .. } => true,
            MaaError::Ffi { error_code: Some(code), .. } if *code < 0 => true,
            _ => false,
        }
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            MaaError::Configuration { .. } => ErrorSeverity::Critical,
            MaaError::InvalidParameter { .. } => ErrorSeverity::Critical,
            MaaError::Ffi { .. } => ErrorSeverity::High,
            MaaError::TaskExecution { .. } => ErrorSeverity::Medium,
            MaaError::Connection { .. } => ErrorSeverity::Medium,
            MaaError::Timeout { .. } => ErrorSeverity::Low,
            MaaError::Device { .. } => ErrorSeverity::Medium,
            _ => ErrorSeverity::Low,
        }
    }

    /// Get structured error information for logging
    pub fn error_info(&self) -> ErrorInfo {
        ErrorInfo {
            error_type: self.error_type(),
            message: self.to_string(),
            is_recoverable: self.is_recoverable(),
            should_retry: self.should_retry(),
            severity: self.severity(),
            details: self.details(),
        }
    }

    /// Get error type as string
    fn error_type(&self) -> &'static str {
        match self {
            MaaError::Connection { .. } => "Connection",
            MaaError::TaskExecution { .. } => "TaskExecution",
            MaaError::Ffi { .. } => "FFI",
            MaaError::Configuration { .. } => "Configuration",
            MaaError::Resource { .. } => "Resource",
            MaaError::Timeout { .. } => "Timeout",
            MaaError::Serialization { .. } => "Serialization",
            MaaError::Io { .. } => "IO",
            MaaError::InvalidState { .. } => "InvalidState",
            MaaError::InvalidParameter { .. } => "InvalidParameter",
            MaaError::Callback { .. } => "Callback",
            MaaError::Synchronization { .. } => "Synchronization",
            MaaError::Device { .. } => "Device",
            MaaError::ImageProcessing { .. } => "ImageProcessing",
            MaaError::TaskQueue { .. } => "TaskQueue",
            MaaError::Internal { .. } => "Internal",
        }
    }

    /// Get additional error details
    fn details(&self) -> Option<String> {
        match self {
            MaaError::TaskExecution { details, .. } => details.clone(),
            MaaError::Configuration { provided_value, .. } => provided_value.clone(),
            MaaError::Resource { path, .. } => path.clone(),
            MaaError::InvalidParameter { provided_value, .. } => provided_value.clone(),
            MaaError::Internal { debug_info, .. } => debug_info.clone(),
            _ => None,
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Critical errors that prevent operation
    Critical,
    /// High severity errors that need immediate attention
    High,
    /// Medium severity errors that should be handled
    Medium,
    /// Low severity errors that are often recoverable
    Low,
}

/// Structured error information for logging and monitoring
#[derive(Debug, Clone)]
pub struct ErrorInfo {
    pub error_type: &'static str,
    pub message: String,
    pub is_recoverable: bool,
    pub should_retry: bool,
    pub severity: ErrorSeverity,
    pub details: Option<String>,
}

// Standard library error conversions
impl From<std::io::Error> for MaaError {
    fn from(err: std::io::Error) -> Self {
        Self::Io {
            operation: "io_operation".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for MaaError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization {
            message: err.to_string(),
        }
    }
}

impl From<tokio::sync::mpsc::error::SendError<crate::maa_adapter::types::CallbackMessage>> for MaaError {
    fn from(err: tokio::sync::mpsc::error::SendError<crate::maa_adapter::types::CallbackMessage>) -> Self {
        Self::Callback {
            message: format!("Failed to send callback message: {}", err),
            callback_type: "mpsc_send".to_string(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_error() {
        let error = MaaError::connection("Failed to connect to device");
        assert!(matches!(error, MaaError::Connection { .. }));
        assert!(error.is_recoverable());
        assert!(error.should_retry());
        assert_eq!(error.severity(), ErrorSeverity::Medium);
    }

    #[test]
    fn test_task_execution_error() {
        let error = MaaError::task_execution(123, "Task failed to complete");
        assert!(matches!(error, MaaError::TaskExecution { .. }));
        assert!(!error.is_recoverable());
        assert!(!error.should_retry());
        assert_eq!(error.severity(), ErrorSeverity::Medium);
    }

    #[test]
    fn test_ffi_error_with_code() {
        let error = MaaError::ffi_with_code("MaaStart", "Invalid handle", -1);
        assert!(matches!(error, MaaError::Ffi { .. }));
        assert!(!error.is_recoverable());
        assert!(error.should_retry()); // Negative error codes should retry
        assert_eq!(error.severity(), ErrorSeverity::High);
    }

    #[test]
    fn test_configuration_error() {
        let error = MaaError::configuration("resource_path", "Path does not exist");
        assert!(matches!(error, MaaError::Configuration { .. }));
        assert!(!error.is_recoverable());
        assert!(!error.should_retry());
        assert_eq!(error.severity(), ErrorSeverity::Critical);
    }

    #[test]
    fn test_timeout_error() {
        let error = MaaError::timeout("screenshot", 5000);
        assert!(matches!(error, MaaError::Timeout { .. }));
        assert!(error.is_recoverable());
        assert!(error.should_retry());
        assert_eq!(error.severity(), ErrorSeverity::Low);
    }

    #[test]
    fn test_error_info() {
        let error = MaaError::connection("Test connection error");
        let info = error.error_info();
        
        assert_eq!(info.error_type, "Connection");
        assert!(info.is_recoverable);
        assert!(info.should_retry);
        assert_eq!(info.severity, ErrorSeverity::Medium);
    }

    #[test]
    fn test_error_display() {
        let error = MaaError::task_execution_with_details(
            42, 
            "Test failed", 
            "Additional details about the failure"
        );
        
        let display_str = error.to_string();
        assert!(display_str.contains("Task execution failed"));
        assert!(display_str.contains("task_id=42"));
        assert!(display_str.contains("Test failed"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let maa_error: MaaError = io_error.into();
        
        assert!(matches!(maa_error, MaaError::Io { .. }));
    }

    #[test]
    fn test_serde_error_conversion() {
        let json_error = serde_json::from_str::<i32>("not a number").unwrap_err();
        let maa_error: MaaError = json_error.into();
        
        assert!(matches!(maa_error, MaaError::Serialization { .. }));
    }
}