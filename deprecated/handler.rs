//! MAA Function Calling 处理器实现
//!
//! 整合所有17个MAA Function Calling工具的核心处理逻辑

use chrono::Utc;
use serde_json::{json, Value};
use tracing::{debug, info, warn};

use super::types::{FunctionCall, FunctionDefinition, FunctionResponse, MaaError};
use super::queue_client::MaaQueueClient;
use crate::maa_core::{MaaTaskSender, TaskExecutionMode, get_task_execution_mode, estimate_task_duration};

// 导入所有功能模块
use super::advanced_automation::*;
use super::core_game::*;
use super::support_features::*;
use super::system_features::*;

/// MAA Function Calling 处理器
///
/// 重构后的任务队列架构：通过消息队列与MAA工作线程通信
#[derive(Clone)]
pub struct EnhancedMaaFunctionHandler {
    queue_client: MaaQueueClient,
}

impl EnhancedMaaFunctionHandler {
    /// 创建新的Function Calling处理器
    pub fn new(task_sender: MaaTaskSender) -> Self {
        info!("创建增强MAA Function Calling处理器");
        let queue_client = MaaQueueClient::new(task_sender);
        Self { queue_client }
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

        // 系统功能 (5个)
        definitions.push(create_closedown_definition());
        definitions.push(create_custom_task_definition());
        definitions.push(create_video_recognition_definition());
        definitions.push(create_system_management_definition());
        definitions.push(create_screenshot_definition());

        info!("加载了 {} 个Function Calling工具", definitions.len());
        definitions
    }

    /// 执行Function Calling - 支持同步/异步模式
    pub async fn execute_function(&self, call: FunctionCall) -> FunctionResponse {
        let execution_mode = get_task_execution_mode(&call.name);
        let duration_estimate = estimate_task_duration(&call.name, &call.arguments);
        
        info!("执行Function Call: {} (模式: {:?}, 预计用时: {})", 
              call.name, execution_mode, duration_estimate);
              
        // 根据执行模式决定响应策略
        match execution_mode {
            TaskExecutionMode::Synchronous => {
                debug!("同步执行模式 - 等待任务完成");
                self.execute_sync_function(call).await
            }
            TaskExecutionMode::Asynchronous => {
                debug!("异步执行模式 - 立即返回任务ID");
                self.execute_async_function(call, duration_estimate).await
            }
        }
    }
    
    /// 执行同步任务（立即等待完成）
    async fn execute_sync_function(&self, call: FunctionCall) -> FunctionResponse {
        debug!("同步执行Function Call: {}", call.name);

        match call.name.as_str() {
            // 同步任务（快速响应）
            "maa_take_screenshot" => handle_screenshot(call.arguments, &self.queue_client).await,
            "maa_system_management" => handle_system_management(call.arguments, &self.queue_client).await,

            _ => {
                warn!("未知的同步Function Call: {}", call.name);
                let error = MaaError::parameter_error(
                    &format!("未知的同步函数调用: {}", call.name),
                    Some("请检查函数名称是否正确")
                );
                FunctionResponse::error(&call.name, error)
            }
        }
    }
    
    /// 执行异步任务（立即返回任务ID）
    async fn execute_async_function(&self, call: FunctionCall, duration_estimate: String) -> FunctionResponse {
        debug!("异步执行Function Call: {}", call.name);

        // 先执行任务启动，获取task_id，然后立即返回
        let start_time = std::time::Instant::now();
        
        match call.name.as_str() {
            // 核心游戏功能（异步）
            "maa_startup" => {
                match self.start_async_task("startup", call.arguments, &call.name).await {
                    Ok(task_id) => self.create_async_response(&call.name, task_id, "游戏启动", &duration_estimate, start_time),
                    Err(e) => FunctionResponse::error(&call.name, e).with_execution_time(start_time.elapsed().as_millis() as u64)
                }
            },
            "maa_combat_enhanced" => {
                match self.start_async_task("combat", call.arguments, &call.name).await {
                    Ok(task_id) => self.create_async_response(&call.name, task_id, "战斗任务", &duration_estimate, start_time),
                    Err(e) => FunctionResponse::error(&call.name, e).with_execution_time(start_time.elapsed().as_millis() as u64)
                }
            },
            "maa_recruit_enhanced" => {
                match self.start_async_task("recruit", call.arguments, &call.name).await {
                    Ok(task_id) => self.create_async_response(&call.name, task_id, "公开招募", &duration_estimate, start_time),
                    Err(e) => FunctionResponse::error(&call.name, e).with_execution_time(start_time.elapsed().as_millis() as u64)
                }
            },
            "maa_infrastructure_enhanced" => {
                match self.start_async_task("infrastructure", call.arguments, &call.name).await {
                    Ok(task_id) => self.create_async_response(&call.name, task_id, "基建管理", &duration_estimate, start_time),
                    Err(e) => FunctionResponse::error(&call.name, e).with_execution_time(start_time.elapsed().as_millis() as u64)
                }
            },
            
            // 高级自动化（异步）
            "maa_roguelike_enhanced" => {
                match self.start_async_task("roguelike", call.arguments, &call.name).await {
                    Ok(task_id) => self.create_async_response(&call.name, task_id, "集成战略", &duration_estimate, start_time),
                    Err(e) => FunctionResponse::error(&call.name, e).with_execution_time(start_time.elapsed().as_millis() as u64)
                }
            },
            "maa_copilot_enhanced" => {
                match self.start_async_task("copilot", call.arguments, &call.name).await {
                    Ok(task_id) => self.create_async_response(&call.name, task_id, "作业执行", &duration_estimate, start_time),
                    Err(e) => FunctionResponse::error(&call.name, e).with_execution_time(start_time.elapsed().as_millis() as u64)
                }
            },
            
            // 其他异步任务...
            _ => {
                // 对于未明确分类的任务，尝试作为异步任务处理
                warn!("未分类的异步Function Call，尝试通用处理: {}", call.name);
                match self.start_async_task("generic", call.arguments, &call.name).await {
                    Ok(task_id) => self.create_async_response(&call.name, task_id, "MAA任务", &duration_estimate, start_time),
                    Err(e) => FunctionResponse::error(&call.name, e).with_execution_time(start_time.elapsed().as_millis() as u64)
                }
            }
        }
    }
    
    /// 启动异步任务（通用方法）
    async fn start_async_task(&self, task_type: &str, arguments: Value, _function_name: &str) -> Result<i32, MaaError> {
        match task_type {
            "recruit" => {
                let max_times = arguments.get("max_times")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(1) as i32;
                let expedite = arguments.get("expedite")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let skip_robot = arguments.get("skip_robot")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                
                match self.queue_client.recruit(max_times, expedite, skip_robot).await {
                    Ok(result) => {
                        if let Some(task_id) = result.get("task_id").and_then(|v| v.as_i64()) {
                            Ok(task_id as i32)
                        } else {
                            Err(MaaError::maa_core_error("无法获取任务ID", Some("任务可能启动失败")))
                        }
                    },
                    Err(e) => Err(MaaError::maa_core_error(&format!("启动招募任务失败: {}", e), None))
                }
            },
            "combat" => {
                let stage = arguments.get("stage")
                    .and_then(|v| v.as_str())
                    .unwrap_or("1-7")
                    .to_string();
                let times = arguments.get("times")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(1) as i32;
                
                match self.queue_client.combat(stage, 0, 0, times).await {
                    Ok(result) => {
                        if let Some(task_id) = result.get("task_id").and_then(|v| v.as_i64()) {
                            Ok(task_id as i32)
                        } else {
                            Err(MaaError::maa_core_error("无法获取任务ID", Some("任务可能启动失败")))
                        }
                    },
                    Err(e) => Err(MaaError::maa_core_error(&format!("启动战斗任务失败: {}", e), None))
                }
            },
            "infrastructure" => {
                let facility = arguments.get("facility")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect::<Vec<_>>())
                    .unwrap_or_else(|| vec!["Mfg".to_string(), "Trade".to_string(), "Power".to_string()]);
                
                match self.queue_client.infrastructure(facility, "NotUse".to_string(), 0.3).await {
                    Ok(result) => {
                        if let Some(task_id) = result.get("task_id").and_then(|v| v.as_i64()) {
                            Ok(task_id as i32)
                        } else {
                            Err(MaaError::maa_core_error("无法获取任务ID", Some("任务可能启动失败")))
                        }
                    },
                    Err(e) => Err(MaaError::maa_core_error(&format!("启动基建任务失败: {}", e), None))
                }
            },
            "startup" => {
                let client_type = arguments.get("client_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Official")
                    .to_string();
                let start_app = arguments.get("start_app")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                    
                match self.queue_client.startup(client_type, start_app, false).await {
                    Ok(result) => {
                        if let Some(task_id) = result.get("task_id").and_then(|v| v.as_i64()) {
                            Ok(task_id as i32)
                        } else {
                            Err(MaaError::maa_core_error("无法获取任务ID", Some("任务可能启动失败")))
                        }
                    },
                    Err(e) => Err(MaaError::maa_core_error(&format!("启动游戏失败: {}", e), None))
                }
            },
            "roguelike" => {
                let theme = arguments.get("theme")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Phantom")
                    .to_string();
                    
                match self.queue_client.roguelike(theme, 0, 0).await {
                    Ok(result) => {
                        if let Some(task_id) = result.get("task_id").and_then(|v| v.as_i64()) {
                            Ok(task_id as i32)
                        } else {
                            Err(MaaError::maa_core_error("无法获取任务ID", Some("任务可能启动失败")))
                        }
                    },
                    Err(e) => Err(MaaError::maa_core_error(&format!("启动集成战略失败: {}", e), None))
                }
            },
            "copilot" => {
                let filename = arguments.get("filename")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                    
                match self.queue_client.copilot(filename, false).await {
                    Ok(result) => {
                        if let Some(task_id) = result.get("task_id").and_then(|v| v.as_i64()) {
                            Ok(task_id as i32)
                        } else {
                            Err(MaaError::maa_core_error("无法获取任务ID", Some("任务可能启动失败")))
                        }
                    },
                    Err(e) => Err(MaaError::maa_core_error(&format!("启动作业执行失败: {}", e), None))
                }
            },
            _ => {
                // 其他任务类型的通用处理
                warn!("不支持的异步任务类型: {}，尝试基本启动", task_type);
                Err(MaaError::parameter_error(&format!("不支持的异步任务类型: {}", task_type), Some("请检查任务类型是否正确")))
            }
        }
    }
    
    /// 创建异步任务响应
    fn create_async_response(
        &self, 
        function_name: &str, 
        task_id: i32, 
        task_description: &str, 
        duration_estimate: &str,
        start_time: std::time::Instant
    ) -> FunctionResponse {
        let response_data = json!({
            "status": "accepted",
            "task_id": task_id,
            "task_type": task_description,
            "message": format!("{}任务已提交，正在后台执行", task_description),
            "duration_estimate": duration_estimate,
            "check_status_url": format!("/task/{}/status", task_id),
            "execution_mode": "asynchronous",
            "timestamp": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()
        });
        
        info!("异步任务已启动: {} (task_id: {})", function_name, task_id);
        FunctionResponse::success(function_name, response_data)
            .with_execution_time(start_time.elapsed().as_millis() as u64)
    }

    /// 获取服务器状态并自动初始化MAA
    pub async fn get_server_status(&self) -> Value {
        // 自动初始化MAA设备连接
        let maa_status = match self.auto_initialize_maa().await {
            Ok(status) => status,
            Err(e) => {
                warn!("MAA自动初始化失败: {}", e);
                json!({
                    "initialized": false,
                    "connected": false,
                    "error": e.to_string()
                })
            }
        };
        
        json!({
            "server_type": "enhanced_function_calling",
            "total_functions": 17,
            "function_categories": {
                "core_game": 4,
                "advanced_automation": 4,
                "support_features": 4,
                "system_features": 5
            },
            "architecture": "simplified_3_layer",
            "maa_core": maa_status,
            "status": if maa_status.get("connected").and_then(|v| v.as_bool()).unwrap_or(false) {
                "ready"
            } else {
                "initializing"
            },
            "timestamp": Utc::now()
        })
    }
    
    /// 自动初始化MAA设备连接
    async fn auto_initialize_maa(&self) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        use crate::config::CONFIG;
        
        // 获取当前MAA状态
        let current_status = self.queue_client.get_status().await?;
        
        // 如果已经连接，直接返回状态
        if current_status.get("connected").and_then(|v| v.as_bool()).unwrap_or(false) {
            info!("MAA已连接，跳过初始化");
            return Ok(current_status);
        }
        
        info!("开始自动初始化MAA设备连接...");
        
        // 尝试连接设备（优先使用PlayCover地址）
        let device_address = &CONFIG.device.playcover_address;
        info!("尝试连接设备: {}", device_address);
        
        match self.queue_client.connect(device_address.clone()).await {
            Ok(_) => {
                info!("MAA设备连接成功");
                // 重新获取状态
                let updated_status = self.queue_client.get_status().await?;
                Ok(updated_status)
            },
            Err(e) => {
                warn!("MAA设备连接失败: {}", e);
                Ok(json!({
                    "initialized": true,
                    "connected": false,
                    "device_address": device_address,
                    "error": format!("连接失败: {}", e),
                    "last_updated": chrono::Utc::now()
                }))
            }
        }
    }
}

/// 创建增强Function Calling处理器实例
pub fn create_enhanced_function_handler(task_sender: MaaTaskSender) -> EnhancedMaaFunctionHandler {
    info!("创建增强MAA Function Calling处理器实例");
    EnhancedMaaFunctionHandler::new(task_sender)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handler_creation() {
        let (task_sender, _) = crate::maa_core::create_maa_task_channel();
        let handler = create_enhanced_function_handler(task_sender);
        let definitions = handler.get_function_definitions();
        assert_eq!(definitions.len(), 17);
    }

    #[tokio::test]
    async fn test_function_definitions() {
        let (task_sender, _) = crate::maa_core::create_maa_task_channel();
        let handler = create_enhanced_function_handler(task_sender);
        let definitions = handler.get_function_definitions();

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
        assert_eq!(system_features, 5);
    }

    #[tokio::test]
    async fn test_startup_function_call() {
        let (task_sender, _) = crate::maa_core::create_maa_task_channel();
        let handler = create_enhanced_function_handler(task_sender);
        let call = FunctionCall {
            name: "maa_startup".to_string(),
            arguments: json!({
                "client_type": "Official",
                "start_app": true
            }),
        };

        let response = handler.execute_function(call).await;
        assert!(response.success);
        assert!(response.result.is_some());
    }

    #[tokio::test]
    async fn test_unknown_function_call() {
        let (task_sender, _) = crate::maa_core::create_maa_task_channel();
        let handler = create_enhanced_function_handler(task_sender);
        let call = FunctionCall {
            name: "unknown_function".to_string(),
            arguments: json!({}),
        };

        let response = handler.execute_function(call).await;
        assert!(!response.success);
        assert!(response.error.is_some());
    }

    #[tokio::test]
    async fn test_server_status() {
        let (task_sender, _) = crate::maa_core::create_maa_task_channel();
        let handler = create_enhanced_function_handler(task_sender);
        let status = handler.get_server_status().await;

        assert_eq!(status["total_functions"], 17);
        assert_eq!(status["server_type"], "enhanced_function_calling");
    }
}
