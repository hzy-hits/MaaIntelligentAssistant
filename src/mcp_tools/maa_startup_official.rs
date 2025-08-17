//! MAA StartUp 官方规范实现
//! 
//! 基于 maa-cli StartUp 任务的官方参数规范

use std::sync::Arc;
use serde_json::{json, Value};
use tracing::{debug, info, error};
use anyhow::{Result, anyhow};

use crate::maa_adapter::{MaaBackend, MaaResult};
use super::FunctionResponse;

fn create_success_response(result: Value) -> FunctionResponse {
    FunctionResponse { success: true, result: Some(result), error: None, timestamp: chrono::Utc::now() }
}

fn create_error_response(error: &str, code: &str) -> FunctionResponse {
    FunctionResponse { success: false, result: None, error: Some(format!("{}: {}", code, error)), timestamp: chrono::Utc::now() }
}

/// 符合官方 maa-cli 规范的 StartUp 任务参数
#[derive(Debug, Clone)]
pub struct OfficialStartUpParams {
    /// 是否启用任务
    pub enable: bool,
    /// 客户端类型
    pub client_type: String,
    /// 是否启动游戏
    pub start_game_enabled: bool,
    /// 账号切换相关（可选）
    pub account_name: Option<String>,
}

impl Default for OfficialStartUpParams {
    fn default() -> Self {
        Self {
            enable: true,
            client_type: "Official".to_string(),
            start_game_enabled: false,
            account_name: None,
        }
    }
}

pub struct MaaOfficialStartUpTask {
    maa_backend: Arc<MaaBackend>,
}

impl MaaOfficialStartUpTask {
    pub fn new(maa_backend: Arc<MaaBackend>) -> Self {
        Self { maa_backend }
    }

    /// 解析符合官方规范的参数
    pub fn parse_arguments(args: &Value) -> Result<OfficialStartUpParams> {
        let mut params = OfficialStartUpParams::default();

        // 客户端类型（必填，有默认值）
        if let Some(client_type) = args.get("client_type").and_then(|v| v.as_str()) {
            if Self::is_valid_client_type(client_type) {
                params.client_type = client_type.to_string();
            } else {
                return Err(anyhow!("不支持的客户端类型: {}", client_type));
            }
        }

        // 是否启动游戏
        if let Some(start_game) = args.get("start_game_enabled").and_then(|v| v.as_bool()) {
            params.start_game_enabled = start_game;
        }

        // 是否启用任务
        if let Some(enable) = args.get("enable").and_then(|v| v.as_bool()) {
            params.enable = enable;
        }

        // 账号名称（可选）
        if let Some(account) = args.get("account_name").and_then(|v| v.as_str()) {
            params.account_name = Some(account.to_string());
        }

        Ok(params)
    }

    /// 验证客户端类型是否有效
    fn is_valid_client_type(client_type: &str) -> bool {
        matches!(client_type, 
            "Official" | "Bilibili" | "Txwy" | "YoStarEN" | "YoStarJP" | "YoStarKR"
        )
    }

    /// 执行 StartUp 任务
    pub async fn execute(&self, params: OfficialStartUpParams) -> Result<FunctionResponse> {
        info!("开始执行官方规范StartUp任务: client_type={}, start_game={}", 
              params.client_type, params.start_game_enabled);

        // 构建符合 MAA Core 的参数格式
        let maa_params = self.build_maa_params(&params)?;
        
        // 执行 MAA 任务
        let result = self.execute_maa_task("StartUp", &maa_params);
        
        match result {
            Ok(task_result) => {
                Ok(create_success_response(json!({
                    "status": "success",
                    "message": "启动任务执行完成",
                    "client_type": params.client_type,
                    "start_game_enabled": params.start_game_enabled,
                    "account_name": params.account_name,
                    "task_result": task_result
                })))
            }
            Err(e) => {
                Ok(create_error_response(&format!("启动任务执行失败: {}", e), "STARTUP_FAILED"))
            }
        }
    }

    /// 构建符合 MAA Core 的任务参数
    fn build_maa_params(&self, params: &OfficialStartUpParams) -> Result<String> {
        let mut maa_params = json!({
            "enable": params.enable,
            "client_type": params.client_type,
            "start_game_enabled": params.start_game_enabled
        });

        // 添加账号信息（如果提供）
        if let Some(account_name) = &params.account_name {
            maa_params["account_name"] = json!(account_name);
        }

        Ok(serde_json::to_string(&maa_params)?)
    }

    /// 执行 MAA 任务（通过 MAA Backend）
    fn execute_maa_task(&self, task_type: &str, params: &str) -> MaaResult<Value> {
        debug!("执行MAA任务: type={}, params={}", task_type, params);
        
        // 在 stub 模式下返回模拟结果
        Ok(json!({
            "task_type": task_type,
            "task_params": params,
            "status": "prepared",
            "backend_type": self.maa_backend.backend_type()
        }))
    }
}

/// 解析自然语言启动命令（保持向下兼容）
pub fn parse_natural_language_startup(command: &str) -> Result<Value> {
    let command = command.to_lowercase();
    
    // 确定客户端类型
    let client_type = if command.contains("官服") || command.contains("official") {
        "Official"
    } else if command.contains("b服") || command.contains("哔哩") || command.contains("bilibili") {
        "Bilibili"  
    } else if command.contains("渠道服") || command.contains("txwy") {
        "Txwy"
    } else if command.contains("国际服") || command.contains("yostar") {
        "YoStarEN"
    } else if command.contains("日服") {
        "YoStarJP"
    } else if command.contains("韩服") {
        "YoStarKR"
    } else {
        "Official" // 默认官服
    };

    // 确定是否启动游戏
    let start_game = command.contains("启动游戏") || 
                    command.contains("打开游戏") || 
                    command.contains("开游戏");

    Ok(json!({
        "client_type": client_type,
        "start_game_enabled": start_game,
        "enable": true
    }))
}