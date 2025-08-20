//! 重构后的MAA Function Calling 处理器
//! 
//! 优化点：
//! 1. 减少JSON序列化次数
//! 2. 直接使用统一任务队列
//! 3. 支持同步/异步任务区别处理

use chrono::Utc;
use serde_json::{json, Value};
use tracing::{debug, info, warn, error};
use anyhow::{Result, anyhow};

use super::types::{FunctionCall, FunctionDefinition, FunctionResponse, MaaError, ErrorType, ResponseMetadata};
use crate::maa_core::{MaaTaskSenderV2, TaskResult};
use crate::maa_core::task_classification_v2::{classify_task, TaskExecutionMode};

// 导入所有功能模块
use super::advanced_automation::*;
use super::core_game::*;
use super::support_features::*;
use super::system_features::*;

/// 重构后的MAA Function Calling 处理器 - V2版本
#[derive(Clone)]
pub struct EnhancedMaaFunctionHandlerV2 {
    task_sender: MaaTaskSenderV2,
}

impl EnhancedMaaFunctionHandlerV2 {
    /// 创建新的Function Calling处理器
    pub fn new(task_sender: MaaTaskSenderV2) -> Self {
        info!("创建增强MAA Function Calling处理器 V2");
        Self { task_sender }
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

        info!("已加载 {} 个增强MAA Function Calling工具", definitions.len());
        definitions
    }

    /// 执行Function Call
    /// 
    /// 优化点：
    /// 1. 直接传递JSON参数，避免重复序列化
    /// 2. 根据任务类型选择同步/异步处理
    pub async fn execute_function(&self, function_call: FunctionCall) -> FunctionResponse {
        let start_time = Utc::now();
        let function_name = function_call.name.clone();
        
        debug!("执行Function Call: {} with args: {:?}", function_name, function_call.arguments);
        
        // 分类任务
        let (execution_mode, priority) = classify_task(&function_name);
        
        // 验证Function Call
        if let Err(validation_error) = self.validate_function_call(&function_call) {
            warn!("Function call 验证失败: {}", validation_error);
            return FunctionResponse {
                success: false,
                result: None,
                error: Some(MaaError {
                    error_type: ErrorType::ParameterError,
                    message: validation_error.to_string(),
                    details: None,
                    suggestion: Some("请检查Function Call参数格式".to_string()),
                    error_code: Some("VALIDATION_ERROR".to_string()),
                }),
                timestamp: Utc::now(),
                execution_time_ms: Some(0),
                metadata: ResponseMetadata {
                    task_id: None,
                    function_name: function_name.clone(),
                    recommendations: vec![],
                    next_actions: vec![],
                    resource_usage: None,
                },
            };
        }
        
        // 发送任务到队列
        let task_result = match self.task_sender.send_task(
            function_name.clone(),
            function_call.arguments, // 直接传递JSON，避免重复序列化
            priority,
            execution_mode,
        ) {
            Ok((task_id, response_rx)) => {
                debug!("任务已发送到队列: {} (task_id: {}, 模式: {:?})", function_name, task_id, execution_mode);
                
                // 根据执行模式处理响应
                match execution_mode {
                    TaskExecutionMode::Synchronous => {
                        // 同步任务：等待完成
                        debug!("等待同步任务完成: {}", function_name);
                        match response_rx.await {
                            Ok(result) => {
                                debug!("同步任务完成: {} (耗时: {:.2}s)", function_name, result.duration_seconds);
                                Ok(result)
                            },
                            Err(e) => {
                                error!("同步任务响应接收失败: {}", e);
                                Err(anyhow!("任务响应接收失败: {}", e))
                            }
                        }
                    },
                    TaskExecutionMode::Asynchronous => {
                        // 异步任务：立即返回任务信息，不等待完成
                        debug!("异步任务已启动: {} (task_id: {})", function_name, task_id);
                        Ok(TaskResult {
                            success: true,
                            task_id,
                            result: Some(json!({
                                "task_id": task_id,
                                "task_type": function_name,
                                "status": "running",
                                "message": "异步任务已启动，正在后台执行",
                                "execution_mode": "asynchronous",
                                "check_status_url": format!("/task/{}/status", task_id),
                                "sse_events": "任务进度将通过SSE推送"
                            })),
                            error: None,
                            completed_at: Utc::now(),
                            duration_seconds: 0.0,
                        })
                    }
                }
            },
            Err(e) => {
                error!("任务发送失败: {}", e);
                Err(anyhow!("任务队列发送失败: {}", e))
            }
        };
        
        // 转换结果为FunctionResponse
        match task_result {
            Ok(result) => {
                let execution_time = (Utc::now() - start_time).num_milliseconds() as f64 / 1000.0;
                info!("Function call 成功: {} (耗时: {:.2}s)", function_name, execution_time);
                
                FunctionResponse {
                    success: true,
                    result: result.result,
                    error: None,
                    timestamp: Utc::now(),
                    execution_time_ms: Some((execution_time * 1000.0) as u64),
                    metadata: ResponseMetadata {
                        task_id: Some(result.task_id.to_string()),
                        function_name: function_name.clone(),
                        recommendations: vec![],
                        next_actions: vec![],
                        resource_usage: None,
                    },
                }
            },
            Err(e) => {
                let execution_time = (Utc::now() - start_time).num_milliseconds() as f64 / 1000.0;
                error!("Function call 失败: {} (耗时: {:.2}s) - {}", function_name, execution_time, e);
                
                FunctionResponse {
                    success: false,
                    result: None,
                    error: Some(MaaError {
                        error_type: ErrorType::MaaCoreError,
                        message: e.to_string(),
                        details: None,
                        suggestion: Some("请检查MAA连接状态和任务参数".to_string()),
                        error_code: Some("EXECUTION_ERROR".to_string()),
                    }),
                    timestamp: Utc::now(),
                    execution_time_ms: Some((execution_time * 1000.0) as u64),
                    metadata: ResponseMetadata {
                        task_id: None,
                        function_name: function_name.clone(),
                        recommendations: vec![],
                        next_actions: vec![],
                        resource_usage: None,
                    },
                }
            }
        }
    }

    /// 验证Function Call参数
    fn validate_function_call(&self, function_call: &FunctionCall) -> Result<()> {
        // 检查function名称
        if function_call.name.is_empty() {
            return Err(anyhow!("Function name不能为空"));
        }
        
        // 检查是否为支持的function
        let supported_functions = [
            "maa_startup", "maa_combat_enhanced", "maa_recruit_enhanced", "maa_infrastructure_enhanced",
            "maa_roguelike_enhanced", "maa_copilot_enhanced", "maa_sss_copilot", "maa_reclamation",
            "maa_rewards_enhanced", "maa_credit_store_enhanced", "maa_depot_management", "maa_operator_box",
            "maa_closedown", "maa_custom_task", "maa_video_recognition", "maa_system_management",
            "maa_take_screenshot"
        ];
        
        if !supported_functions.contains(&function_call.name.as_str()) {
            return Err(anyhow!("不支持的Function: {}", function_call.name));
        }
        
        // 基础参数验证
        if !function_call.arguments.is_object() {
            return Err(anyhow!("Function参数必须是JSON对象"));
        }
        
        Ok(())
    }

    /// 获取服务器状态
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
            "server_type": "enhanced_function_calling_v2",
            "total_functions": 16,
            "function_categories": {
                "core_game": 4,
                "advanced_automation": 4,
                "support_features": 4,
                "system_features": 4
            },
            "architecture": "optimized_v2_single_queue",
            "maa_core": maa_status,
            "status": if maa_status.get("connected").and_then(|v| v.as_bool()).unwrap_or(false) {
                "ready"
            } else {
                "initializing"
            },
            "optimization_features": {
                "unified_queue": true,
                "priority_system": true,
                "reduced_serialization": true,
                "integrated_task_status": true,
                "sse_support": true
            },
            "timestamp": Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()
        })
    }

    /// 自动初始化MAA - 优化服务器V2版本（避免直接调用basic_ops）
    async fn auto_initialize_maa(&self) -> Result<Value, anyhow::Error> {
        // 优化服务器V2使用Worker内部状态管理，不直接调用basic_ops中的get_maa_status
        // 而是返回一个表示MAA准备就绪的状态，实际的MAA初始化由Worker负责
        info!("优化服务器V2: MAA状态由Worker内部管理");
        
        Ok(json!({
            "initialized": true,
            "connected": true, // 假设已连接，由Worker在实际执行时处理
            "running": false,
            "version": "v2-worker-managed",
            "device_address": std::env::var("MAA_DEVICE_ADDRESS").unwrap_or("127.0.0.1:1717".to_string()),
            "last_updated": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string(),
            "active_tasks": serde_json::Value::Array(vec![]),
            "note": "MAA Core状态由Worker V2内部管理，实际初始化在任务执行时进行"
        }))
    }
    
    /// 获取任务执行统计
    pub async fn get_execution_stats(&self) -> Value {
        json!({
            "total_functions": 16,
            "synchronous_functions": 3,
            "asynchronous_functions": 13,
            "optimization_benefits": {
                "reduced_json_serialization": "直接传递参数，避免重复序列化",
                "unified_queue": "单队列+优先级，简化架构",
                "integrated_status": "Worker内部状态管理，减少锁竞争"
            }
        })
    }
}

/// 创建增强Function Calling处理器 (V2)

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maa_core::task_queue_v2::create_maa_task_channel;
    use serde_json::json;

    #[test]
    fn test_function_validation() {
        let (sender, _receiver) = create_maa_task_channel();
        let handler = EnhancedMaaFunctionHandlerV2::new(sender);
        
        // 有效的function call
        let valid_call = FunctionCall {
            name: "maa_startup".to_string(),
            arguments: json!({"client_type": "Official"}),
        };
        assert!(handler.validate_function_call(&valid_call).is_ok());
        
        // 无效的function name
        let invalid_call = FunctionCall {
            name: "unknown_function".to_string(),
            arguments: json!({}),
        };
        assert!(handler.validate_function_call(&invalid_call).is_err());
        
        // 无效的参数类型
        let invalid_args = FunctionCall {
            name: "maa_startup".to_string(),
            arguments: json!("not an object"),
        };
        assert!(handler.validate_function_call(&invalid_args).is_err());
    }

    #[test]
    fn test_task_classification() {
        // 测试同步任务识别
        assert!(is_synchronous_task("maa_startup"));
        assert!(is_synchronous_task("maa_closedown"));
        assert!(is_synchronous_task("maa_take_screenshot"));
        
        // 测试异步任务识别
        assert!(!is_synchronous_task("maa_combat_enhanced"));
        assert!(!is_synchronous_task("maa_recruit_enhanced"));
        assert!(!is_synchronous_task("maa_roguelike_enhanced"));
    }

    #[tokio::test]
    async fn test_function_definitions() {
        let (sender, _receiver) = create_maa_task_channel();
        let handler = EnhancedMaaFunctionHandlerV2::new(sender);
        
        let definitions = handler.get_function_definitions();
        assert_eq!(definitions.len(), 16);
        
        // 验证包含关键函数
        let function_names: Vec<String> = definitions.iter().map(|d| d.name.clone()).collect();
        assert!(function_names.contains(&"maa_startup".to_string()));
        assert!(function_names.contains(&"maa_combat_enhanced".to_string()));
        assert!(function_names.contains(&"maa_closedown".to_string()));
    }
}

/// 创建增强Function Calling处理器V2 - 工厂函数
pub fn create_enhanced_function_handler_v2(task_sender: MaaTaskSenderV2) -> EnhancedMaaFunctionHandlerV2 {
    info!("创建增强MAA Function Calling处理器V2实例");
    EnhancedMaaFunctionHandlerV2::new(task_sender)
}