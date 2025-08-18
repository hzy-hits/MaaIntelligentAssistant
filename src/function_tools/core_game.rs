//! 核心游戏功能模块
//!
//! 包含4个核心MAA游戏功能：
//! - maa_startup: 游戏启动管理
//! - maa_combat_enhanced: 增强战斗系统
//! - maa_recruit_enhanced: 智能招募管理
//! - maa_infrastructure_enhanced: 基建自动化

use serde_json::{json, Value};
use tracing::{debug, info};

use crate::maa_core::{execute_startup, execute_fight, execute_recruit, execute_infrastructure};
use super::types::FunctionDefinition;

/// 创建启动任务工具定义
pub fn create_startup_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_startup".to_string(),
        description: "启动游戏和基础设置配置".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "client_type": {
                    "type": "string",
                    "description": "客户端类型",
                    "enum": ["Official", "Bilibili", "txwy", "YoStarEN", "YoStarJP", "YoStarKR"]
                },
                "start_app": {
                    "type": "boolean", 
                    "description": "是否启动应用程序",
                    "default": true
                },
                "close_app": {
                    "type": "boolean",
                    "description": "任务完成后是否关闭应用",
                    "default": false
                },
                "account_name": {
                    "type": "string",
                    "description": "账户名称（可选）"
                }
            },
            "required": ["client_type"]
        }),
    }
}

/// 处理启动任务
pub async fn handle_startup(args: Value) -> Result<Value, String> {
    info!("🚀 处理游戏启动任务");
    
    let client_type = args.get("client_type")
        .and_then(|v| v.as_str())
        .unwrap_or("Official");
    
    let start_app = args.get("start_app")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
        
    let close_app = args.get("close_app")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    debug!("启动参数: client_type={}, start_app={}, close_app={}", 
           client_type, start_app, close_app);

    match execute_startup(client_type, start_app, close_app).await {
        Ok(result) => {
            info!("✅ 游戏启动任务完成");
            Ok(json!({
                "status": "success",
                "message": "游戏启动任务已完成",
                "client_type": client_type,
                "details": result
            }))
        },
        Err(e) => {
            let error_msg = format!("游戏启动失败: {}", e);
            debug!("❌ {}", error_msg);
            Err(error_msg)
        }
    }
}

/// 创建增强战斗工具定义
pub fn create_combat_enhanced_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_combat_enhanced".to_string(),
        description: "增强的自动战斗系统，支持多种战斗模式和策略".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "stage": {
                    "type": "string",
                    "description": "关卡代码，如'1-7', 'CE-5', 'H12-4'",
                    "pattern": "^[A-Z0-9-]+$"
                },
                "medicine": {
                    "type": "integer",
                    "description": "理智药使用数量",
                    "minimum": 0,
                    "maximum": 999,
                    "default": 0
                },
                "expiring_medicine": {
                    "type": "integer", 
                    "description": "48小时内过期理智药使用数量",
                    "minimum": 0,
                    "maximum": 999,
                    "default": 0
                },
                "stone": {
                    "type": "integer",
                    "description": "源石使用数量", 
                    "minimum": 0,
                    "maximum": 99,
                    "default": 0
                },
                "times": {
                    "type": "integer",
                    "description": "作战次数（-1为无限次）",
                    "minimum": -1,
                    "maximum": 9999,
                    "default": 1
                },
                "drops": {
                    "type": "object",
                    "description": "掉落物品统计设置",
                    "properties": {
                        "enable": {"type": "boolean", "default": true},
                        "item_id": {"type": "string"}
                    }
                },
                "report": {
                    "type": "object", 
                    "description": "作战报告设置",
                    "properties": {
                        "enable": {"type": "boolean", "default": false},
                        "webhook_url": {"type": "string"}
                    }
                }
            },
            "required": ["stage"]
        }),
    }
}

/// 处理增强战斗任务
pub async fn handle_combat_enhanced(args: Value) -> Result<Value, String> {
    info!("⚔️ 处理增强战斗任务");
    
    let stage = args.get("stage")
        .and_then(|v| v.as_str())
        .ok_or("缺少关卡参数")?;
    
    let medicine = args.get("medicine")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
        
    let stone = args.get("stone")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
        
    let times = args.get("times")
        .and_then(|v| v.as_i64())
        .unwrap_or(1) as i32;

    debug!("战斗参数: stage={}, medicine={}, stone={}, times={}", 
           stage, medicine, stone, times);

    match execute_fight(stage, medicine, stone, times).await {
        Ok(result) => {
            info!("✅ 增强战斗任务完成: {}", stage);
            Ok(json!({
                "status": "success",
                "message": format!("关卡 {} 战斗任务完成", stage),
                "stage": stage,
                "times_completed": times,
                "details": result
            }))
        },
        Err(e) => {
            let error_msg = format!("战斗任务失败: {}", e);
            debug!("❌ {}", error_msg);
            Err(error_msg)
        }
    }
}

/// 创建增强招募工具定义
pub fn create_recruit_enhanced_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_recruit_enhanced".to_string(),
        description: "增强的干员招募管理系统".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "select": {
                    "type": "array",
                    "description": "选择的标签组合",
                    "items": {"type": "string"},
                    "default": []
                },
                "confirm": {
                    "type": "array",
                    "description": "确认招募的标签级别",
                    "items": {"type": "integer", "minimum": 3, "maximum": 6},
                    "default": [4, 5, 6]
                },
                "times": {
                    "type": "integer",
                    "description": "招募次数",
                    "minimum": 1,
                    "maximum": 10,
                    "default": 1
                },
                "set_time": {
                    "type": "boolean",
                    "description": "是否设置招募时间",
                    "default": true
                },
                "expedite": {
                    "type": "boolean", 
                    "description": "是否使用加急许可",
                    "default": false
                },
                "skip_robot": {
                    "type": "boolean",
                    "description": "是否跳过小车标签",
                    "default": true
                }
            },
            "required": []
        }),
    }
}

/// 处理增强招募任务
pub async fn handle_recruit_enhanced(args: Value) -> Result<Value, String> {
    info!("🎯 处理增强招募任务");
    
    let times = args.get("times")
        .and_then(|v| v.as_i64())
        .unwrap_or(1) as i32;
        
    let expedite = args.get("expedite")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
        
    let skip_robot = args.get("skip_robot")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    debug!("招募参数: times={}, expedite={}, skip_robot={}", 
           times, expedite, skip_robot);

    match execute_recruit(times, expedite, skip_robot).await {
        Ok(result) => {
            info!("✅ 增强招募任务完成");
            Ok(json!({
                "status": "success",
                "message": "招募任务已完成",
                "times": times,
                "expedite": expedite,
                "details": result
            }))
        },
        Err(e) => {
            let error_msg = format!("招募任务失败: {}", e);
            debug!("❌ {}", error_msg);
            Err(error_msg)
        }
    }
}

/// 创建增强基建工具定义
pub fn create_infrastructure_enhanced_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_infrastructure_enhanced".to_string(),
        description: "增强的基建自动化管理系统".to_string(),
        parameters: json!({
            "type": "object", 
            "properties": {
                "facility": {
                    "type": "array",
                    "description": "设施类型列表",
                    "items": {
                        "type": "string",
                        "enum": ["Mfg", "Trade", "Power", "Control", "Reception", "Office", "Dorm"]
                    },
                    "default": ["Mfg", "Trade", "Power"]
                },
                "dorm_notstationed_enabled": {
                    "type": "boolean",
                    "description": "是否处理宿舍未进驻干员",
                    "default": false  
                },
                "dorm_trust_enabled": {
                    "type": "boolean",
                    "description": "是否启用信赖度收集",
                    "default": true
                },
                "filename": {
                    "type": "string",
                    "description": "排班配置文件名",
                    "default": "plan.json"
                },
                "plan_index": {
                    "type": "integer",
                    "description": "排班方案索引",
                    "minimum": 0,
                    "default": 0
                }
            },
            "required": []
        }),
    }
}

/// 处理增强基建任务
pub async fn handle_infrastructure_enhanced(args: Value) -> Result<Value, String> {
    info!("🏢 处理增强基建任务");
    
    let default_facility = json!(["Mfg", "Trade", "Power"]);
    let facility = args.get("facility").unwrap_or(&default_facility);
    
    let dorm_trust_enabled = args.get("dorm_trust_enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
        
    let filename = args.get("filename")
        .and_then(|v| v.as_str())
        .unwrap_or("plan.json");

    debug!("基建参数: facility={:?}, dorm_trust_enabled={}, filename={}", 
           facility, dorm_trust_enabled, filename);

    match execute_infrastructure(facility.clone(), dorm_trust_enabled, filename).await {
        Ok(result) => {
            info!("✅ 增强基建任务完成");
            Ok(json!({
                "status": "success",
                "message": "基建任务已完成", 
                "facility": facility,
                "dorm_trust_enabled": dorm_trust_enabled,
                "details": result
            }))
        },
        Err(e) => {
            let error_msg = format!("基建任务失败: {}", e);
            debug!("❌ {}", error_msg);
            Err(error_msg)
        }
    }
}