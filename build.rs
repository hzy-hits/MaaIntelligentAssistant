// build.rs - MAA Core 构建脚本

use std::env;
use std::path::PathBuf;
use std::fs;

fn load_env_file() {
    // 尝试读取 .env 文件并设置环境变量
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let env_file = PathBuf::from(&manifest_dir).join(".env");
    
    if let Ok(contents) = fs::read_to_string(&env_file) {
        for line in contents.lines() {
            let line = line.trim();
            // 跳过注释和空行
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            // 解析 KEY=VALUE 格式
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                // 只有当环境变量不存在时才设置（优先使用系统环境变量）
                if env::var(key).is_err() {
                    env::set_var(key, value);
                    println!("cargo:warning=Loaded from .env: {}={}", key, value);
                }
            }
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=maa-official/");
    println!("cargo:rerun-if-changed=.env");
    
    // 首先加载 .env 文件
    load_env_file();
    
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let _maa_official = PathBuf::from(&manifest_dir).join("maa-official");
    
    // 如果启用了 with-maa-core feature，则链接到实际的 MAA Core
    #[cfg(feature = "with-maa-core")]
    {
        let mut found_lib = false;
        
        // 首先尝试使用环境变量配置的路径 (支持使用系统安装的MAA.app)
        if let Ok(maa_core_dir) = env::var("MAA_CORE_DIR") {
            let core_dir = PathBuf::from(&maa_core_dir);
            let lib_file = if cfg!(target_os = "macos") {
                core_dir.join("libMaaCore.dylib")
            } else if cfg!(target_os = "linux") {
                core_dir.join("libMaaCore.so")
            } else {
                core_dir.join("MaaCore.dll")
            };
            
            if lib_file.exists() {
                println!("cargo:rustc-link-search=native={}", core_dir.display());
                println!("cargo:rustc-link-arg=-Wl,-rpath,{}", core_dir.display());
                println!("cargo:warning=Using MAA Core from environment: {}", lib_file.display());
                found_lib = true;
            } else {
                println!("cargo:warning=MAA_CORE_DIR set but library not found: {}", lib_file.display());
            }
        }
        
        // 如果环境变量未设置或无效，回退到原有的构建目录搜索
        if !found_lib {
            let maa_core_paths = [
                _maa_official.join("build/bin"),          // Linux/macOS 构建目录
                _maa_official.join("build/Release"),      // Windows Release 构建
                _maa_official.join("build/Debug"),        // Windows Debug 构建
                _maa_official.join("x64/Release"),        // Visual Studio 构建
                _maa_official.join("x64/Debug"),          // Visual Studio Debug 构建
            ];
            
            for path in &maa_core_paths {
                if path.exists() {
                    println!("cargo:rustc-link-search=native={}", path.display());
                    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", path.display());
                    println!("cargo:warning=Using MAA Core from build directory: {}", path.display());
                    found_lib = true;
                    break;
                }
            }
        }
        
        if !found_lib {
            println!("cargo:warning=MAA Core library not found. Please either:");
            println!("cargo:warning=  1. Set MAA_CORE_DIR environment variable to point to your MAA installation");
            println!("cargo:warning=  2. Build MAA Core: cd maa-official && ./build.sh");
            println!("cargo:warning=  3. Use stub mode: cargo build --no-default-features");
        } else {
            // 只有找到库文件时才尝试链接
            if cfg!(target_os = "windows") {
                println!("cargo:rustc-link-lib=dylib=MaaCore");
            } else {
                println!("cargo:rustc-link-lib=dylib=MaaCore");
            }
        }
    }
    
    // 如果没有启用 with-maa-core，使用 stub 模式（默认）
    #[cfg(not(feature = "with-maa-core"))]
    {
        println!("cargo:warning=Building in stub mode. MAA Core functions will be mocked.");
        println!("cargo:warning=To use real MAA Core, build with: cargo build --features with-maa-core");
    }
}