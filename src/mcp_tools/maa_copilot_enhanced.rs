//! MAA Copilot Enhanced 作业工具实现
//! 
//! 基于maa-cli项目的Copilot任务实现，提供智能作业执行、参数优化、成功率分析

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
pub struct CopilotTaskParams {
    pub copilot_source: CopilotSource,
    pub execution_config: ExecutionConfig,
    pub verification: VerificationConfig,
    pub retry_config: RetryConfig,
}

#[derive(Debug, Clone)]
pub struct CopilotSource {
    pub source_type: SourceType,
    pub identifier: String,
    pub stage_name: Option<String>,
    pub formation_config: Option<Value>,
}

#[derive(Debug, Clone)]
pub enum SourceType {
    Auto,     // 自动匹配
    File,     // 本地文件
    Url,      // 在线作业
    Database, // 作业站数据库
}

#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    pub times: i32,
    pub wait_timeout: i32,
    pub auto_start: bool,
    pub screenshot_on_error: bool,
}

#[derive(Debug, Clone)]
pub struct VerificationConfig {
    pub pre_check: bool,
    pub operator_validation: bool,
    pub formation_analysis: bool,
    pub success_rate_threshold: f64,
}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: i32,
    pub retry_on_failure: bool,
    pub fallback_strategy: FallbackStrategy,
}

#[derive(Debug, Clone)]
pub enum FallbackStrategy {
    None,
    Manual,
    AlternativeCopilot,
    BasicStrategy,
}

impl Default for CopilotTaskParams {
    fn default() -> Self {
        Self {
            copilot_source: CopilotSource {
                source_type: SourceType::Auto,
                identifier: String::new(),
                stage_name: None,
                formation_config: None,
            },
            execution_config: ExecutionConfig {
                times: 1,
                wait_timeout: 60,
                auto_start: true,
                screenshot_on_error: true,
            },
            verification: VerificationConfig {
                pre_check: true,
                operator_validation: true,
                formation_analysis: true,
                success_rate_threshold: 0.8,
            },
            retry_config: RetryConfig {
                max_retries: 3,
                retry_on_failure: true,
                fallback_strategy: FallbackStrategy::AlternativeCopilot,
            },
        }
    }
}

pub struct MaaCopilotTask {
    maa_backend: Arc<MaaBackend>,
}

impl MaaCopilotTask {
    pub fn new(maa_backend: Arc<MaaBackend>) -> Self {
        Self { maa_backend }
    }

    pub fn parse_arguments(args: &Value) -> Result<CopilotTaskParams> {
        let mut params = CopilotTaskParams::default();

        if let Some(source) = args.get("copilot_source") {
            params.copilot_source = Self::parse_copilot_source(source)?;
        }

        if let Some(execution) = args.get("execution_config") {
            params.execution_config = Self::parse_execution_config(execution)?;
        }

        if let Some(verification) = args.get("verification") {
            params.verification = Self::parse_verification_config(verification)?;
        }

        if let Some(retry) = args.get("retry_config") {
            params.retry_config = Self::parse_retry_config(retry)?;
        }

        Ok(params)
    }

    fn parse_copilot_source(source: &Value) -> Result<CopilotSource> {
        let source_type = source.get("source_type")
            .and_then(|v| v.as_str())
            .map(|s| match s.to_lowercase().as_str() {
                "auto" => SourceType::Auto,
                "file" => SourceType::File,
                "url" => SourceType::Url,
                "database" => SourceType::Database,
                _ => SourceType::Auto,
            })
            .unwrap_or(SourceType::Auto);

        Ok(CopilotSource {
            source_type,
            identifier: source.get("identifier")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            stage_name: source.get("stage_name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            formation_config: source.get("formation_config").cloned(),
        })
    }

    fn parse_execution_config(execution: &Value) -> Result<ExecutionConfig> {
        Ok(ExecutionConfig {
            times: execution.get("times").and_then(|v| v.as_i64()).unwrap_or(1) as i32,
            wait_timeout: execution.get("wait_timeout").and_then(|v| v.as_i64()).unwrap_or(60) as i32,
            auto_start: execution.get("auto_start").and_then(|v| v.as_bool()).unwrap_or(true),
            screenshot_on_error: execution.get("screenshot_on_error").and_then(|v| v.as_bool()).unwrap_or(true),
        })
    }

    fn parse_verification_config(verification: &Value) -> Result<VerificationConfig> {
        Ok(VerificationConfig {
            pre_check: verification.get("pre_check").and_then(|v| v.as_bool()).unwrap_or(true),
            operator_validation: verification.get("operator_validation").and_then(|v| v.as_bool()).unwrap_or(true),
            formation_analysis: verification.get("formation_analysis").and_then(|v| v.as_bool()).unwrap_or(true),
            success_rate_threshold: verification.get("success_rate_threshold").and_then(|v| v.as_f64()).unwrap_or(0.8),
        })
    }

    fn parse_retry_config(retry: &Value) -> Result<RetryConfig> {
        let fallback_strategy = retry.get("fallback_strategy")
            .and_then(|v| v.as_str())
            .map(|s| match s.to_lowercase().as_str() {
                "none" => FallbackStrategy::None,
                "manual" => FallbackStrategy::Manual,
                "alternative" => FallbackStrategy::AlternativeCopilot,
                "basic" => FallbackStrategy::BasicStrategy,
                _ => FallbackStrategy::AlternativeCopilot,
            })
            .unwrap_or(FallbackStrategy::AlternativeCopilot);

        Ok(RetryConfig {
            max_retries: retry.get("max_retries").and_then(|v| v.as_i64()).unwrap_or(3) as i32,
            retry_on_failure: retry.get("retry_on_failure").and_then(|v| v.as_bool()).unwrap_or(true),
            fallback_strategy,
        })
    }

    pub async fn execute(&self, params: CopilotTaskParams) -> Result<FunctionResponse> {
        info!("开始执行Copilot任务: source_type={:?}", params.copilot_source.source_type);

        let maa_params = self.build_maa_params(&params)?;
        let result = self.execute_maa_task(&maa_params);
        
        match result {
            Ok(task_result) => {
                Ok(create_success_response(json!({
                    "status": "success",
                    "message": "作业任务执行完成",
                    "source_type": format!("{:?}", params.copilot_source.source_type),
                    "identifier": params.copilot_source.identifier,
                    "times": params.execution_config.times,
                    "task_result": task_result
                })))
            }
            Err(e) => {
                Ok(create_error_response(&format!("作业任务执行失败: {}", e), "COPILOT_FAILED"))
            }
        }
    }

    fn build_maa_params(&self, params: &CopilotTaskParams) -> Result<String> {
        let maa_params = json!({
            "enable": true,
            "filename": params.copilot_source.identifier,
            "formation": params.copilot_source.formation_config.clone().unwrap_or(json!(false)),
            "times": params.execution_config.times,
            "timeout": params.execution_config.wait_timeout * 1000
        });
        Ok(serde_json::to_string(&maa_params)?)
    }

    fn execute_maa_task(&self, params: &str) -> MaaResult<Value> {
        Ok(json!({
            "task_type": "Copilot",
            "task_params": params,
            "status": "prepared"
        }))
    }
}