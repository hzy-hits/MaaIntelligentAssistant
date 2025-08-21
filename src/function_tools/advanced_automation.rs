//! 高级自动化模块 - V2简化版本
//!
//! 包含4个高级MAA自动化功能定义：
//! - maa_roguelike_enhanced: 肉鸽自动化
//! - maa_copilot_enhanced: 作业自动执行  
//! - maa_sss_copilot: SSS级作业
//! - maa_reclamation: 生息演算

use serde_json::json;
use super::types::FunctionDefinition;

/// 创建肉鸽增强工具定义
pub fn create_roguelike_enhanced_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_roguelike_enhanced".to_string(),
        description: "执行集成战略(肉鸽)任务，支持多种主题和模式".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "theme": {
                    "type": "string",
                    "description": "肉鸽主题 (Phantom/Mizuki/Sami/Sarkaz)",
                    "enum": ["Phantom", "Mizuki", "Sami", "Sarkaz"],
                    "default": "Phantom"
                },
                "mode": {
                    "type": "integer",
                    "description": "游戏模式 (0:刷蜡烛, 1:刷源石锭, 2:【投资】效益优先, 3:【投资】次数优先, 4:【投资】常规刷启动)",
                    "enum": [0, 1, 2, 3, 4],
                    "default": 0
                }
            },
            "required": []
        }),
    }
}

/// 创建作业增强工具定义
pub fn create_copilot_enhanced_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_copilot_enhanced".to_string(),
        description: "执行MAA作业文件，支持自动化关卡通关".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "filename": {
                    "type": "string",
                    "description": "作业文件名或路径"
                },
                "formation": {
                    "type": "boolean",
                    "description": "是否自动编队",
                    "default": false
                }
            },
            "required": ["filename"]
        }),
    }
}

/// 创建SSS作业工具定义
pub fn create_sss_copilot_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_sss_copilot".to_string(),
        description: "执行保全派驻SSS作业".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "filename": {
                    "type": "string",
                    "description": "SSS作业文件名或路径"
                }
            },
            "required": ["filename"]
        }),
    }
}

/// 创建生息演算工具定义
pub fn create_reclamation_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_reclamation".to_string(),
        description: "执行生息演算任务".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "theme": {
                    "type": "string",
                    "description": "演算主题 (Fire/Tales)",
                    "enum": ["Fire", "Tales"],
                    "default": "Fire"
                },
                "mode": {
                    "type": "integer",
                    "description": "演算模式",
                    "default": 0
                }
            },
            "required": []
        }),
    }
}