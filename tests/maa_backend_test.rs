//! Basic MAA backend functionality test

use maa_intelligent_server::maa_adapter::{MaaBackend, BackendConfig, MaaResult};

#[tokio::test]
async fn test_stub_backend_creation() -> MaaResult<()> {
    let config = BackendConfig {
        force_stub: true,
        prefer_real: false,
        resource_path: "./test_resources".to_string(),
        verbose: false,
    };
    
    let backend = MaaBackend::new(config)?;
    assert!(backend.is_stub());
    assert!(!backend.is_real());
    assert_eq!(backend.backend_type(), "stub");
    
    println!("✓ Stub backend created successfully");
    Ok(())
}

#[tokio::test]
async fn test_backend_basic_operations() -> MaaResult<()> {
    let config = BackendConfig {
        force_stub: true,
        prefer_real: false,
        resource_path: "./test_resources".to_string(),
        verbose: false,
    };
    
    let mut backend = MaaBackend::new(config)?;
    
    // Test connection
    let connect_id = backend.connect("adb", "127.0.0.1:5555", None)?;
    assert!(connect_id > 0);
    assert!(backend.is_connected());
    println!("✓ Connection successful");
    
    // Test screenshot
    let screenshot = backend.screenshot()?;
    assert!(!screenshot.is_empty());
    println!("✓ Screenshot taken, {} bytes", screenshot.len());
    
    // Test click
    let click_id = backend.click(100, 200)?;
    assert!(click_id > 0);
    println!("✓ Click successful");
    
    // Test task management
    let task_id = backend.create_task("Screenshot", "{}")?;
    assert!(task_id > 0);
    println!("✓ Task created: {}", task_id);
    
    backend.set_task_params(task_id, r#"{"test": true}"#)?;
    println!("✓ Task parameters set");
    
    // Test start/stop
    backend.start()?;
    assert!(backend.is_running());
    println!("✓ Backend started");
    
    backend.stop()?;
    assert!(!backend.is_running());
    println!("✓ Backend stopped");
    
    // Test UUID and target
    let uuid = backend.get_uuid()?;
    assert!(!uuid.is_empty());
    println!("✓ UUID: {}", uuid);
    
    let target = backend.get_target();
    assert!(target.is_some());
    println!("✓ Target: {:?}", target);
    
    Ok(())
}

#[tokio::test]
async fn test_version_and_logging() -> MaaResult<()> {
    let version = MaaBackend::get_version()?;
    assert!(!version.is_empty());
    println!("✓ Version: {}", version);
    
    MaaBackend::log("INFO", "Test log message")?;
    println!("✓ Logging works");
    
    Ok(())
}

#[tokio::test]
async fn test_auto_backend_selection() -> MaaResult<()> {
    let config = BackendConfig {
        force_stub: false,
        prefer_real: true,
        resource_path: "./test_resources".to_string(),
        verbose: false,
    };
    
    let backend = MaaBackend::new(config)?;
    println!("✓ Auto-selected backend type: {}", backend.backend_type());
    
    // Should work regardless of which backend was selected
    let version = MaaBackend::get_version()?;
    assert!(!version.is_empty());
    println!("✓ Version from auto backend: {}", version);
    
    Ok(())
}