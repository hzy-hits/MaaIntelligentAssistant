// build.rs - MAA Core 构建脚本

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=maa-official/");
    
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let _maa_official = PathBuf::from(&manifest_dir).join("maa-official");
    
    // 如果启用了 with-maa-core feature，则链接到实际的 MAA Core
    #[cfg(feature = "with-maa-core")]
    {
        // 检查 MAA Core 库是否存在
        let maa_core_paths = [
            _maa_official.join("build/bin"),          // Linux/macOS 构建目录
            _maa_official.join("build/Release"),      // Windows Release 构建
            _maa_official.join("build/Debug"),        // Windows Debug 构建
            _maa_official.join("x64/Release"),        // Visual Studio 构建
            _maa_official.join("x64/Debug"),          // Visual Studio Debug 构建
        ];
        
        let mut found_lib = false;
        for path in &maa_core_paths {
            if path.exists() {
                println!("cargo:rustc-link-search=native={}", path.display());
                found_lib = true;
                break;
            }
        }
        
        if !found_lib {
            println!("cargo:warning=MAA Core library not found. Please build MAA Core first:");
            println!("cargo:warning=  cd maa-official && ./build.sh");
            println!("cargo:warning=或者使用 stub 模式: cargo build --no-default-features");
        }
        
        // 链接到 MAA Core 库
        if cfg!(target_os = "windows") {
            println!("cargo:rustc-link-lib=dylib=MaaCore");
        } else {
            println!("cargo:rustc-link-lib=dylib=MaaCore");
        }
        
        // 添加运行时库路径
        for path in &maa_core_paths {
            if path.exists() {
                println!("cargo:rustc-link-arg=-Wl,-rpath,{}", path.display());
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