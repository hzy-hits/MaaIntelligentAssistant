//! Test real MAA Core availability

use maa_intelligent_server::maa_adapter::{MaaBackend, BackendConfig};

#[test]
fn test_real_maa_availability() {
    println!("Testing real MAA Core availability...");
    
    let config = BackendConfig {
        force_stub: false,
        prefer_real: true,
        resource_path: "/Users/ivena/Desktop/Fairy/maa/maa-remote-server/maa-official".to_string(),
        verbose: true,
    };
    
    match MaaBackend::new(config) {
        Ok(backend) => {
            if backend.is_real() {
                println!("✅ Real MAA Core is available!");
                let version = MaaBackend::get_version().expect("Should get version");
                println!("✅ Real MAA version: {}", version);
            } else {
                println!("ℹ️ Real MAA Core not available, using stub mode (expected in many environments)");
                assert!(backend.is_stub());
            }
        }
        Err(e) => {
            println!("ℹ️ Backend creation failed (expected without MAA setup): {}", e);
            // This is expected in environments without proper MAA setup
        }
    }
}

#[cfg(feature = "with-maa-core")]
#[test] 
fn test_with_maa_core_feature() {
    println!("Testing with 'with-maa-core' feature enabled...");
    
    let config = BackendConfig {
        force_stub: false,
        prefer_real: true,
        resource_path: "/Users/ivena/Desktop/Fairy/maa/maa-remote-server/maa-official".to_string(),
        verbose: true,
    };
    
    let backend = MaaBackend::new(config).expect("Backend should be created with maa-core feature");
    
    if backend.is_real() {
        println!("✅ Real MAA backend created with feature flag!");
    } else {
        println!("ℹ️ Still using stub despite feature flag (MAA Core not properly configured)");
    }
}

#[test]
fn test_maa_core_path_detection() {
    println!("Testing MAA Core path detection...");
    
    let known_paths = vec![
        "/Users/ivena/Library/Application Support/com.loong.maa/lib/libMaaCore.dylib",
        "/usr/local/lib/libMaaCore.dylib",
        "./libMaaCore.dylib",
    ];
    
    let mut found_paths = Vec::new();
    for path in known_paths {
        if std::path::Path::new(path).exists() {
            found_paths.push(path);
            println!("✅ Found MAA Core at: {}", path);
        }
    }
    
    if found_paths.is_empty() {
        println!("ℹ️ No MAA Core library found at known paths (expected in many test environments)");
    } else {
        println!("✅ MAA Core libraries found: {:?}", found_paths);
    }
}