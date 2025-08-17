//! Operator Manager Error Types
//!
//! This module defines comprehensive error handling for the operator manager,
//! providing structured error types with context information for debugging.

use thiserror::Error;
use serde::{Serialize, Deserialize};

/// Operator manager error types
/// 
/// Comprehensive error handling for all operator management operations.
/// Each error type includes relevant context to aid in debugging and user feedback.
#[derive(Debug, Error, Serialize, Deserialize, Clone)]
pub enum OperatorError {
    /// MAA adapter operation failed
    #[error("MAA operation failed: {operation} - {message}")]
    MaaOperation {
        operation: String,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        error_code: Option<String>,
    },
    
    /// Cache operation failed
    #[error("Cache operation failed: {operation} - {message}")]
    Cache {
        operation: String,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        key: Option<String>,
    },
    
    /// Operator not found
    #[error("Operator not found: {name}")]
    OperatorNotFound {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        suggestion: Option<String>,
    },
    
    /// Invalid operator data
    #[error("Invalid operator data: {field} - {message}")]
    InvalidData {
        field: String,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<String>,
    },
    
    /// Scanning operation failed
    #[error("Operator scan failed: {reason}")]
    ScanFailed {
        reason: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        operators_processed: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        partial_results: Option<bool>,
    },
    
    /// Serialization/deserialization error
    #[error("Serialization error: {operation} - {message}")]
    Serialization {
        operation: String,
        message: String,
    },
    
    /// Configuration error
    #[error("Configuration error: {setting} - {message}")]
    Configuration {
        setting: String,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        expected_value: Option<String>,
    },
    
    /// Resource access error (file system, network, etc.)
    #[error("Resource access error: {resource} - {message}")]
    ResourceAccess {
        resource: String,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        resource_type: Option<String>,
    },
    
    /// Validation error
    #[error("Validation error: {field} - {message}")]
    Validation {
        field: String,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        expected_format: Option<String>,
    },
    
    /// Operation timeout
    #[error("Operation timed out: {operation} after {timeout_ms}ms")]
    Timeout {
        operation: String,
        timeout_ms: u64,
    },
    
    /// Concurrency error (lock contention, etc.)
    #[error("Concurrency error: {operation} - {message}")]
    Concurrency {
        operation: String,
        message: String,
    },
    
    /// Internal error (unexpected conditions)
    #[error("Internal error: {component} - {message}")]
    Internal {
        component: String,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        details: Option<String>,
    },
    
    /// Dependency error (missing modules, services, etc.)
    #[error("Dependency error: {dependency} - {message}")]
    Dependency {
        dependency: String,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        required_version: Option<String>,
    },
}

impl OperatorError {
    /// Create a MAA operation error
    pub fn maa_operation(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::MaaOperation {
            operation: operation.into(),
            message: message.into(),
            error_code: None,
        }
    }
    
    /// Create a MAA operation error with error code
    pub fn maa_operation_with_code(operation: impl Into<String>, message: impl Into<String>, error_code: impl Into<String>) -> Self {
        Self::MaaOperation {
            operation: operation.into(),
            message: message.into(),
            error_code: Some(error_code.into()),
        }
    }
    
    /// Create a cache operation error
    pub fn cache(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Cache {
            operation: operation.into(),
            message: message.into(),
            key: None,
        }
    }
    
    /// Create a cache operation error with key
    pub fn cache_with_key(operation: impl Into<String>, message: impl Into<String>, key: impl Into<String>) -> Self {
        Self::Cache {
            operation: operation.into(),
            message: message.into(),
            key: Some(key.into()),
        }
    }
    
    /// Create an operator not found error
    pub fn operator_not_found(name: impl Into<String>) -> Self {
        Self::OperatorNotFound {
            name: name.into(),
            suggestion: None,
        }
    }
    
    /// Create an operator not found error with suggestion
    pub fn operator_not_found_with_suggestion(name: impl Into<String>, suggestion: impl Into<String>) -> Self {
        Self::OperatorNotFound {
            name: name.into(),
            suggestion: Some(suggestion.into()),
        }
    }
    
    /// Create an invalid data error
    pub fn invalid_data(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidData {
            field: field.into(),
            message: message.into(),
            value: None,
        }
    }
    
    /// Create an invalid data error with value
    pub fn invalid_data_with_value(field: impl Into<String>, message: impl Into<String>, value: impl Into<String>) -> Self {
        Self::InvalidData {
            field: field.into(),
            message: message.into(),
            value: Some(value.into()),
        }
    }
    
    /// Create a scan failed error
    pub fn scan_failed(reason: impl Into<String>) -> Self {
        Self::ScanFailed {
            reason: reason.into(),
            operators_processed: None,
            partial_results: None,
        }
    }
    
    /// Create a scan failed error with partial results
    pub fn scan_failed_partial(reason: impl Into<String>, operators_processed: u32) -> Self {
        Self::ScanFailed {
            reason: reason.into(),
            operators_processed: Some(operators_processed),
            partial_results: Some(true),
        }
    }
    
    /// Create a serialization error
    pub fn serialization(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Serialization {
            operation: operation.into(),
            message: message.into(),
        }
    }
    
    /// Create a configuration error
    pub fn configuration(setting: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Configuration {
            setting: setting.into(),
            message: message.into(),
            expected_value: None,
        }
    }
    
    /// Create a configuration error with expected value
    pub fn configuration_with_expected(setting: impl Into<String>, message: impl Into<String>, expected_value: impl Into<String>) -> Self {
        Self::Configuration {
            setting: setting.into(),
            message: message.into(),
            expected_value: Some(expected_value.into()),
        }
    }
    
    /// Create a resource access error
    pub fn resource_access(resource: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ResourceAccess {
            resource: resource.into(),
            message: message.into(),
            resource_type: None,
        }
    }
    
    /// Create a resource access error with type
    pub fn resource_access_with_type(resource: impl Into<String>, message: impl Into<String>, resource_type: impl Into<String>) -> Self {
        Self::ResourceAccess {
            resource: resource.into(),
            message: message.into(),
            resource_type: Some(resource_type.into()),
        }
    }
    
    /// Create a validation error
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
            expected_format: None,
        }
    }
    
    /// Create a validation error with expected format
    pub fn validation_with_format(field: impl Into<String>, message: impl Into<String>, expected_format: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
            expected_format: Some(expected_format.into()),
        }
    }
    
    /// Create a timeout error
    pub fn timeout(operation: impl Into<String>, timeout_ms: u64) -> Self {
        Self::Timeout {
            operation: operation.into(),
            timeout_ms,
        }
    }
    
    /// Create a concurrency error
    pub fn concurrency(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Concurrency {
            operation: operation.into(),
            message: message.into(),
        }
    }
    
    /// Create an internal error
    pub fn internal(component: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Internal {
            component: component.into(),
            message: message.into(),
            details: None,
        }
    }
    
    /// Create an internal error with details
    pub fn internal_with_details(component: impl Into<String>, message: impl Into<String>, details: impl Into<String>) -> Self {
        Self::Internal {
            component: component.into(),
            message: message.into(),
            details: Some(details.into()),
        }
    }
    
    /// Create a dependency error
    pub fn dependency(dependency: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Dependency {
            dependency: dependency.into(),
            message: message.into(),
            required_version: None,
        }
    }
    
    /// Create a dependency error with required version
    pub fn dependency_with_version(dependency: impl Into<String>, message: impl Into<String>, required_version: impl Into<String>) -> Self {
        Self::Dependency {
            dependency: dependency.into(),
            message: message.into(),
            required_version: Some(required_version.into()),
        }
    }
    
    /// Get the error category for logging/metrics
    pub fn category(&self) -> &'static str {
        match self {
            Self::MaaOperation { .. } => "maa_operation",
            Self::Cache { .. } => "cache",
            Self::OperatorNotFound { .. } => "not_found",
            Self::InvalidData { .. } => "invalid_data",
            Self::ScanFailed { .. } => "scan_failed",
            Self::Serialization { .. } => "serialization",
            Self::Configuration { .. } => "configuration",
            Self::ResourceAccess { .. } => "resource_access",
            Self::Validation { .. } => "validation",
            Self::Timeout { .. } => "timeout",
            Self::Concurrency { .. } => "concurrency",
            Self::Internal { .. } => "internal",
            Self::Dependency { .. } => "dependency",
        }
    }
    
    /// Check if this error indicates a recoverable condition
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::MaaOperation { .. } => true,  // MAA operations can often be retried
            Self::Cache { .. } => true,         // Cache operations can be retried
            Self::OperatorNotFound { .. } => false, // Operator doesn't exist
            Self::InvalidData { .. } => false,  // Data is fundamentally invalid
            Self::ScanFailed { partial_results: Some(true), .. } => true, // Partial scan can be retried
            Self::ScanFailed { .. } => true,    // Scan can be retried
            Self::Serialization { .. } => false, // Serialization issues are usually permanent
            Self::Configuration { .. } => false, // Configuration needs to be fixed
            Self::ResourceAccess { .. } => true, // Resource might become available
            Self::Validation { .. } => false,   // Validation errors need input correction
            Self::Timeout { .. } => true,       // Timeouts can be retried
            Self::Concurrency { .. } => true,   // Lock contention can be retried
            Self::Internal { .. } => false,     // Internal errors need investigation
            Self::Dependency { .. } => false,   // Dependencies need to be installed
        }
    }
    
    /// Check if this error should be logged at error level
    pub fn is_error_level(&self) -> bool {
        match self {
            Self::MaaOperation { .. } => true,
            Self::Cache { .. } => true,
            Self::OperatorNotFound { .. } => false, // INFO level
            Self::InvalidData { .. } => true,
            Self::ScanFailed { .. } => true,
            Self::Serialization { .. } => true,
            Self::Configuration { .. } => true,
            Self::ResourceAccess { .. } => true,
            Self::Validation { .. } => false,  // WARN level
            Self::Timeout { .. } => true,
            Self::Concurrency { .. } => false, // WARN level
            Self::Internal { .. } => true,
            Self::Dependency { .. } => true,
        }
    }
}

/// Result type for operator manager operations
pub type OperatorResult<T> = Result<T, OperatorError>;

/// Convert from MAA adapter errors
impl From<crate::maa_adapter::MaaError> for OperatorError {
    fn from(error: crate::maa_adapter::MaaError) -> Self {
        match error {
            crate::maa_adapter::MaaError::Connection { message, .. } => {
                Self::maa_operation("connection", message)
            },
            crate::maa_adapter::MaaError::TaskExecution { reason, .. } => {
                Self::maa_operation("task", reason)
            },
            crate::maa_adapter::MaaError::Ffi { message, .. } => {
                Self::maa_operation("ffi", message)
            },
            crate::maa_adapter::MaaError::Timeout { operation, timeout_ms } => {
                Self::timeout(operation, timeout_ms)
            },
            crate::maa_adapter::MaaError::Configuration { message, .. } => {
                Self::configuration("maa_config", message)
            },
            crate::maa_adapter::MaaError::Resource { message, .. } => {
                Self::resource_access("maa_resource", message)
            },
            crate::maa_adapter::MaaError::Callback { message, .. } => {
                Self::internal("maa_callback", message)
            },
            crate::maa_adapter::MaaError::Serialization { message, .. } => {
                Self::serialization("maa_data", message)
            },
            _ => {
                Self::internal("maa_adapter", error.to_string())
            }
        }
    }
}

/// Convert from sled database errors
impl From<sled::Error> for OperatorError {
    fn from(error: sled::Error) -> Self {
        match error {
            sled::Error::Io(io_error) => {
                Self::cache("io", format!("Database I/O error: {}", io_error))
            },
            sled::Error::Corruption { at, .. } => {
                Self::cache_with_key("corruption", "Database corruption detected".to_string(), format!("at_{:?}", at))
            },
            sled::Error::ReportableBug(msg) => {
                Self::internal("sled", format!("Database bug: {}", msg))
            },
            sled::Error::CollectionNotFound(name) => {
                Self::cache_with_key("collection_not_found", "Collection not found".to_string(), 
                    String::from_utf8_lossy(&name).to_string())
            },
            sled::Error::Unsupported(msg) => {
                Self::dependency("sled", format!("Unsupported operation: {}", msg))
            },
        }
    }
}

/// Convert from serde JSON errors
impl From<serde_json::Error> for OperatorError {
    fn from(error: serde_json::Error) -> Self {
        let operation = if error.is_data() {
            "deserialize"
        } else if error.is_syntax() {
            "parse"
        } else if error.is_eof() {
            "read"
        } else {
            "unknown"
        };
        
        Self::serialization(operation, error.to_string())
    }
}

/// Convert from standard I/O errors
impl From<std::io::Error> for OperatorError {
    fn from(error: std::io::Error) -> Self {
        let resource_type = match error.kind() {
            std::io::ErrorKind::NotFound => "file",
            std::io::ErrorKind::PermissionDenied => "permission",
            std::io::ErrorKind::TimedOut => "network",
            _ => "io",
        };
        
        Self::resource_access_with_type("file_system", error.to_string(), resource_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_creation() {
        let error = OperatorError::maa_operation("screenshot", "device disconnected");
        assert_eq!(error.category(), "maa_operation");
        assert!(error.is_recoverable());
        assert!(error.is_error_level());
        assert!(error.to_string().contains("MAA operation failed"));
    }
    
    #[test]
    fn test_operator_not_found() {
        let error = OperatorError::operator_not_found("UnknownOperator");
        assert_eq!(error.category(), "not_found");
        assert!(!error.is_recoverable());
        assert!(!error.is_error_level());
        assert!(error.to_string().contains("Operator not found"));
    }
    
    #[test]
    fn test_cache_error() {
        let error = OperatorError::cache_with_key("get", "key not found", "operators:amiya");
        assert_eq!(error.category(), "cache");
        assert!(error.is_recoverable());
        
        if let OperatorError::Cache { key, .. } = error {
            assert_eq!(key, Some("operators:amiya".to_string()));
        } else {
            panic!("Expected Cache error");
        }
    }
    
    #[test]
    fn test_scan_failed_partial() {
        let error = OperatorError::scan_failed_partial("network timeout", 5);
        assert_eq!(error.category(), "scan_failed");
        assert!(error.is_recoverable());
        
        if let OperatorError::ScanFailed { operators_processed, partial_results, .. } = error {
            assert_eq!(operators_processed, Some(5));
            assert_eq!(partial_results, Some(true));
        } else {
            panic!("Expected ScanFailed error");
        }
    }
    
    #[test]
    fn test_validation_error() {
        let error = OperatorError::validation_with_format("skill_level", "must be between 1-7", "1-7");
        assert_eq!(error.category(), "validation");
        assert!(!error.is_recoverable());
        assert!(!error.is_error_level());
    }
    
    #[test]
    fn test_timeout_error() {
        let error = OperatorError::timeout("operator_scan", 30000);
        assert_eq!(error.category(), "timeout");
        assert!(error.is_recoverable());
        assert!(error.is_error_level());
        assert!(error.to_string().contains("30000ms"));
    }
    
    #[test]
    fn test_internal_error_with_details() {
        let error = OperatorError::internal_with_details("scanner", "unexpected state", "state=invalid");
        assert_eq!(error.category(), "internal");
        assert!(!error.is_recoverable());
        assert!(error.is_error_level());
        
        if let OperatorError::Internal { details, .. } = error {
            assert_eq!(details, Some("state=invalid".to_string()));
        } else {
            panic!("Expected Internal error");
        }
    }
    
    #[test]
    fn test_serialization() {
        let error = OperatorError::validation("test_field", "test message");
        let serialized = serde_json::to_string(&error).unwrap();
        let deserialized: OperatorError = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(error.category(), deserialized.category());
        assert_eq!(error.to_string(), deserialized.to_string());
    }
    
    #[test]
    fn test_error_conversion_from_serde() {
        let json_error = serde_json::from_str::<serde_json::Value>("{invalid json").unwrap_err();
        let op_error: OperatorError = json_error.into();
        
        assert_eq!(op_error.category(), "serialization");
        assert!(op_error.to_string().contains("Serialization error"));
    }
}