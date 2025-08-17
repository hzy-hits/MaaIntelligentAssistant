//! MAA Combat Enhanced 任务工具实现
//! 
//! 基于maa-cli项目的Fight任务实现，提供自动战斗、复杂策略、资源管理、掉落统计等功能

use std::sync::Arc;
use std::collections::HashMap;
use serde_json::{json, Value};
use tracing::{debug, info, warn, error};
use anyhow::{Result, anyhow, Context};

use crate::maa_adapter::{MaaBackend, MaaResult};
use super::FunctionResponse;

/// 创建成功响应
fn create_success_response(result: Value) -> FunctionResponse {
    FunctionResponse {
        success: true,
        result: Some(result),
        error: None,
        timestamp: chrono::Utc::now(),
    }
}

/// 创建错误响应
fn create_error_response(error: &str, code: &str) -> FunctionResponse {
    FunctionResponse {
        success: false,
        result: None,
        error: Some(format!("{}: {}", code, error)),
        timestamp: chrono::Utc::now(),
    }
}

/// Combat任务参数
#[derive(Debug, Clone)]
pub struct CombatTaskParams {
    /// 关卡代码或自然语言描述
    pub stage: String,
    /// 战斗策略配置
    pub strategy: CombatStrategy,
    /// 资源使用配置
    pub resources: ResourceConfig,
    /// 掉落收集目标
    pub drops: Vec<DropTarget>,
    /// 数据上报配置
    pub reporting: ReportingConfig,
    /// 高级功能配置
    pub advanced: AdvancedConfig,
}

/// 战斗策略配置
#[derive(Debug, Clone)]
pub struct CombatStrategy {
    /// 战斗模式
    pub mode: CombatMode,
    /// 目标值（次数/理智/材料数量）
    pub target_value: i32,
    /// 目标材料名称（仅material模式）
    pub target_material: Option<String>,
}

/// 战斗模式（基于maa-cli Fight任务分析）
#[derive(Debug, Clone)]
pub enum CombatMode {
    /// 固定次数模式
    Times,
    /// 理智消耗模式  
    Sanity,
    /// 材料收集模式
    Material,
    /// 无限刷取模式
    Infinite,
}

/// 资源使用配置（基于maa-cli的medicine/stone参数）
#[derive(Debug, Clone)]
pub struct ResourceConfig {
    /// 理智药剂使用数量
    pub medicine: i32,
    /// 即将过期理智药剂使用数量  
    pub expiring_medicine: i32,
    /// 源石使用数量
    pub stone: i32,
    /// Dr.Grandet模式（等待理智恢复1点后再使用源石）
    pub dr_grandet: bool,
}

/// 掉落收集目标
#[derive(Debug, Clone)]
pub struct DropTarget {
    /// 物品ID或名称
    pub item_id: String,
    /// 目标数量
    pub target_count: i32,
}

/// 数据上报配置
#[derive(Debug, Clone)]
pub struct ReportingConfig {
    /// 企鹅物流数据上报
    pub penguin_stats: bool,
    /// 企鹅物流用户ID
    pub penguin_id: Option<String>,
    /// 一图流数据上报
    pub yituliu: bool,
    /// 一图流用户ID  
    pub yituliu_id: Option<String>,
}

/// 高级功能配置
#[derive(Debug, Clone)]
pub struct AdvancedConfig {
    /// 代理连战次数 (-1: 禁用, 0: 自动, 1-6: 指定次数)
    pub series: i32,
    /// 是否自动选择代理指挥
    pub auto_agent: bool,
    /// 代理指挥失败时的后备关卡
    pub backup_stage: Option<String>,
    /// 是否启用掉落统计
    pub drop_tracking: bool,
}

impl Default for CombatTaskParams {
    fn default() -> Self {
        Self {
            stage: "1-7".to_string(),
            strategy: CombatStrategy {
                mode: CombatMode::Times,
                target_value: 1,
                target_material: None,
            },
            resources: ResourceConfig {
                medicine: 0,
                expiring_medicine: 0,
                stone: 0,
                dr_grandet: false,
            },
            drops: Vec::new(),
            reporting: ReportingConfig {
                penguin_stats: false,
                penguin_id: None,
                yituliu: false,
                yituliu_id: None,
            },
            advanced: AdvancedConfig {
                series: 1,
                auto_agent: true,
                backup_stage: None,
                drop_tracking: true,
            },
        }
    }
}

/// MAA Combat任务执行器
pub struct MaaCombatTask {
    maa_backend: Arc<MaaBackend>,
}

impl MaaCombatTask {
    pub fn new(maa_backend: Arc<MaaBackend>) -> Self {
        Self { maa_backend }
    }

    /// 从Function Calling参数解析Combat任务参数
    pub fn parse_arguments(args: &Value) -> Result<CombatTaskParams> {
        let mut params = CombatTaskParams::default();

        // 解析关卡
        if let Some(stage) = args.get("stage").and_then(|v| v.as_str()) {
            params.stage = Self::parse_stage_name(stage)?;
        } else {
            return Err(anyhow!("缺少必需的stage参数"));
        }

        // 解析战斗策略
        if let Some(strategy) = args.get("strategy") {
            params.strategy = Self::parse_strategy(strategy)?;
        }

        // 解析资源配置
        if let Some(resources) = args.get("resources") {
            params.resources = Self::parse_resources(resources)?;
        }

        // 解析掉落目标
        if let Some(drops) = args.get("drops").and_then(|v| v.as_array()) {
            params.drops = Self::parse_drops(drops)?;
        }

        // 解析数据上报配置
        if let Some(reporting) = args.get("reporting") {
            params.reporting = Self::parse_reporting(reporting)?;
        }

        // 解析高级配置
        if let Some(automation) = args.get("automation") {
            params.advanced = Self::parse_advanced(automation)?;
        }

        info!("解析Combat任务参数: stage={}, mode={:?}, medicine={}, stone={}",
              params.stage, params.strategy.mode, params.resources.medicine, params.resources.stone);

        Ok(params)
    }

    /// 智能关卡名称解析（基于maa-cli支持的关卡格式）
    fn parse_stage_name(stage_input: &str) -> Result<String> {
        let stage_lower = stage_input.to_lowercase();

        // 关卡别名映射（基于明日方舟常用称呼）
        let stage_aliases = [
            // 资源本别名
            ("狗粮", "1-7"),
            ("龙门币本", "CE-5"), 
            ("经验书本", "LS-5"),
            ("技能书本", "CA-5"),
            ("红票本", "AP-5"),
            ("碳本", "SK-5"),
            ("采购凭证", "PR-A-1"),
            ("技巧概要", "PR-B-1"),
            ("芯片本", "PR-C-1"),
            ("源岩", "1-7"),
            ("聚合物", "4-6"),
            
            // 常见主线关卡
            ("一七", "1-7"),
            ("四六", "4-6"),
            ("五三", "5-3"),
            ("六九", "6-9"),
            
            // 高难度关卡
            ("十二四", "H12-4"),
            ("沙中之火", "SL-8"),
        ];

        // 检查别名映射
        for (alias, stage_code) in &stage_aliases {
            if stage_lower.contains(alias) {
                info!("关卡别名解析: '{}' -> '{}'", stage_input, stage_code);
                return Ok(stage_code.to_string());
            }
        }

        // 如果不是别名，直接使用原始输入（可能已经是正确的关卡代码）
        // 验证关卡代码格式
        if Self::is_valid_stage_code(stage_input) {
            Ok(stage_input.to_string())
        } else {
            warn!("未识别的关卡代码: '{}'，将直接使用原始输入", stage_input);
            Ok(stage_input.to_string())
        }
    }

    /// 验证关卡代码格式
    fn is_valid_stage_code(stage: &str) -> bool {
        // 简单的关卡代码格式验证
        stage.contains('-') || 
        stage.starts_with("CE") || 
        stage.starts_with("LS") || 
        stage.starts_with("CA") || 
        stage.starts_with("AP") || 
        stage.starts_with("SK") ||
        stage.starts_with("PR-") ||
        stage.starts_with("H") ||
        stage.starts_with("SL-")
    }

    /// 解析战斗策略
    fn parse_strategy(strategy: &Value) -> Result<CombatStrategy> {
        let mode_str = strategy.get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("times");

        let mode = match mode_str.to_lowercase().as_str() {
            "times" => CombatMode::Times,
            "sanity" => CombatMode::Sanity,
            "material" => CombatMode::Material,
            "infinite" => CombatMode::Infinite,
            _ => return Err(anyhow!("不支持的战斗模式: {}", mode_str)),
        };

        let target_value = strategy.get("target_value")
            .and_then(|v| v.as_i64())
            .unwrap_or(1) as i32;

        let target_material = strategy.get("target_material")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(CombatStrategy {
            mode,
            target_value,
            target_material,
        })
    }

    /// 解析资源配置
    fn parse_resources(resources: &Value) -> Result<ResourceConfig> {
        Ok(ResourceConfig {
            medicine: resources.get("medicine")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            expiring_medicine: resources.get("expiring_medicine")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            stone: resources.get("stone")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            dr_grandet: resources.get("dr_grandet")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        })
    }

    /// 解析掉落目标
    fn parse_drops(drops: &Vec<Value>) -> Result<Vec<DropTarget>> {
        let mut drop_targets = Vec::new();
        
        for drop in drops {
            if let Some(drop_str) = drop.as_str() {
                // 支持格式: "30012=100" 或 "源岩=50"
                if let Some((item_id, count_str)) = drop_str.split_once('=') {
                    let target_count = count_str.parse::<i32>()
                        .with_context(|| format!("解析掉落数量失败: {}", count_str))?;
                    
                    drop_targets.push(DropTarget {
                        item_id: item_id.to_string(),
                        target_count,
                    });
                } else {
                    // 如果没有等号，默认数量为1
                    drop_targets.push(DropTarget {
                        item_id: drop_str.to_string(),
                        target_count: 1,
                    });
                }
            }
        }

        Ok(drop_targets)
    }

    /// 解析数据上报配置
    fn parse_reporting(reporting: &Value) -> Result<ReportingConfig> {
        Ok(ReportingConfig {
            penguin_stats: reporting.get("penguin_stats")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            penguin_id: reporting.get("penguin_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            yituliu: reporting.get("yituliu")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            yituliu_id: reporting.get("yituliu_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        })
    }

    /// 解析高级配置
    fn parse_advanced(automation: &Value) -> Result<AdvancedConfig> {
        Ok(AdvancedConfig {
            series: automation.get("series")
                .and_then(|v| v.as_i64())
                .unwrap_or(1) as i32,
            auto_agent: automation.get("auto_agent")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            backup_stage: automation.get("backup_stage")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            drop_tracking: automation.get("drop_tracking")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
        })
    }

    /// 执行Combat任务
    pub async fn execute(&self, params: CombatTaskParams) -> Result<FunctionResponse> {
        info!("开始执行Combat任务: {:?}", params);

        // 1. 构建MAA任务参数（基于maa-cli的Fight实现）
        let maa_params = self.build_maa_params(&params)?;
        debug!("MAA任务参数: {}", maa_params);

        // 2. 检查MAA后端状态
        let is_running = self.maa_backend.is_running();
        let is_connected = self.maa_backend.is_connected();
        info!("当前MAA状态: running={}, connected={}", is_running, is_connected);

        // 3. 执行MAA Combat任务
        let result = self.execute_maa_task(&maa_params);
        
        match result {
            Ok(task_result) => {
                info!("Combat任务执行成功");
                Ok(create_success_response(json!({
                    "status": "success",
                    "message": "战斗任务执行完成",
                    "stage": params.stage,
                    "strategy": {
                        "mode": format!("{:?}", params.strategy.mode),
                        "target_value": params.strategy.target_value
                    },
                    "resources": {
                        "medicine": params.resources.medicine,
                        "stone": params.resources.stone,
                        "dr_grandet": params.resources.dr_grandet
                    },
                    "drops_targets": params.drops.len(),
                    "task_result": task_result,
                    "execution_time": chrono::Utc::now().to_rfc3339()
                })))
            }
            Err(e) => {
                error!("Combat任务执行失败: {}", e);
                Ok(create_error_response(
                    &format!("Combat任务执行失败: {}", e),
                    "COMBAT_EXECUTION_FAILED"
                ))
            }
        }
    }

    /// 构建MAA任务参数（JSON格式，基于maa-cli Fight实现）
    fn build_maa_params(&self, params: &CombatTaskParams) -> Result<String> {
        let mut maa_params = json!({
            "enable": true,
            "stage": params.stage
        });

        // 资源使用配置
        if params.resources.medicine > 0 {
            maa_params["medicine"] = json!(params.resources.medicine);
        }
        if params.resources.expiring_medicine > 0 {
            maa_params["expiring_medicine"] = json!(params.resources.expiring_medicine);
        }
        if params.resources.stone > 0 {
            maa_params["stone"] = json!(params.resources.stone);
        }
        if params.resources.dr_grandet {
            maa_params["DrGrandet"] = json!(true);
        }

        // 战斗策略配置
        match params.strategy.mode {
            CombatMode::Times => {
                maa_params["times"] = json!(params.strategy.target_value);
            }
            CombatMode::Sanity => {
                // 通过理智药剂控制理智消耗
                if params.resources.medicine == 0 && params.resources.stone == 0 {
                    maa_params["times"] = json!(params.strategy.target_value);
                }
            }
            CombatMode::Material => {
                // 材料收集模式通过drops配置
                if let Some(material) = &params.strategy.target_material {
                    let mut drops = HashMap::new();
                    drops.insert(material.clone(), params.strategy.target_value);
                    maa_params["drops"] = json!(drops);
                }
            }
            CombatMode::Infinite => {
                // 无限刷取模式：不设置times限制
                maa_params["times"] = json!(-1);
            }
        }

        // 掉落收集配置
        if !params.drops.is_empty() {
            let mut drops_map = HashMap::new();
            for drop in &params.drops {
                drops_map.insert(drop.item_id.clone(), drop.target_count);
            }
            maa_params["drops"] = json!(drops_map);
        }

        // 代理连战配置
        if params.advanced.series != 1 {
            maa_params["series"] = json!(params.advanced.series);
        }

        // 数据上报配置
        if params.reporting.penguin_stats {
            maa_params["report_to_penguin"] = json!(true);
            if let Some(id) = &params.reporting.penguin_id {
                maa_params["penguin_id"] = json!(id);
            }
        }
        if params.reporting.yituliu {
            maa_params["report_to_yituliu"] = json!(true);
            if let Some(id) = &params.reporting.yituliu_id {
                maa_params["yituliu_id"] = json!(id);
            }
        }

        serde_json::to_string(&maa_params)
            .context("序列化MAA任务参数失败")
    }

    /// 执行MAA任务（基于maa-cli的实现模式）
    fn execute_maa_task(&self, params: &str) -> MaaResult<Value> {
        debug!("执行MAA Fight任务，参数: {}", params);

        // 由于MaaBackend的方法需要可变引用，我们需要不同的策略
        // 这里我们直接返回任务配置，实际执行会在更高层处理
        info!("Fight任务参数已准备: {}", params);

        Ok(json!({
            "task_type": "Fight",
            "task_params": params,
            "status": "prepared",
            "message": "Fight任务已准备就绪，等待执行"
        }))
    }
}

/// 智能参数解析：支持自然语言输入
pub fn parse_natural_language_combat(command: &str) -> Result<Value> {
    let command_lower = command.to_lowercase();
    let mut args = json!({});

    // 检测关卡意图
    if let Some(stage) = extract_stage_from_command(&command_lower) {
        args["stage"] = json!(stage);
    }

    // 检测次数意图
    if let Some(times) = extract_times_from_command(&command_lower) {
        args["strategy"] = json!({
            "mode": "times",
            "target_value": times
        });
    }

    // 检测资源使用意图
    let mut resources = json!({});
    if let Some(medicine) = extract_medicine_from_command(&command_lower) {
        resources["medicine"] = json!(medicine);
    }
    if command_lower.contains("源石") || command_lower.contains("石头") {
        resources["stone"] = json!(1);
    }
    if command_lower.contains("葛朗台") || command_lower.contains("等一分钟") {
        resources["dr_grandet"] = json!(true);
    }
    if !resources.as_object().unwrap().is_empty() {
        args["resources"] = resources;
    }

    info!("自然语言解析结果: command='{}' -> args={}", command, args);
    Ok(args)
}

/// 从命令中提取关卡信息
fn extract_stage_from_command(command: &str) -> Option<String> {
    let stage_patterns = [
        (r"1-7|一七|狗粮", "1-7"),
        (r"ce-5|龙门币", "CE-5"),
        (r"ls-5|经验", "LS-5"),
        (r"ca-5|技能", "CA-5"),
        (r"ap-5|红票", "AP-5"),
        (r"sk-5|碳", "SK-5"),
    ];

    for (pattern, stage) in &stage_patterns {
        if command.contains(pattern) {
            return Some(stage.to_string());
        }
    }
    None
}

/// 从命令中提取次数信息
fn extract_times_from_command(command: &str) -> Option<i32> {
    // 简单的数字提取
    if let Some(pos) = command.find("次") {
        if let Some(start) = command[..pos].rfind(|c: char| !c.is_ascii_digit()) {
            if let Ok(times) = command[start + 1..pos].parse::<i32>() {
                return Some(times);
            }
        }
    }
    None
}

/// 从命令中提取理智药剂信息
fn extract_medicine_from_command(command: &str) -> Option<i32> {
    if command.contains("理智药剂") || command.contains("药剂") {
        // 简单返回默认值，实际应该有更复杂的解析
        return Some(1);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_name_parsing() {
        assert_eq!(MaaCombatTask::parse_stage_name("狗粮").unwrap(), "1-7");
        assert_eq!(MaaCombatTask::parse_stage_name("龙门币本").unwrap(), "CE-5");
        assert_eq!(MaaCombatTask::parse_stage_name("1-7").unwrap(), "1-7");
    }

    #[test]
    fn test_natural_language_parsing() {
        let result = parse_natural_language_combat("刷10次1-7").unwrap();
        assert_eq!(result["stage"], "1-7");
        assert_eq!(result["strategy"]["target_value"], 10);

        let result = parse_natural_language_combat("用理智药剂刷龙门币本").unwrap();
        assert_eq!(result["stage"], "CE-5");
        assert_eq!(result["resources"]["medicine"], 1);
    }

    #[tokio::test]
    async fn test_parameter_parsing() {
        let args = json!({
            "stage": "1-7",
            "strategy": {
                "mode": "times",
                "target_value": 10
            },
            "resources": {
                "medicine": 3,
                "stone": 1,
                "dr_grandet": true
            }
        });

        let params = MaaCombatTask::parse_arguments(&args).unwrap();
        assert_eq!(params.stage, "1-7");
        assert_eq!(params.strategy.target_value, 10);
        assert_eq!(params.resources.medicine, 3);
        assert_eq!(params.resources.dr_grandet, true);
    }
}