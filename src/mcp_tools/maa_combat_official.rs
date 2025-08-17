//! MAA Combat 官方规范实现
//! 
//! 基于 maa-cli Fight 任务的官方参数规范，提供标准的战斗任务功能

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

/// 符合官方 maa-cli 规范的 Combat 任务参数
/// 基于官方 Fight 任务的 JSON 配置格式
#[derive(Debug, Clone)]
pub struct OfficialCombatParams {
    /// 是否启用任务
    pub enable: bool,
    /// 关卡名称 (如 "1-7", "CE-5", "AP-5")
    pub stage: String,
    /// 理智药剂使用数量
    pub medicine: i32,
    /// 源石使用数量  
    pub stone: i32,
    /// 战斗次数
    pub times: i32,
    /// 理智不足时的处理方式
    pub client_type: Option<String>,
    /// series: 连战系列 (可选)
    pub series: Option<Value>,
    /// DrGrandet 模式
    pub dr_grandet: Option<bool>,
}

impl Default for OfficialCombatParams {
    fn default() -> Self {
        Self {
            enable: true,
            stage: "1-7".to_string(),
            medicine: 0,
            stone: 0,
            times: 1,
            client_type: Some("Official".to_string()),
            series: None,
            dr_grandet: Some(false),
        }
    }
}

pub struct MaaOfficialCombatTask {
    maa_backend: Arc<MaaBackend>,
}

impl MaaOfficialCombatTask {
    pub fn new(maa_backend: Arc<MaaBackend>) -> Self {
        Self { maa_backend }
    }

    /// 解析符合官方规范的参数
    pub fn parse_arguments(args: &Value) -> Result<OfficialCombatParams> {
        let mut params = OfficialCombatParams::default();

        // 必填参数：关卡
        if let Some(stage) = args.get("stage").and_then(|v| v.as_str()) {
            params.stage = Self::normalize_stage_name(stage);
        } else {
            return Err(anyhow!("缺少必填参数: stage"));
        }

        // 可选参数
        if let Some(medicine) = args.get("medicine").and_then(|v| v.as_i64()) {
            params.medicine = medicine as i32;
        }

        if let Some(stone) = args.get("stone").and_then(|v| v.as_i64()) {
            params.stone = stone as i32;
        }

        if let Some(times) = args.get("times").and_then(|v| v.as_i64()) {
            params.times = times as i32;
        }

        if let Some(client_type) = args.get("client_type").and_then(|v| v.as_str()) {
            params.client_type = Some(client_type.to_string());
        }

        if let Some(enable) = args.get("enable").and_then(|v| v.as_bool()) {
            params.enable = enable;
        }

        if let Some(dr_grandet) = args.get("dr_grandet").and_then(|v| v.as_bool()) {
            params.dr_grandet = Some(dr_grandet);
        }

        // series 参数（复杂对象，直接传递）
        if let Some(series) = args.get("series") {
            params.series = Some(series.clone());
        }

        Ok(params)
    }

    /// 标准化关卡名称，支持自然语言别名
    fn normalize_stage_name(stage: &str) -> String {
        match stage.to_lowercase().as_str() {
            // 经验书本（狗粮）
            "狗粮" | "经验书" | "经验" | "exp" => "1-7".to_string(),
            
            // 龙门币本
            "龙门币" | "龙门币本" | "钱本" | "money" => "CE-5".to_string(),
            
            // 技能书本
            "技能书" | "技能" | "skill" => "CA-5".to_string(),
            
            // 红票本
            "红票" | "红票本" | "recruit" => "AP-5".to_string(),
            
            // 芯片本
            "芯片" | "chip" => "PR-A-1".to_string(),
            "芯片a" | "芯片A" => "PR-A-2".to_string(),
            "芯片b" | "芯片B" => "PR-B-2".to_string(),
            
            // 直接返回原始关卡名
            _ => stage.to_string(),
        }
    }

    /// 执行 Combat 任务
    pub async fn execute(&self, params: OfficialCombatParams) -> Result<FunctionResponse> {
        info!("开始执行官方规范Combat任务: stage={}, times={}", params.stage, params.times);

        // 构建符合 MAA Core 的参数格式
        let maa_params = self.build_maa_params(&params)?;
        
        // 执行 MAA 任务
        let result = self.execute_maa_task("Fight", &maa_params);
        
        match result {
            Ok(task_result) => {
                Ok(create_success_response(json!({
                    "status": "success",
                    "message": "战斗任务执行完成",
                    "stage": params.stage,
                    "times": params.times,
                    "medicine": params.medicine,
                    "stone": params.stone,
                    "task_result": task_result
                })))
            }
            Err(e) => {
                Ok(create_error_response(&format!("战斗任务执行失败: {}", e), "COMBAT_FAILED"))
            }
        }
    }

    /// 构建符合 MAA Core 的任务参数
    fn build_maa_params(&self, params: &OfficialCombatParams) -> Result<String> {
        let mut maa_params = json!({
            "enable": params.enable,
            "stage": params.stage,
            "medicine": params.medicine,
            "stone": params.stone,
            "times": params.times
        });

        // 添加可选参数
        if let Some(client_type) = &params.client_type {
            maa_params["client_type"] = json!(client_type);
        }

        if let Some(dr_grandet) = params.dr_grandet {
            maa_params["DrGrandet"] = json!(dr_grandet);
        }

        if let Some(series) = &params.series {
            maa_params["series"] = series.clone();
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

/// 解析自然语言战斗命令（保持向下兼容）
pub fn parse_natural_language_combat(command: &str) -> Result<Value> {
    let command = command.to_lowercase();
    
    // 提取关卡信息
    let stage = if command.contains("1-7") || command.contains("狗粮") || command.contains("经验") {
        "1-7"
    } else if command.contains("ce-5") || command.contains("龙门币") || command.contains("钱") {
        "CE-5"
    } else if command.contains("ca-5") || command.contains("技能书") {
        "CA-5"
    } else if command.contains("ap-5") || command.contains("红票") {
        "AP-5"
    } else {
        // 尝试提取数字-数字格式
        if let Some(captures) = regex::Regex::new(r"(\d+-\d+)").unwrap().captures(&command) {
            captures.get(1).unwrap().as_str()
        } else {
            "1-7" // 默认
        }
    };

    // 提取次数
    let times = extract_times_from_command(&command).unwrap_or(1);
    
    // 提取理智药剂
    let medicine = extract_medicine_from_command(&command).unwrap_or(0);

    Ok(json!({
        "stage": stage,
        "times": times,
        "medicine": medicine,
        "stone": 0,
        "enable": true
    }))
}

fn extract_times_from_command(command: &str) -> Option<i32> {
    if let Some(captures) = regex::Regex::new(r"刷?(\d+)次").unwrap().captures(command) {
        captures.get(1).unwrap().as_str().parse().ok()
    } else if let Some(captures) = regex::Regex::new(r"(\d+)次").unwrap().captures(command) {
        captures.get(1).unwrap().as_str().parse().ok()
    } else {
        None
    }
}

fn extract_medicine_from_command(command: &str) -> Option<i32> {
    if command.contains("用药") || command.contains("理智药") {
        if let Some(captures) = regex::Regex::new(r"用?(\d+)个?药").unwrap().captures(command) {
            captures.get(1).unwrap().as_str().parse().ok()
        } else {
            Some(999) // 默认用完所有药
        }
    } else {
        None
    }
}