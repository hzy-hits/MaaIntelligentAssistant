//! MAA Backend Abstraction
//!
//! This module provides a unified interface that can switch between different MAA backends:
//! - Real FFI: Uses actual maa-sys for real MAA Core integration
//! - Stub: Uses mock implementation for development and testing

use tracing::{debug, info, warn};

use crate::maa_adapter::{MaaResult, CallbackMessage};
use crate::maa_adapter::ffi_real::MaaFFIReal;
use crate::maa_adapter::ffi_stub::MaaFFIStub;

/// MAA Backend abstraction
pub enum MaaBackend {
    /// Real MAA Core FFI implementation
    Real(MaaFFIReal),
    /// Stub implementation for development/testing
    Stub(MaaFFIStub),
}

/// Configuration for backend selection
#[derive(Debug, Clone)]
pub struct BackendConfig {
    /// Prefer real MAA if available
    pub prefer_real: bool,
    /// Force stub mode (for testing)
    pub force_stub: bool,
    /// Resource path for MAA
    pub resource_path: String,
    /// Enable verbose logging
    pub verbose: bool,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            prefer_real: true,
            force_stub: false,
            resource_path: "/Users/ivena/Desktop/Fairy/maa/maa-remote-server/maa-official".to_string(),
            verbose: false,
        }
    }
}

impl MaaBackend {
    /// Create a new MAA backend with automatic selection
    pub fn new(config: BackendConfig) -> MaaResult<Self> {
        if config.force_stub {
            info!("Creating MAA backend: Forced stub mode");
            let stub = MaaFFIStub::new(config.resource_path)?;
            return Ok(MaaBackend::Stub(stub));
        }
        
        if config.prefer_real {
            debug!("Attempting to create real MAA backend");
            match MaaFFIReal::new(config.resource_path.clone()) {
                Ok(real) => {
                    info!("Created MAA backend: Real FFI mode");
                    return Ok(MaaBackend::Real(real));
                }
                Err(e) => {
                    warn!("Failed to create real MAA backend, falling back to stub: {}", e);
                    let stub = MaaFFIStub::new(config.resource_path)?;
                    return Ok(MaaBackend::Stub(stub));
                }
            }
        }
        
        debug!("Creating MAA backend: Stub mode");
        let stub = MaaFFIStub::new(config.resource_path)?;
        Ok(MaaBackend::Stub(stub))
    }
    
    /// Create a new MAA backend with callback support
    pub fn with_callback(
        config: BackendConfig,
        callback_sender: tokio::sync::mpsc::UnboundedSender<CallbackMessage>,
    ) -> MaaResult<Self> {
        if config.force_stub {
            info!("Creating MAA backend with callback: Forced stub mode");
            let stub = MaaFFIStub::with_callback(config.resource_path, callback_sender)?;
            return Ok(MaaBackend::Stub(stub));
        }
        
        if config.prefer_real {
            debug!("Attempting to create real MAA backend with callback");
            match MaaFFIReal::with_callback(config.resource_path.clone(), callback_sender.clone()) {
                Ok(real) => {
                    info!("Created MAA backend with callback: Real FFI mode");
                    return Ok(MaaBackend::Real(real));
                }
                Err(e) => {
                    warn!("Failed to create real MAA backend, falling back to stub: {}", e);
                    let stub = MaaFFIStub::with_callback(config.resource_path, callback_sender)?;
                    return Ok(MaaBackend::Stub(stub));
                }
            }
        }
        
        debug!("Creating MAA backend with callback: Stub mode");
        let stub = MaaFFIStub::with_callback(config.resource_path, callback_sender)?;
        Ok(MaaBackend::Stub(stub))
    }
    
    /// Get backend type as string
    pub fn backend_type(&self) -> &'static str {
        match self {
            MaaBackend::Real(_) => "real",
            MaaBackend::Stub(_) => "stub",
        }
    }
    
    /// Check if using real MAA backend
    pub fn is_real(&self) -> bool {
        matches!(self, MaaBackend::Real(_))
    }
    
    /// Check if using stub backend
    pub fn is_stub(&self) -> bool {
        matches!(self, MaaBackend::Stub(_))
    }
}

// Delegate all methods to the appropriate backend implementation
impl MaaBackend {
    /// Connect to device
    pub fn connect(&mut self, adb_path: &str, device_address: &str, config: Option<&str>) -> MaaResult<i32> {
        match self {
            MaaBackend::Real(real) => real.connect(adb_path, device_address, config),
            MaaBackend::Stub(stub) => stub.connect(adb_path, device_address, config),
        }
    }
    
    /// Check if MAA is currently running
    pub fn is_running(&self) -> bool {
        match self {
            MaaBackend::Real(real) => real.is_running(),
            MaaBackend::Stub(stub) => stub.is_running(),
        }
    }
    
    /// Check if connected to device
    pub fn is_connected(&self) -> bool {
        match self {
            MaaBackend::Real(real) => real.is_connected(),
            MaaBackend::Stub(stub) => stub.is_connected(),
        }
    }
    
    /// Take a screenshot
    pub fn screenshot(&self) -> MaaResult<Vec<u8>> {
        match self {
            MaaBackend::Real(real) => real.screenshot(),
            MaaBackend::Stub(stub) => stub.screenshot(),
        }
    }
    
    /// Click at specified coordinates
    pub fn click(&self, x: i32, y: i32) -> MaaResult<i32> {
        match self {
            MaaBackend::Real(real) => real.click(x, y),
            MaaBackend::Stub(stub) => stub.click(x, y),
        }
    }
    
    /// Create a new task
    pub fn create_task(&mut self, task_type: &str, params: &str) -> MaaResult<i32> {
        match self {
            MaaBackend::Real(real) => real.create_task(task_type, params),
            MaaBackend::Stub(stub) => stub.create_task(task_type, params),
        }
    }
    
    /// Set task parameters
    pub fn set_task_params(&self, task_id: i32, params: &str) -> MaaResult<()> {
        match self {
            MaaBackend::Real(real) => real.set_task_params(task_id, params),
            MaaBackend::Stub(stub) => stub.set_task_params(task_id, params),
        }
    }
    
    /// Start task execution
    pub fn start(&self) -> MaaResult<()> {
        match self {
            MaaBackend::Real(real) => real.start(),
            MaaBackend::Stub(stub) => stub.start(),
        }
    }
    
    /// Stop task execution
    pub fn stop(&self) -> MaaResult<()> {
        match self {
            MaaBackend::Real(real) => real.stop(),
            MaaBackend::Stub(stub) => stub.stop(),
        }
    }
    
    /// Get device UUID
    pub fn get_uuid(&mut self) -> MaaResult<String> {
        match self {
            MaaBackend::Real(real) => real.get_uuid(),
            MaaBackend::Stub(stub) => stub.get_uuid(),
        }
    }
    
    /// Get current target device
    pub fn get_target(&self) -> Option<String> {
        match self {
            MaaBackend::Real(real) => real.get_target(),
            MaaBackend::Stub(stub) => stub.get_target(),
        }
    }
    
    /// Get all active tasks
    pub fn get_tasks(&self) -> MaaResult<Vec<i32>> {
        match self {
            MaaBackend::Real(real) => real.get_tasks(),
            MaaBackend::Stub(stub) => stub.get_tasks(),
        }
    }
    
    /// Get MAA version
    pub fn get_version() -> MaaResult<String> {
        // Try real version first, fallback to stub
        #[cfg(feature = "with-maa-core")]
        {
            match MaaFFIReal::get_version() {
                Ok(version) => return Ok(version),
                Err(_) => {} // Fall through to stub
            }
        }
        
        MaaFFIStub::get_version()
    }
    
    /// Log message via MAA
    pub fn log(level: &str, message: &str) -> MaaResult<()> {
        // Try real logging first, fallback to stub
        #[cfg(feature = "with-maa-core")]
        {
            match MaaFFIReal::log(level, message) {
                Ok(()) => return Ok(()),
                Err(_) => {} // Fall through to stub
            }
        }
        
        MaaFFIStub::log(level, message)
    }
    
    /// Go back to home screen
    pub fn back_to_home(&self) -> MaaResult<()> {
        match self {
            MaaBackend::Real(real) => real.back_to_home(),
            MaaBackend::Stub(stub) => stub.back_to_home(),
        }
    }
}

// Thread safety
unsafe impl Send for MaaBackend {}
unsafe impl Sync for MaaBackend {}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[test]
    fn test_backend_config_default() {
        let config = BackendConfig::default();
        assert!(config.prefer_real);
        assert!(!config.force_stub);
        assert!(!config.resource_path.is_empty());
    }

    #[test]
    fn test_forced_stub_creation() {
        let config = BackendConfig {
            force_stub: true,
            prefer_real: false,
            resource_path: "./test_resources".to_string(),
            verbose: false,
        };
        
        let backend = MaaBackend::new(config).unwrap();
        assert!(backend.is_stub());
        assert!(!backend.is_real());
        assert_eq!(backend.backend_type(), "stub");
    }

    #[tokio::test]
    async fn test_backend_with_callback() {
        let (tx, _rx) = mpsc::unbounded_channel();
        
        let config = BackendConfig {
            force_stub: true,
            prefer_real: false,
            resource_path: "./test_resources".to_string(),
            verbose: false,
        };
        
        let backend = MaaBackend::with_callback(config, tx).unwrap();
        assert!(backend.is_stub());
    }

    #[test]
    fn test_backend_version() {
        // This should work regardless of whether real MAA is available
        let version = MaaBackend::get_version().unwrap();
        assert!(!version.is_empty());
    }
}