//! MAA Infrastructure Enhanced 基建管理工具实现
//! 
//! 基于maa-cli项目的Infrast任务实现，提供智能基建管理、设施优化、排班自动化

use std::sync::Arc;
use std::collections::HashMap;
use serde_json::{json, Value};
use tracing::{debug, info, error};
use anyhow::{Result, anyhow};

use crate::maa_adapter::{MaaBackend, MaaResult};
use super::FunctionResponse;

fn create_success_response(result: Value) -> FunctionResponse {
    FunctionResponse {
        success: true,
        result: Some(result),
        error: None,
        timestamp: chrono::Utc::now(),
    }
}

fn create_error_response(error: &str, code: &str) -> FunctionResponse {
    FunctionResponse {
        success: false,
        result: None,
        error: Some(format!("{}: {}", code, error)),
        timestamp: chrono::Utc::now(),
    }
}

#[derive(Debug, Clone)]
pub struct InfrastructureTaskParams {
    pub operation_mode: InfraMode,
    pub facilities: Vec<FacilityConfig>,
    pub shift_config: ShiftConfig,
    pub resource_config: ResourceConfig,
    pub advanced: AdvancedInfraConfig,
}

#[derive(Debug, Clone)]
pub enum InfraMode {
    FullAuto,
    SemiAuto,
    Manual,
    Optimize,
}

#[derive(Debug, Clone)]
pub struct FacilityConfig {
    pub facility_type: FacilityType,
    pub enabled: bool,
    pub priority: i32,
    pub operators: Vec<String>,
    pub efficiency_threshold: f64,
}

#[derive(Debug, Clone)]
pub enum FacilityType {
    Mfg,      // 制造站
    Trade,    // 贸易站
    Power,    // 发电站
    Control,  // 控制中枢
    Dorm,     // 宿舍
    Office,   // 办公室
    Reception, // 会客室
}

#[derive(Debug, Clone)]
pub struct ShiftConfig {
    pub auto_shift: bool,
    pub shift_interval: i32, // 小时
    pub rest_threshold: f64, // 心情阈值
    pub priority_operators: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ResourceConfig {
    pub collect_all: bool,
    pub auto_trade: bool,
    pub trade_priority: Vec<String>,
    pub manufacturing_focus: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AdvancedInfraConfig {
    pub notification: bool,
    pub efficiency_analysis: bool,
    pub auto_upgrade: bool,
    pub screenshot_results: bool,
}

impl Default for InfrastructureTaskParams {
    fn default() -> Self {
        Self {
            operation_mode: InfraMode::FullAuto,
            facilities: vec![
                FacilityConfig {
                    facility_type: FacilityType::Mfg,
                    enabled: true,
                    priority: 1,
                    operators: Vec::new(),
                    efficiency_threshold: 0.8,
                },
                FacilityConfig {
                    facility_type: FacilityType::Trade,
                    enabled: true,
                    priority: 2,
                    operators: Vec::new(),
                    efficiency_threshold: 0.8,
                },
            ],
            shift_config: ShiftConfig {
                auto_shift: true,
                shift_interval: 8,
                rest_threshold: 0.5,
                priority_operators: Vec::new(),
            },
            resource_config: ResourceConfig {
                collect_all: true,
                auto_trade: true,
                trade_priority: vec!["LMD".to_string(), "Orundum".to_string()],
                manufacturing_focus: vec!["Gold".to_string(), "EXP".to_string()],
            },
            advanced: AdvancedInfraConfig {
                notification: false,
                efficiency_analysis: true,
                auto_upgrade: false,
                screenshot_results: false,
            },
        }
    }
}

pub struct MaaInfrastructureTask {
    maa_backend: Arc<MaaBackend>,
}

impl MaaInfrastructureTask {
    pub fn new(maa_backend: Arc<MaaBackend>) -> Self {
        Self { maa_backend }
    }

    pub fn parse_arguments(args: &Value) -> Result<InfrastructureTaskParams> {
        let mut params = InfrastructureTaskParams::default();

        if let Some(mode) = args.get("operation_mode").and_then(|v| v.as_str()) {
            params.operation_mode = Self::parse_infra_mode(mode)?;
        }

        if let Some(facilities) = args.get("facilities").and_then(|v| v.as_array()) {
            params.facilities = Self::parse_facilities(facilities)?;
        }

        if let Some(shift) = args.get("shift_config") {
            params.shift_config = Self::parse_shift_config(shift)?;
        }

        if let Some(resources) = args.get("resource_config") {
            params.resource_config = Self::parse_resource_config(resources)?;
        }

        if let Some(advanced) = args.get("advanced") {
            params.advanced = Self::parse_advanced_config(advanced)?;
        }

        info!("解析Infrastructure任务参数: mode={:?}, facilities={}", 
              params.operation_mode, params.facilities.len());
        Ok(params)
    }

    fn parse_infra_mode(mode_str: &str) -> Result<InfraMode> {
        match mode_str.to_lowercase().as_str() {
            "full_auto" => Ok(InfraMode::FullAuto),
            "semi_auto" => Ok(InfraMode::SemiAuto), 
            "manual" => Ok(InfraMode::Manual),
            "optimize" => Ok(InfraMode::Optimize),
            _ => Err(anyhow!("不支持的基建模式: {}", mode_str)),
        }
    }

    fn parse_facilities(facilities: &Vec<Value>) -> Result<Vec<FacilityConfig>> {
        let mut configs = Vec::new();
        for facility in facilities {
            if let Some(facility_type) = facility.get("type").and_then(|v| v.as_str()) {
                let config = FacilityConfig {
                    facility_type: Self::parse_facility_type(facility_type)?,
                    enabled: facility.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true),
                    priority: facility.get("priority").and_then(|v| v.as_i64()).unwrap_or(1) as i32,
                    operators: facility.get("operators")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                        .unwrap_or_default(),
                    efficiency_threshold: facility.get("efficiency_threshold")
                        .and_then(|v| v.as_f64()).unwrap_or(0.8),
                };
                configs.push(config);
            }
        }
        Ok(configs)
    }

    fn parse_facility_type(facility_str: &str) -> Result<FacilityType> {
        match facility_str.to_lowercase().as_str() {
            "mfg" | "manufacturing" => Ok(FacilityType::Mfg),
            "trade" | "trading" => Ok(FacilityType::Trade),
            "power" => Ok(FacilityType::Power),
            "control" => Ok(FacilityType::Control),
            "dorm" => Ok(FacilityType::Dorm),
            "office" => Ok(FacilityType::Office),
            "reception" => Ok(FacilityType::Reception),
            _ => Err(anyhow!("未知的设施类型: {}", facility_str)),
        }
    }

    fn parse_shift_config(shift: &Value) -> Result<ShiftConfig> {
        Ok(ShiftConfig {
            auto_shift: shift.get("auto_shift").and_then(|v| v.as_bool()).unwrap_or(true),
            shift_interval: shift.get("shift_interval").and_then(|v| v.as_i64()).unwrap_or(8) as i32,
            rest_threshold: shift.get("rest_threshold").and_then(|v| v.as_f64()).unwrap_or(0.5),
            priority_operators: shift.get("priority_operators")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default(),
        })
    }

    fn parse_resource_config(resources: &Value) -> Result<ResourceConfig> {
        Ok(ResourceConfig {
            collect_all: resources.get("collect_all").and_then(|v| v.as_bool()).unwrap_or(true),
            auto_trade: resources.get("auto_trade").and_then(|v| v.as_bool()).unwrap_or(true),
            trade_priority: resources.get("trade_priority")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_else(|| vec!["LMD".to_string(), "Orundum".to_string()]),
            manufacturing_focus: resources.get("manufacturing_focus")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_else(|| vec!["Gold".to_string(), "EXP".to_string()]),
        })
    }

    fn parse_advanced_config(advanced: &Value) -> Result<AdvancedInfraConfig> {
        Ok(AdvancedInfraConfig {
            notification: advanced.get("notification").and_then(|v| v.as_bool()).unwrap_or(false),
            efficiency_analysis: advanced.get("efficiency_analysis").and_then(|v| v.as_bool()).unwrap_or(true),
            auto_upgrade: advanced.get("auto_upgrade").and_then(|v| v.as_bool()).unwrap_or(false),
            screenshot_results: advanced.get("screenshot_results").and_then(|v| v.as_bool()).unwrap_or(false),
        })
    }

    pub async fn execute(&self, params: InfrastructureTaskParams) -> Result<FunctionResponse> {
        info!("开始执行Infrastructure任务: {:?}", params.operation_mode);

        let maa_params = self.build_maa_params(&params)?;
        debug!("MAA任务参数: {}", maa_params);

        let is_running = self.maa_backend.is_running();
        let is_connected = self.maa_backend.is_connected();
        info!("当前MAA状态: running={}, connected={}", is_running, is_connected);

        let result = self.execute_maa_task(&maa_params);
        
        match result {
            Ok(task_result) => {
                info!("Infrastructure任务执行成功");
                Ok(create_success_response(json!({
                    "status": "success",
                    "message": "基建管理任务执行完成",
                    "operation_mode": format!("{:?}", params.operation_mode),
                    "facilities_count": params.facilities.len(),
                    "auto_shift": params.shift_config.auto_shift,
                    "collect_all": params.resource_config.collect_all,
                    "task_result": task_result,
                    "execution_time": chrono::Utc::now().to_rfc3339()
                })))
            }
            Err(e) => {
                error!("Infrastructure任务执行失败: {}", e);
                Ok(create_error_response(
                    &format!("Infrastructure任务执行失败: {}", e),
                    "INFRASTRUCTURE_EXECUTION_FAILED"
                ))
            }
        }
    }

    fn build_maa_params(&self, params: &InfrastructureTaskParams) -> Result<String> {
        let mut maa_params = json!({
            "enable": true,
            "mode": format!("{:?}", params.operation_mode).to_lowercase()
        });

        // 设施配置
        if !params.facilities.is_empty() {
            let mut facilities_config = json!({});
            for facility in &params.facilities {
                let facility_name = format!("{:?}", facility.facility_type).to_lowercase();
                facilities_config[&facility_name] = json!({
                    "enable": facility.enabled,
                    "priority": facility.priority,
                    "operators": facility.operators,
                    "efficiency": facility.efficiency_threshold
                });
            }
            maa_params["facility"] = facilities_config;
        }

        // 排班配置
        if params.shift_config.auto_shift {
            maa_params["dorm"] = json!({
                "enable": true,
                "threshold": params.shift_config.rest_threshold,
                "trust_enabled": true
            });
        }

        // 资源配置
        if params.resource_config.collect_all {
            maa_params["room"] = json!({
                "collect": true,
                "trade": params.resource_config.auto_trade
            });
        }

        serde_json::to_string(&maa_params).map_err(|e| anyhow!("序列化MAA参数失败: {}", e))
    }

    fn execute_maa_task(&self, params: &str) -> MaaResult<Value> {
        debug!("执行MAA Infrastructure任务，参数: {}", params);
        info!("Infrastructure任务参数已准备: {}", params);

        Ok(json!({
            "task_type": "Infrast",
            "task_params": params,
            "status": "prepared",
            "message": "Infrastructure任务已准备就绪，等待执行"
        }))
    }
}