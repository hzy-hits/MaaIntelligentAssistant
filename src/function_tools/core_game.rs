//! 核心游戏功能模块 - V2简化版本
//!
//! 包含4个核心MAA游戏功能定义：
//! - maa_startup: 游戏启动管理
//! - maa_combat_enhanced: 增强战斗系统
//! - maa_recruit_enhanced: 智能招募管理
//! - maa_infrastructure_enhanced: 基建自动化

use serde_json::json;
use super::types::FunctionDefinition;

/// 创建启动任务工具定义
pub fn create_startup_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_startup".to_string(),
        description: "启动明日方舟游戏并进行初始化设置".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "client_type": {
                    "type": "string",
                    "description": "客户端类型 (Official/Bilibili/YoStarEN/YoStarJP/YoStarKR/Txwy)",
                    "enum": ["Official", "Bilibili", "YoStarEN", "YoStarJP", "YoStarKR", "Txwy"],
                    "default": "Official"
                },
                "start_app": {
                    "type": "boolean",
                    "description": "是否启动应用程序",
                    "default": true
                }
            },
            "required": []
        }),
    }
}

/// 创建战斗增强工具定义
pub fn create_combat_enhanced_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_combat_enhanced".to_string(),
        description: "执行增强战斗任务，支持智能关卡选择和资源管理".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "stage": {
                    "type": "string",
                    "description": "关卡名称 (如: 1-7, CE-5, CA-5, AP-5)",
                    "default": "1-7"
                },
                "times": {
                    "type": "integer",
                    "description": "战斗次数",
                    "default": 1,
                    "minimum": 1
                },
                "use_medicine": {
                    "type": "boolean",
                    "description": "是否使用体力药剂",
                    "default": false
                },
                "use_stone": {
                    "type": "boolean", 
                    "description": "是否使用源石回复体力",
                    "default": false
                }
            },
            "required": ["stage"]
        }),
    }
}

/// 创建招募增强工具定义
pub fn create_recruit_enhanced_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_recruit_enhanced".to_string(),
        description: "执行智能公开招募，支持标签识别和自动选择".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "max_times": {
                    "type": "integer",
                    "description": "最大招募次数",
                    "default": 4,
                    "minimum": 1,
                    "maximum": 4
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

/// 创建基建增强工具定义
pub fn create_infrastructure_enhanced_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_infrastructure_enhanced".to_string(),
        description: "执行基建管理任务，支持收菜、换班、生产管理".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "operation_mode": {
                    "type": "string",
                    "description": "操作模式: full_auto(全自动), collect_only(仅收取), custom(自定义)",
                    "enum": ["full_auto", "collect_only", "custom"],
                    "default": "full_auto"
                },
                "facilities": {
                    "type": "array",
                    "description": "自定义模式下的设施列表",
                    "items": {
                        "type": "string",
                        "enum": ["Mfg", "Trade", "Power", "Control", "Reception", "Office", "Dorm"]
                    },
                    "default": ["Mfg", "Trade", "Power", "Control"]
                }
            },
            "required": []
        }),
    }
}