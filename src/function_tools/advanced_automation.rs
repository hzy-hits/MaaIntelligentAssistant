//! 高级自动化模块
//!
//! 包含4个高级MAA自动化功能：
//! - maa_roguelike_enhanced: 肉鸽自动化
//! - maa_copilot_enhanced: 作业自动执行  
//! - maa_sss_copilot: SSS级作业
//! - maa_reclamation: 生息演算

use serde_json::{json, Value};
use tracing::{debug, info};

use super::types::{FunctionDefinition, FunctionResponse, MaaError};
use super::queue_client::MaaQueueClient;
use std::time::Instant;

/// 创建肉鸽增强工具定义
pub fn create_roguelike_enhanced_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_roguelike_enhanced".to_string(),
        description: "增强的肉鸽自动化系统，支持多种肉鸽模式".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "theme": {
                    "type": "string",
                    "description": "肉鸽主题",
                    "enum": ["Phantom", "Mizuki", "Sami", "Sarkaz"],
                    "default": "Phantom"
                },
                "mode": {
                    "type": "integer",
                    "description": "肉鸽模式：0-刷蜡烛，1-刷源石锭，2-两者兼顾，3-尝试通关，4-收集模式",
                    "enum": [0, 1, 2, 3, 4],
                    "default": 0
                },
                "starts_count": {
                    "type": "integer",
                    "description": "开始探索次数",
                    "minimum": 1,
                    "maximum": 9999,
                    "default": 99999
                },
                "investments_count": {
                    "type": "integer", 
                    "description": "投资次数限制",
                    "minimum": 0,
                    "maximum": 9999,
                    "default": 99999
                },
                "stop_when_investment_full": {
                    "type": "boolean",
                    "description": "投资满时是否停止",
                    "default": false
                },
                "squad": {
                    "type": "string",
                    "description": "作战分队名",
                    "default": "指挥分队1"
                },
                "roles": {
                    "type": "string", 
                    "description": "职业组合",
                    "default": "取长补短"
                },
                "core_char": {
                    "type": "string",
                    "description": "核心干员"
                }
            },
            "required": []
        }),
    }
}

/// 处理肉鸽增强任务
pub async fn handle_roguelike_enhanced(args: Value, queue_client: &MaaQueueClient) -> FunctionResponse {
    let start_time = Instant::now();
    info!("🃏 处理肉鸽增强任务");
    
    let theme = args.get("theme")
        .and_then(|v| v.as_str())
        .unwrap_or("Phantom");
        
    let mode = args.get("mode")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
        
    let starts_count = args.get("starts_count")
        .and_then(|v| v.as_i64())
        .unwrap_or(99999) as i32;

    debug!("肉鸽参数: theme={}, mode={}, starts_count={}", 
           theme, mode, starts_count);

    match queue_client.roguelike(theme.to_string(), mode, starts_count).await {
        Ok(result) => {
            info!("肉鸽增强任务完成");
            let response_data = json!({
                "status": "success",
                "message": format!("肉鸽 {} 任务完成", theme),
                "theme": theme,
                "mode": mode,
                "starts_count": starts_count,
                "details": result
            });
            FunctionResponse::success("maa_roguelike_enhanced", response_data)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        },
        Err(e) => {
            let error = MaaError::maa_core_error(&format!("肉鸽任务失败: {}", e), Some("检查肉鸽模式是否已开放，理智是否充足"));
            debug!("肉鸽任务失败: {}", e);
            FunctionResponse::error("maa_roguelike_enhanced", error)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        }
    }
}

/// 创建作业增强工具定义
pub fn create_copilot_enhanced_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_copilot_enhanced".to_string(),
        description: "增强的作业自动执行系统".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "filename": {
                    "type": "string",
                    "description": "作业文件路径"
                },
                "formation": {
                    "type": "boolean",
                    "description": "是否自动编队",
                    "default": false
                },
                "support_unit_name": {
                    "type": "string",
                    "description": "指定助战单位名称"
                },
                "stage_name": {
                    "type": "string",
                    "description": "关卡名称，如'1-7', 'CE-5'"
                },
                "loop": {
                    "type": "object",
                    "description": "循环设置",
                    "properties": {
                        "enable": {"type": "boolean", "default": false},
                        "times": {"type": "integer", "minimum": 1, "default": 1}
                    }
                }
            },
            "required": []
        }),
    }
}

/// 处理作业增强任务
pub async fn handle_copilot_enhanced(args: Value, queue_client: &MaaQueueClient) -> FunctionResponse {
    let start_time = Instant::now();
    info!("处理作业增强任务");
    
    let filename = args.get("filename")
        .and_then(|v| v.as_str())
        .unwrap_or("");
        
    let formation = args.get("formation")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
        
    let stage_name = args.get("stage_name")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    debug!("作业参数: filename={}, formation={}, stage_name={}", 
           filename, formation, stage_name);

    match queue_client.copilot(filename.to_string(), formation).await {
        Ok(result) => {
            info!("作业增强任务完成");
            let response_data = json!({
                "status": "success", 
                "message": "作业任务已完成",
                "filename": filename,
                "stage_name": stage_name,
                "formation": formation,
                "details": result
            });
            FunctionResponse::success("maa_copilot_enhanced", response_data)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        },
        Err(e) => {
            let error = MaaError::maa_core_error(&format!("作业任务失败: {}", e), Some("检查作业文件路径和关卡名称"));
            debug!("作业任务失败: {}", e);
            FunctionResponse::error("maa_copilot_enhanced", error)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        }
    }
}

/// 创建SSS作业工具定义
pub fn create_sss_copilot_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_sss_copilot".to_string(),
        description: "SSS级高难度作业执行系统".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "stage_name": {
                    "type": "string",
                    "description": "SSS关卡名称",
                    "pattern": "^[A-Z]{1,2}-[0-9]+$"
                },
                "loop": {
                    "type": "object",
                    "description": "循环设置",
                    "properties": {
                        "enable": {"type": "boolean", "default": false},
                        "times": {"type": "integer", "minimum": 1, "maximum": 10, "default": 1}
                    }
                },
                "formation": {
                    "type": "boolean",
                    "description": "是否自动编队", 
                    "default": true
                },
                "support_unit_name": {
                    "type": "string",
                    "description": "指定助战单位"
                }
            },
            "required": ["stage_name"]
        }),
    }
}

/// 处理SSS作业任务
pub async fn handle_sss_copilot(args: Value, queue_client: &MaaQueueClient) -> FunctionResponse {
    let start_time = Instant::now();
    info!("🌟 处理SSS作业任务");
    
    let stage_name = match args.get("stage_name").and_then(|v| v.as_str()) {
        Some(name) => name,
        None => {
            let error = MaaError::validation_error("缺少必要参数: stage_name", Some("请提供有效的关卡名称"));
            return FunctionResponse::error("maa_sss_copilot", error)
                .with_execution_time(start_time.elapsed().as_millis() as u64);
        }
    };
        
    let formation = args.get("formation")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
        
    let loop_times = args.get("loop")
        .and_then(|v| v.get("times"))
        .and_then(|v| v.as_i64())
        .unwrap_or(1) as i32;

    debug!("SSS作业参数: stage_name={}, formation={}, loop_times={}", 
           stage_name, formation, loop_times);

    match queue_client.sss_copilot(format!("sss_{}.json", stage_name), loop_times).await {
        Ok(result) => {
            info!("SSS作业任务完成: {}", stage_name);
            let response_data = json!({
                "status": "success",
                "message": format!("SSS关卡 {} 作业完成", stage_name),
                "stage_name": stage_name,
                "formation": formation,
                "loop_times": loop_times,
                "details": result
            });
            FunctionResponse::success("maa_sss_copilot", response_data)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        },
        Err(e) => {
            let error = MaaError::maa_core_error(&format!("SSS作业任务失败: {}", e), Some("检查SSS作业文件和关卡名称"));
            debug!("SSS作业任务失败: {}", e);
            FunctionResponse::error("maa_sss_copilot", error)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        }
    }
}

/// 创建生息演算工具定义  
pub fn create_reclamation_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_reclamation".to_string(),
        description: "生息演算自动化系统".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "theme": {
                    "type": "string", 
                    "description": "生息演算主题",
                    "enum": ["Fire", "Tales"],
                    "default": "Fire"
                },
                "mode": {
                    "type": "integer",
                    "description": "演算模式：0-默认，1-保守，2-激进",
                    "enum": [0, 1, 2],
                    "default": 0
                },
                "tool_to_craft": {
                    "type": "string",
                    "description": "优先制造的道具"
                },
                "increment_mode": {
                    "type": "integer",
                    "description": "递增模式",
                    "minimum": 0,
                    "maximum": 2,
                    "default": 0
                }
            },
            "required": []
        }),
    }
}

/// 处理生息演算任务
pub async fn handle_reclamation(args: Value, queue_client: &MaaQueueClient) -> FunctionResponse {
    let start_time = Instant::now();
    info!("🌱 处理生息演算任务");
    
    let theme = args.get("theme")
        .and_then(|v| v.as_str())
        .unwrap_or("Fire");
        
    let mode = args.get("mode") 
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
        
    let tool_to_craft = args.get("tool_to_craft")
        .and_then(|v| v.as_str())
        .map(|s| vec![s.to_string()])
        .unwrap_or_else(Vec::new);

    debug!("生息演算参数: theme={}, mode={}, tools={:?}", theme, mode, tool_to_craft);

    match queue_client.reclamation(theme.to_string(), mode).await {
        Ok(result) => {
            info!("生息演算任务完成: {}", theme);
            let response_data = json!({
                "status": "success",
                "message": format!("生息演算 {} 任务完成", theme),
                "theme": theme,
                "mode": mode, 
                "details": result
            });
            FunctionResponse::success("maa_reclamation", response_data)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        },
        Err(e) => {
            let error = MaaError::maa_core_error(&format!("生息演算任务失败: {}", e), Some("检查生息演算模式是否已开放"));
            debug!("生息演算任务失败: {}", e);
            FunctionResponse::error("maa_reclamation", error)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        }
    }
}