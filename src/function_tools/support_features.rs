//! 辅助功能模块 - V2简化版本
//!
//! 包含4个辅助MAA功能定义：
//! - maa_rewards_enhanced: 奖励收集增强
//! - maa_credit_store_enhanced: 信用商店增强
//! - maa_depot_management: 仓库管理
//! - maa_operator_box: 干员整理

use serde_json::json;
use super::types::FunctionDefinition;

/// 创建奖励增强工具定义
pub fn create_rewards_enhanced_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_rewards_enhanced".to_string(),
        description: "收集各种奖励，包括邮件、任务奖励等".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "award_type": {
                    "type": "string",
                    "description": "奖励类型 (all/mail/mission)",
                    "enum": ["all", "mail", "mission"],
                    "default": "all"
                }
            },
            "required": []
        }),
    }
}

/// 创建信用商店增强工具定义
pub fn create_credit_store_enhanced_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_credit_store_enhanced".to_string(),
        description: "自动购买信用商店物品，支持优先级设置".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "buy_first": {
                    "type": "array",
                    "description": "优先购买的物品列表",
                    "items": {
                        "type": "string"
                    },
                    "default": ["龙门币", "赤金"]
                },
                "blacklist": {
                    "type": "array",
                    "description": "不购买的物品黑名单",
                    "items": {
                        "type": "string"
                    },
                    "default": []
                }
            },
            "required": []
        }),
    }
}

/// 创建仓库管理工具定义
pub fn create_depot_management_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_depot_management".to_string(),
        description: "执行仓库整理和管理任务".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "enable": {
                    "type": "boolean",
                    "description": "是否启用仓库管理",
                    "default": true
                }
            },
            "required": []
        }),
    }
}

/// 创建干员整理工具定义
pub fn create_operator_box_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_operator_box".to_string(),
        description: "执行干员整理和管理任务".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "enable": {
                    "type": "boolean",
                    "description": "是否启用干员整理",
                    "default": true
                }
            },
            "required": []
        }),
    }
}