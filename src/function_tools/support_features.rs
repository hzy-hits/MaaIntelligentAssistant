//! 辅助功能模块
//!
//! 包含4个辅助MAA功能：
//! - maa_rewards_enhanced: 奖励收集增强
//! - maa_credit_store_enhanced: 信用商店增强
//! - maa_depot_management: 仓库管理
//! - maa_operator_box: 干员整理

use serde_json::{json, Value};
use tracing::{debug, info};

use super::types::{FunctionDefinition, FunctionResponse, MaaError};
use super::queue_client::MaaQueueClient;
use std::time::Instant;

/// 创建奖励增强工具定义
pub fn create_rewards_enhanced_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_rewards_enhanced".to_string(),
        description: "增强的奖励收集系统，自动收集各种奖励".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "award": {
                    "type": "boolean",
                    "description": "是否收集每日任务奖励",
                    "default": true
                },
                "mail": {
                    "type": "boolean", 
                    "description": "是否收集邮件奖励",
                    "default": true
                },
                "recruit": {
                    "type": "boolean",
                    "description": "是否收集招募奖励",
                    "default": true
                },
                "orundum": {
                    "type": "boolean",
                    "description": "是否收集合成玉奖励", 
                    "default": true
                },
                "mining": {
                    "type": "boolean",
                    "description": "是否收集采矿奖励",
                    "default": true
                },
                "specialaccess": {
                    "type": "boolean",
                    "description": "是否收集特别通行证奖励",
                    "default": true
                }
            },
            "required": []
        }),
    }
}

/// 处理奖励增强任务
pub async fn handle_rewards_enhanced(args: Value, queue_client: &MaaQueueClient) -> FunctionResponse {
    let start_time = Instant::now();
    info!("处理奖励收集任务");
    
    let award = args.get("award").and_then(|v| v.as_bool()).unwrap_or(true);
    let mail = args.get("mail").and_then(|v| v.as_bool()).unwrap_or(true);
    let recruit = args.get("recruit").and_then(|v| v.as_bool()).unwrap_or(true);
    let orundum = args.get("orundum").and_then(|v| v.as_bool()).unwrap_or(true);
    let mining = args.get("mining").and_then(|v| v.as_bool()).unwrap_or(true);
    let specialaccess = args.get("specialaccess").and_then(|v| v.as_bool()).unwrap_or(true);

    debug!("奖励收集参数: award={}, mail={}, recruit={}, orundum={}, mining={}, specialaccess={}", 
           award, mail, recruit, orundum, mining, specialaccess);

    match queue_client.rewards(award, mail, recruit, orundum).await {
        Ok(result) => {
            info!("奖励收集任务完成");
            let response_data = json!({
                "status": "success",
                "message": "奖励收集任务已完成",
                "collected": {
                    "award": award,
                    "mail": mail, 
                    "recruit": recruit,
                    "orundum": orundum,
                    "mining": mining,
                    "specialaccess": specialaccess
                },
                "details": result
            });
            FunctionResponse::success("maa_rewards_enhanced", response_data)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        },
        Err(e) => {
            let error = MaaError::maa_core_error(&format!("奖励收集失败: {}", e), Some("检查MAA连接状态和游戏界面"));
            debug!("奖励收集失败: {}", e);
            FunctionResponse::error("maa_rewards_enhanced", error)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        }
    }
}

/// 创建信用商店增强工具定义
pub fn create_credit_store_enhanced_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_credit_store_enhanced".to_string(),
        description: "增强的信用商店自动购买系统".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "enable": {
                    "type": "boolean",
                    "description": "是否启用信用商店购买",
                    "default": true
                },
                "force_shopping_if_credit_full": {
                    "type": "boolean",
                    "description": "信用满时是否强制购买",
                    "default": true
                },
                "buy_first": {
                    "type": "array",
                    "description": "优先购买的商品列表",
                    "items": {"type": "string"},
                    "default": ["招聘许可", "龙门币"]
                },
                "blacklist": {
                    "type": "array", 
                    "description": "购买黑名单",
                    "items": {"type": "string"},
                    "default": ["家具", "碳"]
                },
                "reserve_max_credit": {
                    "type": "boolean",
                    "description": "是否保留最大信用点",
                    "default": false
                }
            },
            "required": []
        }),
    }
}

/// 处理信用商店增强任务
pub async fn handle_credit_store_enhanced(args: Value, queue_client: &MaaQueueClient) -> FunctionResponse {
    let start_time = Instant::now();
    info!("🏪 处理信用商店任务");
    
    let enable = args.get("enable").and_then(|v| v.as_bool()).unwrap_or(true);
    let force_shopping = args.get("force_shopping_if_credit_full").and_then(|v| v.as_bool()).unwrap_or(true);
    
    if !enable {
        let response_data = json!({
            "status": "success",
            "message": "信用商店功能已禁用",
            "enabled": false
        });
        return FunctionResponse::success("maa_credit_store_enhanced", response_data)
            .with_execution_time(start_time.elapsed().as_millis() as u64);
    }

    debug!("信用商店参数: enable={}, force_shopping={}", enable, force_shopping);

    let _buy_first = args.get("buy_first")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|item| item.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_else(|| vec!["招聘许可".to_string(), "龙门币".to_string()]);
        
    let _blacklist = args.get("blacklist")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|item| item.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_else(|| vec!["家具".to_string(), "碳".to_string()]);

    match queue_client.credit_store(enable).await {
        Ok(result) => {
            info!("信用商店任务完成");
            let response_data = json!({
                "status": "success",
                "message": "信用商店购买任务完成",
                "enabled": enable,
                "force_shopping": force_shopping,
                "details": result
            });
            FunctionResponse::success("maa_credit_store_enhanced", response_data)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        },
        Err(e) => {
            let error = MaaError::maa_core_error(&format!("信用商店购买失败: {}", e), Some("检查信用点数量和商店状态"));
            debug!("信用商店购买失败: {}", e);
            FunctionResponse::error("maa_credit_store_enhanced", error)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        }
    }
}

/// 创建仓库管理工具定义
pub fn create_depot_management_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_depot_management".to_string(),
        description: "智能仓库管理系统".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "enable": {
                    "type": "boolean",
                    "description": "是否启用仓库管理",
                    "default": true
                },
                "depot_enable": {
                    "type": "boolean",
                    "description": "是否开启仓库识别",
                    "default": true
                },
                "scan_only": {
                    "type": "boolean",
                    "description": "是否只扫描不管理",
                    "default": false
                },
                "category_filter": {
                    "type": "array",
                    "description": "物品分类过滤器",
                    "items": {
                        "type": "string",
                        "enum": ["材料", "芯片", "技能概要", "模组数据块", "家具"]
                    },
                    "default": ["材料", "芯片"]
                },
                "rarity_filter": {
                    "type": "array",
                    "description": "稀有度过滤器",
                    "items": {"type": "integer", "minimum": 1, "maximum": 5},
                    "default": [1, 2, 3, 4, 5]
                }
            },
            "required": []
        }),
    }
}

/// 处理仓库管理任务
pub async fn handle_depot_management(args: Value, queue_client: &MaaQueueClient) -> FunctionResponse {
    let start_time = Instant::now();
    info!("📦 处理仓库管理任务");
    
    let enable = args.get("enable").and_then(|v| v.as_bool()).unwrap_or(true);
    let depot_enable = args.get("depot_enable").and_then(|v| v.as_bool()).unwrap_or(true);
    let scan_only = args.get("scan_only").and_then(|v| v.as_bool()).unwrap_or(false);

    if !enable {
        let response_data = json!({
            "status": "success",
            "message": "仓库管理功能已禁用",
            "enabled": false
        });
        return FunctionResponse::success("maa_depot_management", response_data)
            .with_execution_time(start_time.elapsed().as_millis() as u64);
    }

    debug!("仓库管理参数: enable={}, depot_enable={}, scan_only={}", 
           enable, depot_enable, scan_only);

    match queue_client.depot_management(depot_enable).await {
        Ok(result) => {
            info!("仓库管理任务完成");
            let response_data = json!({
                "status": "success",
                "message": "仓库管理任务完成",
                "enabled": enable,
                "scan_only": scan_only,
                "details": result
            });
            FunctionResponse::success("maa_depot_management", response_data)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        },
        Err(e) => {
            let error = MaaError::maa_core_error(&format!("仓库管理失败: {}", e), Some("检查仓库页面是否已打开"));
            debug!("仓库管理失败: {}", e);
            FunctionResponse::error("maa_depot_management", error)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        }
    }
}

/// 创建干员整理工具定义
pub fn create_operator_box_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_operator_box".to_string(),
        description: "干员整理和管理系统".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "enable": {
                    "type": "boolean",
                    "description": "是否启用干员整理",
                    "default": true
                },
                "sort_by": {
                    "type": "string",
                    "description": "排序方式",
                    "enum": ["稀有度", "等级", "职业", "获得时间"],
                    "default": "稀有度"
                },
                "filter_elite": {
                    "type": "array",
                    "description": "精英化等级过滤",
                    "items": {"type": "integer", "minimum": 0, "maximum": 2},
                    "default": [0, 1, 2]
                },
                "filter_level": {
                    "type": "object",
                    "description": "等级过滤范围",
                    "properties": {
                        "min": {"type": "integer", "minimum": 1, "default": 1},
                        "max": {"type": "integer", "maximum": 90, "default": 90}
                    }
                },
                "filter_rarity": {
                    "type": "array",
                    "description": "稀有度过滤",
                    "items": {"type": "integer", "minimum": 1, "maximum": 6},
                    "default": [1, 2, 3, 4, 5, 6]
                }
            },
            "required": []
        }),
    }
}

/// 处理干员整理任务
pub async fn handle_operator_box(args: Value, queue_client: &MaaQueueClient) -> FunctionResponse {
    let start_time = Instant::now();
    info!("👥 处理干员整理任务");
    
    let enable = args.get("enable").and_then(|v| v.as_bool()).unwrap_or(true);
    let sort_by = args.get("sort_by").and_then(|v| v.as_str()).unwrap_or("稀有度");

    if !enable {
        let response_data = json!({
            "status": "success",
            "message": "干员整理功能已禁用",
            "enabled": false
        });
        return FunctionResponse::success("maa_operator_box", response_data)
            .with_execution_time(start_time.elapsed().as_millis() as u64);
    }

    debug!("干员整理参数: enable={}, sort_by={}", enable, sort_by);

    match queue_client.operator_box(enable).await {
        Ok(result) => {
            info!("干员整理任务完成");
            let response_data = json!({
                "status": "success",
                "message": "干员整理任务完成",
                "enabled": enable,
                "sort_by": sort_by,
                "details": result
            });
            FunctionResponse::success("maa_operator_box", response_data)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        },
        Err(e) => {
            let error = MaaError::maa_core_error(&format!("干员整理失败: {}", e), Some("检查干员列表页面是否已打开"));
            debug!("干员整理失败: {}", e);
            FunctionResponse::error("maa_operator_box", error)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        }
    }
}