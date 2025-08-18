//! MAA Adapter Module - Simplified
//!
//! This module provides essential types and error handling for MAA operations.
//! The actual MAA functionality has been moved to maa_core module for simplicity.

pub mod types;
pub mod errors;
pub mod ffi_stub;

// Re-export essential types only
pub use types::{MaaConfig, MaaStatus, TaskParams};
pub use errors::{MaaError, MaaResult};

// Constants for MAA operations
pub const DEFAULT_TIMEOUT_MS: u64 = 30000;
pub const MAX_RETRY_ATTEMPTS: u32 = 3;
pub const CALLBACK_BUFFER_SIZE: usize = 1000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Test that all expected types are available
        let _config: MaaConfig = MaaConfig::default();
        let _status: MaaStatus = MaaStatus::Idle;
        let _error: MaaError = MaaError::connection("test");
    }
}