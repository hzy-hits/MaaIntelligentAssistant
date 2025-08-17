//! Simple backend functionality test

use maa_intelligent_server::maa_adapter::{MaaBackend, BackendConfig};

#[test]
fn test_backend_config_creation() {
    let config = BackendConfig::default();
    assert!(config.prefer_real);
    assert!(!config.force_stub);
    assert!(!config.resource_path.is_empty());
    println!("✓ BackendConfig created successfully");
}

#[test]
fn test_backend_type_checking() {
    let config = BackendConfig {
        force_stub: true,
        prefer_real: false,
        resource_path: "./test".to_string(),
        verbose: false,
    };
    
    let backend = MaaBackend::new(config).expect("Backend creation should succeed");
    assert!(backend.is_stub());
    assert!(!backend.is_real());
    assert_eq!(backend.backend_type(), "stub");
    println!("✓ Backend type checking works");
}

#[test]
fn test_version_info() {
    let version = MaaBackend::get_version().expect("Should get version");
    assert!(!version.is_empty());
    println!("✓ Version: {}", version);
}

#[test]
fn test_logging() {
    let result = MaaBackend::log("INFO", "Test message");
    assert!(result.is_ok());
    println!("✓ Logging works");
}