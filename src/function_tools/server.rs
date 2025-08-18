//! MAA Function Calling 服务器实现
//!
//! 整合所有16个MAA Function Calling工具的服务器

use chrono::Utc;
use serde_json::{json, Value};
use tracing::{debug, info, warn};

use super::types::{FunctionCall, FunctionDefinition, FunctionResponse};

// 导入所有功能模块
use super::advanced_automation::*;
use super::core_game::*;
use super::support_features::*;
use super::system_features::*;

/// 增强的MAA Function Calling服务器
///
/// 重构后的简化架构：直接调用MaaCore，无复杂抽象层
#[derive(Clone)]
pub struct EnhancedMaaFunctionServer {
    // 简化：直接使用MaaCore单例，不需要字段
}

impl EnhancedMaaFunctionServer {
    /// 创建新的Function Calling服务器
    pub fn new() -> Self {
        info!("🚀 创建增强MAA Function Calling服务器");
        Self {}
    }

    /// 获取所有Function Calling工具定义
    pub fn get_function_definitions(&self) -> Vec<FunctionDefinition> {
        let mut definitions = Vec::new();

        // 核心游戏功能 (4个)
        definitions.push(create_startup_definition());
        definitions.push(create_combat_enhanced_definition());
        definitions.push(create_recruit_enhanced_definition());
        definitions.push(create_infrastructure_enhanced_definition());

        // 高级自动化 (4个)
        definitions.push(create_roguelike_enhanced_definition());
        definitions.push(create_copilot_enhanced_definition());
        definitions.push(create_sss_copilot_definition());
        definitions.push(create_reclamation_definition());

        // 辅助功能 (4个)
        definitions.push(create_rewards_enhanced_definition());
        definitions.push(create_credit_store_enhanced_definition());
        definitions.push(create_depot_management_definition());
        definitions.push(create_operator_box_definition());

        // 系统功能 (4个)
        definitions.push(create_closedown_definition());
        definitions.push(create_custom_task_definition());
        definitions.push(create_video_recognition_definition());
        definitions.push(create_system_management_definition());

        info!("📋 加载了 {} 个Function Calling工具", definitions.len());
        definitions
    }

    /// 执行Function Calling
    pub async fn execute_function(&self, call: FunctionCall) -> FunctionResponse {
        debug!("🎯 执行Function Call: {}", call.name);

        let result = match call.name.as_str() {
            // 核心游戏功能
            "maa_startup" => handle_startup(call.arguments).await,
            "maa_combat_enhanced" => handle_combat_enhanced(call.arguments).await,
            "maa_recruit_enhanced" => handle_recruit_enhanced(call.arguments).await,
            "maa_infrastructure_enhanced" => handle_infrastructure_enhanced(call.arguments).await,

            // 高级自动化
            "maa_roguelike_enhanced" => handle_roguelike_enhanced(call.arguments).await,
            "maa_copilot_enhanced" => handle_copilot_enhanced(call.arguments).await,
            "maa_sss_copilot" => handle_sss_copilot(call.arguments).await,
            "maa_reclamation" => handle_reclamation(call.arguments).await,

            // 辅助功能
            "maa_rewards_enhanced" => handle_rewards_enhanced(call.arguments).await,
            "maa_credit_store_enhanced" => handle_credit_store_enhanced(call.arguments).await,
            "maa_depot_management" => handle_depot_management(call.arguments).await,
            "maa_operator_box" => handle_operator_box(call.arguments).await,

            // 系统功能
            "maa_closedown" => handle_closedown(call.arguments).await,
            "maa_custom_task" => handle_custom_task(call.arguments).await,
            "maa_video_recognition" => handle_video_recognition(call.arguments).await,
            "maa_system_management" => handle_system_management(call.arguments).await,

            _ => {
                warn!("❌ 未知的Function Call: {}", call.name);
                Err(format!("未知的函数调用: {}", call.name))
            }
        };

        match result {
            Ok(value) => {
                debug!("✅ Function Call 成功: {}", call.name);
                FunctionResponse {
                    success: true,
                    result: Some(value),
                    error: None,
                    timestamp: Utc::now(),
                }
            }
            Err(error) => {
                warn!("❌ Function Call 失败: {} - {}", call.name, error);
                FunctionResponse {
                    success: false,
                    result: None,
                    error: Some(error),
                    timestamp: Utc::now(),
                }
            }
        }
    }

    /// 获取服务器状态
    pub async fn get_server_status(&self) -> Value {
        json!({
            "server_type": "enhanced_function_calling",
            "total_functions": 16,
            "function_categories": {
                "core_game": 4,
                "advanced_automation": 4,
                "support_features": 4,
                "system_features": 4
            },
            "architecture": "simplified_3_layer",
            "maa_core": "singleton_ready",
            "status": "healthy",
            "timestamp": Utc::now()
        })
    }
}

/// 创建增强Function Calling服务器实例
pub fn create_enhanced_function_server() -> EnhancedMaaFunctionServer {
    info!("🎯 创建增强MAA Function Calling服务器实例");
    EnhancedMaaFunctionServer::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        let server = create_enhanced_function_server();
        let definitions = server.get_function_definitions();
        assert_eq!(definitions.len(), 16);
    }

    #[tokio::test]
    async fn test_function_definitions() {
        let server = create_enhanced_function_server();
        let definitions = server.get_function_definitions();

        // 检查每个类别的工具数量
        let core_game = definitions
            .iter()
            .filter(|d| {
                d.name.starts_with("maa_startup")
                    || d.name.starts_with("maa_combat_enhanced")
                    || d.name.starts_with("maa_recruit_enhanced")
                    || d.name.starts_with("maa_infrastructure_enhanced")
            })
            .count();
        assert_eq!(core_game, 4);

        let advanced_automation = definitions
            .iter()
            .filter(|d| {
                d.name.starts_with("maa_roguelike_enhanced")
                    || d.name.starts_with("maa_copilot_enhanced")
                    || d.name.starts_with("maa_sss_copilot")
                    || d.name.starts_with("maa_reclamation")
            })
            .count();
        assert_eq!(advanced_automation, 4);

        let support_features = definitions
            .iter()
            .filter(|d| {
                d.name.starts_with("maa_rewards_enhanced")
                    || d.name.starts_with("maa_credit_store_enhanced")
                    || d.name.starts_with("maa_depot_management")
                    || d.name.starts_with("maa_operator_box")
            })
            .count();
        assert_eq!(support_features, 4);

        let system_features = definitions
            .iter()
            .filter(|d| {
                d.name.starts_with("maa_closedown")
                    || d.name.starts_with("maa_custom_task")
                    || d.name.starts_with("maa_video_recognition")
                    || d.name.starts_with("maa_system_management")
            })
            .count();
        assert_eq!(system_features, 4);
    }

    #[tokio::test]
    async fn test_startup_function_call() {
        let server = create_enhanced_function_server();
        let call = FunctionCall {
            name: "maa_startup".to_string(),
            arguments: json!({
                "client_type": "Official",
                "start_app": true
            }),
        };

        let response = server.execute_function(call).await;
        assert!(response.success);
        assert!(response.result.is_some());
    }

    #[tokio::test]
    async fn test_unknown_function_call() {
        let server = create_enhanced_function_server();
        let call = FunctionCall {
            name: "unknown_function".to_string(),
            arguments: json!({}),
        };

        let response = server.execute_function(call).await;
        assert!(!response.success);
        assert!(response.error.is_some());
    }

    #[tokio::test]
    async fn test_server_status() {
        let server = create_enhanced_function_server();
        let status = server.get_server_status().await;

        assert_eq!(status["total_functions"], 16);
        assert_eq!(status["server_type"], "enhanced_function_calling");
        assert_eq!(status["status"], "healthy");
    }
}
