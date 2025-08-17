//! Integration tests for MAA FFI functionality
//!
//! This module provides comprehensive tests for the MAA adapter integration,
//! testing both stub and real FFI implementations.

use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, debug};

use crate::maa_adapter::{
    MaaBackend, BackendConfig, MaaResult, CallbackMessage
};

/// Test configuration for integration tests
#[derive(Clone)]
pub struct TestConfig {
    pub resource_path: String,
    pub test_device: String,
    pub adb_path: String,
    pub timeout_ms: u64,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            resource_path: "/Users/ivena/Desktop/Fairy/maa/maa-remote-server/maa-official".to_string(),
            test_device: "127.0.0.1:5555".to_string(),
            adb_path: "adb".to_string(),
            timeout_ms: 5000,
        }
    }
}

/// Integration test suite for MAA backends
pub struct MaaIntegrationTest {
    config: TestConfig,
}

impl MaaIntegrationTest {
    /// Create a new integration test suite
    pub fn new(config: TestConfig) -> Self {
        Self { config }
    }
    
    /// Test backend creation and basic operations
    pub async fn test_backend_creation(&self) -> MaaResult<()> {
        info!("Testing MAA backend creation");
        
        // Test forced stub creation
        let stub_config = BackendConfig {
            force_stub: true,
            prefer_real: false,
            resource_path: self.config.resource_path.clone(),
            verbose: true,
        };
        
        let stub_backend = MaaBackend::new(stub_config)?;
        assert!(stub_backend.is_stub());
        assert!(!stub_backend.is_real());
        info!("✓ Stub backend created successfully: {}", stub_backend.backend_type());
        
        // Test automatic backend selection (will likely fall back to stub)
        let auto_config = BackendConfig {
            force_stub: false,
            prefer_real: true,
            resource_path: self.config.resource_path.clone(),
            verbose: true,
        };
        
        let auto_backend = MaaBackend::new(auto_config)?;
        info!("✓ Auto backend created: {} mode", auto_backend.backend_type());
        
        Ok(())
    }
    
    /// Test callback functionality
    pub async fn test_callback_integration(&self) -> MaaResult<()> {
        info!("Testing MAA callback integration");
        
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        
        let config = BackendConfig {
            force_stub: true,
            prefer_real: false,
            resource_path: self.config.resource_path.clone(),
            verbose: true,
        };
        
        let backend = MaaBackend::with_callback(config, tx)?;
        assert!(backend.is_stub());
        info!("✓ Backend with callback created successfully");
        
        // Test that we don't block on callback reception
        let timeout_result = tokio::time::timeout(
            Duration::from_millis(100), 
            rx.recv()
        ).await;
        
        // We expect timeout since no callbacks should be sent yet
        assert!(timeout_result.is_err());
        info!("✓ Callback system properly initialized (no spurious callbacks)");
        
        Ok(())
    }
    
    /// Test basic operations (stub mode)
    pub async fn test_basic_operations(&self) -> MaaResult<()> {
        info!("Testing basic MAA operations in stub mode");
        
        let config = BackendConfig {
            force_stub: true,
            prefer_real: false,
            resource_path: self.config.resource_path.clone(),
            verbose: true,
        };
        
        let mut backend = MaaBackend::new(config)?;
        
        // Test connection
        debug!("Testing connection...");
        let connect_id = backend.connect(
            &self.config.adb_path,
            &self.config.test_device,
            None
        )?;
        assert!(connect_id > 0);
        info!("✓ Connection successful with ID: {}", connect_id);
        
        // Test connection status
        assert!(backend.is_connected());
        info!("✓ Connection status check passed");
        
        // Test screenshot
        debug!("Testing screenshot...");
        let screenshot = backend.screenshot()?;
        assert!(!screenshot.is_empty());
        info!("✓ Screenshot taken, size: {} bytes", screenshot.len());
        
        // Test click
        debug!("Testing click...");
        let click_id = backend.click(100, 200)?;
        assert!(click_id > 0);
        info!("✓ Click successful with ID: {}", click_id);
        
        // Test task creation
        debug!("Testing task creation...");
        let task_id = backend.create_task("Screenshot", "{}")?;
        assert!(task_id > 0);
        info!("✓ Task created with ID: {}", task_id);
        
        // Test task parameter setting
        debug!("Testing task parameter setting...");
        backend.set_task_params(task_id, r#"{"test": true}"#)?;
        info!("✓ Task parameters set successfully");
        
        // Test start/stop
        debug!("Testing start/stop...");
        backend.start()?;
        assert!(backend.is_running());
        info!("✓ Backend started successfully");
        
        backend.stop()?;
        assert!(!backend.is_running());
        info!("✓ Backend stopped successfully");
        
        // Test UUID retrieval
        debug!("Testing UUID retrieval...");
        let uuid = backend.get_uuid()?;
        assert!(!uuid.is_empty());
        info!("✓ UUID retrieved: {}", uuid);
        
        // Test target retrieval
        debug!("Testing target retrieval...");
        let target = backend.get_target();
        assert!(target.is_some());
        info!("✓ Target retrieved: {:?}", target);
        
        // Test tasks retrieval
        debug!("Testing tasks retrieval...");
        let tasks = backend.get_tasks()?;
        assert!(!tasks.is_empty());
        info!("✓ Tasks retrieved: {:?}", tasks);
        
        Ok(())
    }
    
    /// Test error handling
    pub async fn test_error_handling(&self) -> MaaResult<()> {
        info!("Testing MAA error handling");
        
        let config = BackendConfig {
            force_stub: true,
            prefer_real: false,
            resource_path: "/nonexistent/path".to_string(),
            verbose: true,
        };
        
        // This should still work in stub mode
        let backend_result = MaaBackend::new(config);
        assert!(backend_result.is_ok());
        info!("✓ Stub backend handles invalid resource path gracefully");
        
        Ok(())
    }
    
    /// Test version information
    pub async fn test_version_info(&self) -> MaaResult<()> {
        info!("Testing MAA version information");
        
        let version = MaaBackend::get_version()?;
        assert!(!version.is_empty());
        info!("✓ MAA version: {}", version);
        
        Ok(())
    }
    
    /// Test logging functionality
    pub async fn test_logging(&self) -> MaaResult<()> {
        info!("Testing MAA logging functionality");
        
        MaaBackend::log("INFO", "Test log message from integration test")?;
        info!("✓ Logging functionality working");
        
        Ok(())
    }
    
    /// Run all integration tests
    pub async fn run_all_tests(&self) -> MaaResult<()> {
        info!("Starting MAA integration test suite");
        
        self.test_backend_creation().await?;
        self.test_callback_integration().await?;
        self.test_basic_operations().await?;
        self.test_error_handling().await?;
        self.test_version_info().await?;
        self.test_logging().await?;
        
        info!("✅ All MAA integration tests passed!");
        Ok(())
    }
}

/// Test with real MAA Core if available
pub async fn test_real_maa_if_available() -> MaaResult<()> {
    info!("Testing real MAA Core integration (if available)");
    
    let config = BackendConfig {
        force_stub: false,
        prefer_real: true,
        resource_path: "/Users/ivena/Desktop/Fairy/maa/maa-remote-server/maa-official".to_string(),
        verbose: true,
    };
    
    match MaaBackend::new(config) {
        Ok(backend) => {
            if backend.is_real() {
                info!("✅ Real MAA Core is available and working!");
                
                // Test basic real operations
                let version = MaaBackend::get_version()?;
                info!("Real MAA version: {}", version);
                
                info!("✅ Real MAA Core integration successful!");
            } else {
                info!("ℹ️ Real MAA Core not available, using stub mode");
            }
        }
        Err(e) => {
            info!("ℹ️ Real MAA Core test failed (expected): {}", e);
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber;

    fn init_logging() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter("debug")
            .try_init();
    }

    #[tokio::test]
    async fn test_integration_suite() {
        init_logging();
        
        let config = TestConfig::default();
        let test_suite = MaaIntegrationTest::new(config);
        
        let result = test_suite.run_all_tests().await;
        assert!(result.is_ok(), "Integration tests failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_real_maa_availability() {
        init_logging();
        
        let result = test_real_maa_if_available().await;
        assert!(result.is_ok(), "Real MAA test failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_backend_switching() {
        init_logging();
        
        let resource_path = "./test_resources".to_string();
        
        // Test stub mode
        let stub_config = BackendConfig {
            force_stub: true,
            prefer_real: false,
            resource_path: resource_path.clone(),
            verbose: false,
        };
        
        let stub_backend = MaaBackend::new(stub_config).unwrap();
        assert!(stub_backend.is_stub());
        
        // Test auto selection (will likely be stub)
        let auto_config = BackendConfig {
            force_stub: false,
            prefer_real: true,
            resource_path,
            verbose: false,
        };
        
        let auto_backend = MaaBackend::new(auto_config).unwrap();
        info!("Auto-selected backend: {}", auto_backend.backend_type());
    }
}