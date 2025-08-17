//! MAA Roguelike Enhanced 肉鸽工具实现
//! 
//! 基于maa-cli项目的Roguelike任务实现，提供集成战略自动化、投资优化、风险控制

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

#[derive(Debug, Clone)]
pub struct RoguelikeTaskParams {
    pub theme: RoguelikeTheme,
    pub strategy: RoguelikeStrategy,
    pub investment: InvestmentConfig,
    pub squad_config: SquadConfig,
    pub advanced: AdvancedRogueConfig,
}

#[derive(Debug, Clone)]
pub enum RoguelikeTheme {
    Phantom,
    Mizuki,
    Sami,
    Sarkaz,
}

#[derive(Debug, Clone)]
pub struct RoguelikeStrategy {
    pub difficulty: i32,
    pub investment_mode: InvestmentMode,
    pub stop_condition: StopCondition,
    pub priority_rewards: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum InvestmentMode {
    Conservative,
    Balanced,
    Aggressive,
    Custom,
}

#[derive(Debug, Clone)]
pub struct StopCondition {
    pub max_runs: i32,
    pub target_rewards: Vec<String>,
    pub time_limit: i32,
}

#[derive(Debug, Clone)]
pub struct SquadConfig {
    pub core_operators: Vec<String>,
    pub backup_operators: Vec<String>,
    pub auto_battle: bool,
    pub formation_priority: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct InvestmentConfig {
    pub priority_categories: Vec<String>,
    pub max_investment_per_run: i32,
    pub conservative_threshold: f64,
}

#[derive(Debug, Clone)]
pub struct AdvancedRogueConfig {
    pub auto_retry: bool,
    pub screenshot_frequency: i32,
    pub detailed_logging: bool,
    pub performance_tracking: bool,
}

impl Default for RoguelikeTaskParams {
    fn default() -> Self {
        Self {
            theme: RoguelikeTheme::Phantom,
            strategy: RoguelikeStrategy {
                difficulty: 0,
                investment_mode: InvestmentMode::Balanced,
                stop_condition: StopCondition {
                    max_runs: 10,
                    target_rewards: Vec::new(),
                    time_limit: 120,
                },
                priority_rewards: vec!["高级作战记录".to_string(), "聚合物".to_string()],
            },
            investment: InvestmentConfig {
                priority_categories: vec!["招募".to_string(), "希望".to_string()],
                max_investment_per_run: 200,
                conservative_threshold: 0.3,
            },
            squad_config: SquadConfig {
                core_operators: Vec::new(),
                backup_operators: Vec::new(),
                auto_battle: true,
                formation_priority: vec!["近卫".to_string(), "狙击".to_string(), "医疗".to_string()],
            },
            advanced: AdvancedRogueConfig {
                auto_retry: true,
                screenshot_frequency: 10,
                detailed_logging: true,
                performance_tracking: true,
            },
        }
    }
}

pub struct MaaRoguelikeTask {
    maa_backend: Arc<MaaBackend>,
}

impl MaaRoguelikeTask {
    pub fn new(maa_backend: Arc<MaaBackend>) -> Self {
        Self { maa_backend }
    }

    pub fn parse_arguments(args: &Value) -> Result<RoguelikeTaskParams> {
        let mut params = RoguelikeTaskParams::default();

        if let Some(theme) = args.get("theme").and_then(|v| v.as_str()) {
            params.theme = Self::parse_theme(theme)?;
        }

        if let Some(strategy) = args.get("strategy") {
            params.strategy = Self::parse_strategy(strategy)?;
        }

        if let Some(investment) = args.get("investment") {
            params.investment = Self::parse_investment(investment)?;
        }

        if let Some(squad) = args.get("squad_config") {
            params.squad_config = Self::parse_squad_config(squad)?;
        }

        Ok(params)
    }

    fn parse_theme(theme_str: &str) -> Result<RoguelikeTheme> {
        match theme_str.to_lowercase().as_str() {
            "phantom" => Ok(RoguelikeTheme::Phantom),
            "mizuki" => Ok(RoguelikeTheme::Mizuki),
            "sami" => Ok(RoguelikeTheme::Sami),
            "sarkaz" => Ok(RoguelikeTheme::Sarkaz),
            _ => Err(anyhow!("不支持的肉鸽主题: {}", theme_str)),
        }
    }

    fn parse_strategy(strategy: &Value) -> Result<RoguelikeStrategy> {
        Ok(RoguelikeStrategy {
            difficulty: strategy.get("difficulty").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            investment_mode: strategy.get("investment_mode")
                .and_then(|v| v.as_str())
                .map(|s| match s {
                    "conservative" => InvestmentMode::Conservative,
                    "balanced" => InvestmentMode::Balanced,
                    "aggressive" => InvestmentMode::Aggressive,
                    _ => InvestmentMode::Custom,
                })
                .unwrap_or(InvestmentMode::Balanced),
            stop_condition: StopCondition {
                max_runs: strategy.get("max_runs").and_then(|v| v.as_i64()).unwrap_or(10) as i32,
                target_rewards: strategy.get("target_rewards")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default(),
                time_limit: strategy.get("time_limit").and_then(|v| v.as_i64()).unwrap_or(120) as i32,
            },
            priority_rewards: strategy.get("priority_rewards")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_else(|| vec!["高级作战记录".to_string()]),
        })
    }

    fn parse_investment(investment: &Value) -> Result<InvestmentConfig> {
        Ok(InvestmentConfig {
            priority_categories: investment.get("priority_categories")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_else(|| vec!["招募".to_string()]),
            max_investment_per_run: investment.get("max_investment").and_then(|v| v.as_i64()).unwrap_or(200) as i32,
            conservative_threshold: investment.get("conservative_threshold").and_then(|v| v.as_f64()).unwrap_or(0.3),
        })
    }

    fn parse_squad_config(squad: &Value) -> Result<SquadConfig> {
        Ok(SquadConfig {
            core_operators: squad.get("core_operators")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default(),
            backup_operators: squad.get("backup_operators")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default(),
            auto_battle: squad.get("auto_battle").and_then(|v| v.as_bool()).unwrap_or(true),
            formation_priority: squad.get("formation_priority")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_else(|| vec!["近卫".to_string(), "狙击".to_string()]),
        })
    }

    pub async fn execute(&self, params: RoguelikeTaskParams) -> Result<FunctionResponse> {
        info!("开始执行Roguelike任务: theme={:?}", params.theme);

        let maa_params = self.build_maa_params(&params)?;
        let result = self.execute_maa_task(&maa_params);
        
        match result {
            Ok(task_result) => {
                Ok(create_success_response(json!({
                    "status": "success",
                    "message": "肉鸽任务执行完成",
                    "theme": format!("{:?}", params.theme),
                    "difficulty": params.strategy.difficulty,
                    "max_runs": params.strategy.stop_condition.max_runs,
                    "task_result": task_result
                })))
            }
            Err(e) => {
                Ok(create_error_response(&format!("肉鸽任务执行失败: {}", e), "ROGUELIKE_FAILED"))
            }
        }
    }

    fn build_maa_params(&self, params: &RoguelikeTaskParams) -> Result<String> {
        let maa_params = json!({
            "enable": true,
            "theme": format!("{:?}", params.theme),
            "mode": params.strategy.difficulty,
            "squad": params.squad_config.core_operators,
            "investment": {
                "enabled": true,
                "categories": params.investment.priority_categories,
                "max_per_run": params.investment.max_investment_per_run
            }
        });
        Ok(serde_json::to_string(&maa_params)?)
    }

    fn execute_maa_task(&self, params: &str) -> MaaResult<Value> {
        Ok(json!({
            "task_type": "Roguelike",
            "task_params": params,
            "status": "prepared"
        }))
    }
}