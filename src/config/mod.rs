use anyhow::{Result, Context};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub device: DeviceConfig,
    pub maa: MaaConfig,
    pub client: ClientConfig,
    pub stages: StageConfig,
    pub logging: LogConfig,
    pub ai: AiConfig,
    pub webui: WebUIConfig,
    pub performance: PerformanceConfig,
    pub messages: MessageConfig,
    pub status_codes: StatusCodeConfig,
    pub env_keys: EnvKeyConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub default_port: String,
    pub default_host: String,
    pub health_check_path: String,
    pub tools_path: String,
    pub call_path: String,
    pub status_path: String,
}

#[derive(Debug, Deserialize)]
pub struct DeviceConfig {
    pub playcover_address: String,
    pub android_emulator_address: String,
    pub touch_mode_playcover: String,
    pub connection_timeout_ms: u64,
    pub retry_attempts: u32,
}

#[derive(Debug, Deserialize)]
pub struct MaaConfig {
    pub default_app_path: String,
    pub default_core_lib_path: String,
    pub default_resource_path: String,
    pub default_adb_path: String,
    pub fallback_lib_paths: Vec<String>,
    pub fallback_resource_paths: Vec<String>,
    pub stub_version: String,
    pub backend_mode_real: String,
    pub backend_mode_stub: String,
}

#[derive(Debug, Deserialize)]
pub struct ClientConfig {
    pub default_client: String,
    pub supported_clients: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct StageConfig {
    pub common_stages: Vec<String>,
    pub material_stages: Vec<String>,
    pub example_stages: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct LogConfig {
    pub default_level: String,
    pub levels: Vec<String>,
    pub debug_mode_level: String,
    pub production_level: String,
}

#[derive(Debug, Deserialize)]
pub struct AiConfig {
    pub default_provider: String,
    pub supported_providers: Vec<String>,
    pub default_qwen_model: String,
    pub default_openai_model: String,
    pub qwen_base_url: String,
    pub openai_base_url: String,
}

#[derive(Debug, Deserialize)]
pub struct WebUIConfig {
    pub default_port: String,
    pub default_name: String,
    pub default_secret_key: String,
}

#[derive(Debug, Deserialize)]
pub struct PerformanceConfig {
    pub task_queue_buffer_size: usize,
    pub response_timeout_ms: u64,
    pub worker_heartbeat_ms: u64,
    pub connection_pool_size: usize,
    pub max_concurrent_requests: usize,
}

#[derive(Debug, Deserialize)]
pub struct MessageConfig {
    pub success: String,
    pub failure: String,
    pub timeout: String,
    pub connection_error: String,
    pub invalid_request: String,
    pub system_error: String,
}

#[derive(Debug, Deserialize)]
pub struct StatusCodeConfig {
    pub success: i32,
    pub failure: i32,
    pub timeout: i32,
    pub connection_error: i32,
    pub invalid_request: i32,
}

#[derive(Debug, Deserialize)]
pub struct EnvKeyConfig {
    pub server_port: String,
    pub device_address: String,
    pub core_lib: String,
    pub resource_path: String,
    pub adb_path: String,
    pub app_path: String,
    pub core_dir: String,
    pub dyld_library_path: String,
    pub backend_mode: String,
    pub verbose: String,
    pub force_stub: String,
    pub ai_provider: String,
    pub ai_api_key: String,
    pub ai_base_url: String,
    pub ai_model: String,
    pub webui_port: String,
    pub webui_name: String,
    pub webui_secret: String,
    pub debug_mode: String,
    pub log_level: String,
    pub http_proxy: String,
    pub https_proxy: String,
}

impl AppConfig {
    pub fn load() -> Result<AppConfig> {
        let config_path = Self::find_config_file()?;
        let content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {:?}", config_path))?;
        
        let config: AppConfig = toml::from_str(&content)
            .with_context(|| "Failed to parse config file")?;
        
        Ok(config)
    }
    
    fn find_config_file() -> Result<std::path::PathBuf> {
        let possible_paths = [
            "config/app.toml",
            "./config/app.toml", 
            "../config/app.toml",
        ];
        
        for path in &possible_paths {
            if Path::new(path).exists() {
                return Ok(Path::new(path).to_path_buf());
            }
        }
        
        Err(anyhow::anyhow!("Config file not found. Searched in: {:?}", possible_paths))
    }
}

impl ServerConfig {
    pub fn bind_address(&self, port: Option<&str>) -> String {
        format!("{}:{}", self.default_host, port.unwrap_or(&self.default_port))
    }
}

impl DeviceConfig {
    pub fn is_playcover_address(&self, address: &str) -> bool {
        address == self.playcover_address || 
        address == "localhost:1717" ||
        address.contains(":1717")
    }
}

impl ClientConfig {
    pub fn is_valid_client(&self, client: &str) -> bool {
        self.supported_clients.contains(&client.to_string())
    }
    
    pub fn validate_or_default<'a>(&'a self, client: Option<&'a str>) -> &'a str {
        client
            .filter(|c| self.is_valid_client(c))
            .unwrap_or(&self.default_client)
    }
}

impl LogConfig {
    pub fn is_valid_level(&self, level: &str) -> bool {
        self.levels.contains(&level.to_string())
    }
    
    pub fn get_level_or_default<'a>(&'a self, level: Option<&'a str>) -> &'a str {
        level
            .filter(|l| self.is_valid_level(l))
            .unwrap_or(&self.default_level)
    }
}

impl AiConfig {
    pub fn is_supported_provider(&self, provider: &str) -> bool {
        self.supported_providers.contains(&provider.to_string())
    }
    
    pub fn get_default_model(&self, provider: &str) -> Option<&str> {
        match provider {
            "qwen" => Some(&self.default_qwen_model),
            "openai" => Some(&self.default_openai_model),
            _ => None,
        }
    }
    
    pub fn get_default_base_url(&self, provider: &str) -> Option<&str> {
        match provider {
            "qwen" => Some(&self.qwen_base_url),
            "openai" => Some(&self.openai_base_url),
            _ => None,
        }
    }
}

use once_cell::sync::Lazy;

pub static CONFIG: Lazy<AppConfig> = Lazy::new(|| {
    AppConfig::load().unwrap_or_else(|e| {
        eprintln!("Warning: Failed to load config file, using defaults: {}", e);
        create_default_config()
    })
});

fn create_default_config() -> AppConfig {
    // 返回默认配置，避免程序崩溃
    AppConfig {
        server: ServerConfig {
            default_port: "8080".to_string(),
            default_host: "0.0.0.0".to_string(),
            health_check_path: "/health".to_string(),
            tools_path: "/tools".to_string(),
            call_path: "/call".to_string(),
            status_path: "/status".to_string(),
        },
        device: DeviceConfig {
            playcover_address: "127.0.0.1:1717".to_string(),
            android_emulator_address: "127.0.0.1:5555".to_string(),
            touch_mode_playcover: "MacPlayTools".to_string(),
            connection_timeout_ms: 10000,
            retry_attempts: 3,
        },
        maa: MaaConfig {
            default_app_path: "/Applications/MAA.app".to_string(),
            default_core_lib_path: "/Applications/MAA.app/Contents/Frameworks/libMaaCore.dylib".to_string(),
            default_resource_path: "/Applications/MAA.app/Contents/Resources".to_string(),
            default_adb_path: "/Applications/MAA.app/Contents/MacOS/adb".to_string(),
            fallback_lib_paths: vec![
                "/Applications/MAA.app/Contents/Frameworks/libMaaCore.dylib".to_string(),
            ],
            fallback_resource_paths: vec![
                "/Applications/MAA.app/Contents/Resources".to_string(),
            ],
            stub_version: "stub".to_string(),
            backend_mode_real: "real".to_string(),
            backend_mode_stub: "stub".to_string(),
        },
        client: ClientConfig {
            default_client: "Official".to_string(),
            supported_clients: vec!["Official".to_string(), "Bilibili".to_string()],
        },
        stages: StageConfig {
            common_stages: vec!["1-7".to_string()],
            material_stages: vec!["CE-5".to_string()],
            example_stages: vec!["1-7".to_string()],
        },
        logging: LogConfig {
            default_level: "info".to_string(),
            levels: vec!["error".to_string(), "warn".to_string(), "info".to_string()],
            debug_mode_level: "debug".to_string(),
            production_level: "warn".to_string(),
        },
        ai: AiConfig {
            default_provider: "qwen".to_string(),
            supported_providers: vec!["qwen".to_string()],
            default_qwen_model: "qwen-plus".to_string(),
            default_openai_model: "gpt-4".to_string(),
            qwen_base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            openai_base_url: "https://api.openai.com/v1".to_string(),
        },
        webui: WebUIConfig {
            default_port: "3000".to_string(),
            default_name: "MAA智能助手".to_string(),
            default_secret_key: "change-this-in-production".to_string(),
        },
        performance: PerformanceConfig {
            task_queue_buffer_size: 1000,
            response_timeout_ms: 30000,
            worker_heartbeat_ms: 1000,
            connection_pool_size: 10,
            max_concurrent_requests: 100,
        },
        messages: MessageConfig {
            success: "Operation completed successfully".to_string(),
            failure: "Operation failed".to_string(),
            timeout: "Operation timed out".to_string(),
            connection_error: "Connection error".to_string(),
            invalid_request: "Invalid request".to_string(),
            system_error: "System error".to_string(),
        },
        status_codes: StatusCodeConfig {
            success: 0,
            failure: -1,
            timeout: -2,
            connection_error: -3,
            invalid_request: -4,
        },
        env_keys: EnvKeyConfig {
            server_port: "MAA_PORT".to_string(),
            device_address: "MAA_DEVICE_ADDRESS".to_string(),
            core_lib: "MAA_CORE_LIB".to_string(),
            resource_path: "MAA_RESOURCE_PATH".to_string(),
            adb_path: "MAA_ADB_PATH".to_string(),
            app_path: "MAA_APP_PATH".to_string(),
            core_dir: "MAA_CORE_DIR".to_string(),
            dyld_library_path: "DYLD_LIBRARY_PATH".to_string(),
            backend_mode: "MAA_BACKEND_MODE".to_string(),
            verbose: "MAA_VERBOSE".to_string(),
            force_stub: "MAA_FORCE_STUB".to_string(),
            ai_provider: "AI_PROVIDER".to_string(),
            ai_api_key: "AI_API_KEY".to_string(),
            ai_base_url: "AI_BASE_URL".to_string(),
            ai_model: "AI_MODEL".to_string(),
            webui_port: "WEBUI_PORT".to_string(),
            webui_name: "WEBUI_NAME".to_string(),
            webui_secret: "WEBUI_SECRET_KEY".to_string(),
            debug_mode: "DEBUG_MODE".to_string(),
            log_level: "LOG_LEVEL".to_string(),
            http_proxy: "HTTP_PROXY".to_string(),
            https_proxy: "HTTPS_PROXY".to_string(),
        },
    }
}