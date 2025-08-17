//! Integration tests for MAA FFI functionality (DISABLED)
//!
//! These tests verify that the MAA FFI integration is working correctly.
//! They test both real FFI functionality (when available) and fallback behavior.
//!
//! DISABLED: These tests use old API that has been replaced by the new backend system.

#[cfg(disabled)]
mod disabled_tests {

use maa_intelligent_server::maa_adapter::{
    MaaAdapter, MaaAdapterTrait, MaaConfig, MaaTaskType, TaskParams, MaaFFIWrapper
};
use std::collections::HashMap;
use tokio::time::{timeout, Duration};

/// Create a test configuration
fn create_test_config() -> MaaConfig {
    MaaConfig {
        resource_path: "./maa-official/resource".to_string(),
        adb_path: "adb".to_string(),
        device_address: "127.0.0.1:5555".to_string(),
        connection_type: "ADB".to_string(),
        runtime_mode: maa_intelligent_server::maa_adapter::RuntimeMode::TestOnly,
        options: HashMap::new(),
        timeout_ms: 5000,
        max_retries: 1,
        debug: true,
    }
}

#[tokio::test]
async fn test_adapter_creation_with_ffi() {
    // Test that adapter can be created with FFI support
    let config = create_test_config();
    
    let result = MaaAdapter::new(config).await;
    
    // This should succeed even if MAA resources are not available (fallback to mock)
    assert!(result.is_ok(), "Adapter creation should succeed");
    
    let adapter = result.unwrap();
    let status = adapter.get_status().await.unwrap();
    println!("Adapter status: {:?}", status);
}

#[tokio::test]
async fn test_ffi_wrapper_creation() {
    // Test direct FFI wrapper creation
    let resource_path = "./maa-official/resource".to_string();
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    
    let result = MaaFFIWrapper::with_callback(resource_path, tx);
    
    match result {
        Ok(_wrapper) => {
            println!("FFI wrapper created successfully - MAA resources are available");
        }
        Err(e) => {
            println!("FFI wrapper creation failed (expected in test environment): {}", e);
            // This is expected if MAA resources are not properly set up
        }
    }
}

#[tokio::test]
async fn test_adapter_connection_with_fallback() {
    let config = create_test_config();
    let mut adapter = MaaAdapter::new(config).await.unwrap();
    
    // Test connection (will likely fall back to mock)
    let result = adapter.connect("test_device").await;
    
    assert!(result.is_ok(), "Connection should succeed (with fallback)");
    
    let status = adapter.get_status().await.unwrap();
    println!("Connection status: {:?}", status);
    
    // Test device info
    let device_info = adapter.get_device_info().await.unwrap();
    assert!(device_info.is_some(), "Device info should be available");
    println!("Device info: {:?}", device_info.unwrap());
}

#[tokio::test]
async fn test_adapter_screenshot_with_fallback() {
    let config = create_test_config();
    let mut adapter = MaaAdapter::new(config).await.unwrap();
    
    // Connect first
    adapter.connect("test_device").await.unwrap();
    
    // Test screenshot (will likely fall back to mock)
    let result = adapter.screenshot().await;
    
    assert!(result.is_ok(), "Screenshot should succeed (with fallback)");
    
    let screenshot = result.unwrap();
    assert!(!screenshot.is_empty(), "Screenshot should not be empty");
    println!("Screenshot size: {} bytes", screenshot.len());
}

#[tokio::test]
async fn test_adapter_click_with_fallback() {
    let config = create_test_config();
    let mut adapter = MaaAdapter::new(config).await.unwrap();
    
    // Connect first
    adapter.connect("test_device").await.unwrap();
    
    // Test click (will likely fall back to mock)
    let result = adapter.click(100, 200).await;
    
    assert!(result.is_ok(), "Click should succeed (with fallback)");
    println!("Click operation completed");
}

#[tokio::test]
async fn test_task_management_with_ffi() {
    let config = create_test_config();
    let mut adapter = MaaAdapter::new(config).await.unwrap();
    
    // Connect first
    adapter.connect("test_device").await.unwrap();
    
    // Create a task
    let task_id = adapter.create_task(
        MaaTaskType::Screenshot,
        TaskParams::default()
    ).await.unwrap();
    
    assert!(task_id > 0, "Task ID should be positive");
    println!("Created task with ID: {}", task_id);
    
    // Get task info
    let task = adapter.get_task(task_id).await.unwrap();
    assert!(task.is_some(), "Task should exist");
    println!("Task info: {:?}", task.unwrap());
    
    // Get all tasks
    let all_tasks = adapter.get_all_tasks().await.unwrap();
    assert!(!all_tasks.is_empty(), "Should have at least one task");
    println!("Total tasks: {}", all_tasks.len());
    
    // Start task
    let start_result = adapter.start_task(task_id).await;
    assert!(start_result.is_ok(), "Task start should succeed");
    
    // Wait a bit for task to run
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
    // Check task status after execution
    let final_task = adapter.get_task(task_id).await.unwrap();
    if let Some(task) = final_task {
        println!("Final task status: {:?}", task.status);
    }
}

#[tokio::test]
async fn test_version_info() {
    // Test getting MAA version (if available)
    match MaaFFIWrapper::get_version() {
        Ok(version) => {
            println!("MAA Core version: {}", version);
            assert!(!version.is_empty(), "Version should not be empty");
        }
        Err(e) => {
            println!("Could not get MAA version (expected in test environment): {}", e);
        }
    }
}

#[tokio::test]
async fn test_concurrent_operations() {
    let config = create_test_config();
    let mut adapter = MaaAdapter::new(config).await.unwrap();
    
    // Connect first
    adapter.connect("test_device").await.unwrap();
    
    // Test concurrent operations
    let screenshot_task = adapter.screenshot();
    let click_task = adapter.click(50, 50);
    
    let screenshot_timeout = timeout(Duration::from_secs(5), screenshot_task);
    let click_timeout = timeout(Duration::from_secs(5), click_task);
    
    let (screenshot_result, click_result) = tokio::join!(screenshot_timeout, click_timeout);
    
    assert!(screenshot_result.is_ok(), "Screenshot should complete within timeout");
    assert!(click_result.is_ok(), "Click should complete within timeout");
    
    println!("Concurrent operations completed successfully");
}

#[tokio::test]
async fn test_error_handling() {
    let config = create_test_config();
    let adapter = MaaAdapter::new(config).await.unwrap();
    
    // Test operations without connection (should fail)
    let screenshot_result = adapter.screenshot().await;
    assert!(screenshot_result.is_err(), "Screenshot without connection should fail");
    
    let click_result = adapter.click(100, 200).await;
    assert!(click_result.is_err(), "Click without connection should fail");
    
    println!("Error handling tests passed");
}

#[tokio::test]
async fn test_adapter_lifecycle() {
    let config = create_test_config();
    let mut adapter = MaaAdapter::new(config).await.unwrap();
    
    // Test full lifecycle
    println!("1. Created adapter");
    
    // Connect
    adapter.connect("test_device").await.unwrap();
    println!("2. Connected to device");
    
    // Take screenshot
    let _screenshot = adapter.screenshot().await.unwrap();
    println!("3. Took screenshot");
    
    // Create and start task
    let task_id = adapter.create_task(
        MaaTaskType::Screenshot,
        TaskParams::default()
    ).await.unwrap();
    adapter.start_task(task_id).await.unwrap();
    println!("4. Created and started task");
    
    // Disconnect
    adapter.disconnect().await.unwrap();
    println!("5. Disconnected from device");
    
    // Check final status
    let final_status = adapter.get_status().await.unwrap();
    println!("6. Final status: {:?}", final_status);
}

} // End of disabled_tests module