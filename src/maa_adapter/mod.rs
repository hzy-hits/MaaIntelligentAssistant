//! MAA Adapter Module
//!
//! This module provides a safe, async wrapper around the MAA (MaaAssistantArknights) FFI bindings.
//! It handles the complexity of FFI callbacks, resource management, and provides a clean async interface
//! for interacting with MAA functionality.
//!
//! # Architecture
//!
//! - **Core**: Main MaaAdapter implementation with thread-safe operations
//! - **Types**: Data structures for MAA operations and configurations
//! - **Errors**: Unified error handling for all MAA operations
//! - **Callbacks**: FFI callback handling with tokio channel integration
//!
//! # Example
//!
//! ```rust
//! use maa_intelligent_server::maa_adapter::{MaaAdapter, MaaAdapterTrait, MaaConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = MaaConfig::default();
//!     let mut adapter = MaaAdapter::new(config).await?;
//!     
//!     adapter.connect("emulator").await?;
//!     let screenshot = adapter.screenshot().await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod types;
pub mod errors;
pub mod callbacks;
pub mod core;
pub mod ffi_wrapper;
pub mod ffi_stub;
pub mod ffi_bindings;

// Re-export public API
pub use types::{
    MaaConfig, MaaStatus, MaaTask, MaaTaskType, MaaController,
    MaaResource, MaaCallback, TaskParams, DeviceInfo
};
pub use errors::{MaaError, MaaResult};
pub use callbacks::CallbackHandler;
pub use types::CallbackMessage;
pub use core::{MaaAdapter, MaaAdapterTrait};
pub use ffi_wrapper::{MaaFFIWrapper, ConnectionParams};

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