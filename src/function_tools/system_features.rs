//! 系统功能模块
//!
//! 包含4个系统MAA功能：
//! - maa_closedown: 关闭游戏
//! - maa_custom_task: 自定义任务 
//! - maa_video_recognition: 视频识别
//! - maa_system_management: 系统管理

use serde_json::{json, Value};
use tracing::{debug, info};

use super::types::{FunctionDefinition, FunctionResponse, MaaError};
use super::queue_client::MaaQueueClient;
use std::time::Instant;

/// 创建关闭游戏工具定义
pub fn create_closedown_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_closedown".to_string(),
        description: "安全关闭游戏和MAA系统".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "enable": {
                    "type": "boolean",
                    "description": "是否启用关闭功能",
                    "default": true
                },
                "force": {
                    "type": "boolean",
                    "description": "是否强制关闭",
                    "default": false
                },
                "save_state": {
                    "type": "boolean",
                    "description": "是否保存当前状态",
                    "default": true
                },
                "timeout": {
                    "type": "integer",
                    "description": "关闭超时时间（秒）",
                    "minimum": 1,
                    "maximum": 300,
                    "default": 30
                }
            },
            "required": []
        }),
    }
}

/// 处理关闭游戏任务
pub async fn handle_closedown(args: Value, queue_client: &MaaQueueClient) -> FunctionResponse {
    let start_time = Instant::now();
    info!("🔴 处理游戏关闭任务");
    
    let enable = args.get("enable").and_then(|v| v.as_bool()).unwrap_or(true);
    let force = args.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
    let save_state = args.get("save_state").and_then(|v| v.as_bool()).unwrap_or(true);
    let timeout = args.get("timeout").and_then(|v| v.as_i64()).unwrap_or(30) as i32;

    if !enable {
        let response_data = json!({
            "status": "success",
            "message": "关闭功能已禁用",
            "enabled": false
        });
        return FunctionResponse::success("maa_closedown", response_data)
            .with_execution_time(start_time.elapsed().as_millis() as u64);
    }

    debug!("关闭参数: enable={}, force={}, save_state={}, timeout={}", 
           enable, force, save_state, timeout);

    match queue_client.closedown().await {
        Ok(result) => {
            info!("游戏关闭任务完成");
            let response_data = json!({
                "status": "success",
                "message": "游戏已安全关闭",
                "enabled": enable,
                "force": force,
                "save_state": save_state,
                "timeout": timeout,
                "details": result
            });
            FunctionResponse::success("maa_closedown", response_data)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        },
        Err(e) => {
            let error = MaaError::maa_core_error(&format!("关闭任务失败: {}", e), Some("检查游戏状态和连接"));
            debug!("关闭任务失败: {}", e);
            FunctionResponse::error("maa_closedown", error)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        }
    }
}

/// 创建自定义任务工具定义
pub fn create_custom_task_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_custom_task".to_string(),
        description: "执行自定义MAA任务".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "task_name": {
                    "type": "string",
                    "description": "自定义任务名称"
                },
                "task_params": {
                    "type": "object",
                    "description": "任务参数",
                    "properties": {},
                    "additionalProperties": true
                },
                "entry": {
                    "type": "string",
                    "description": "任务入口点",
                    "default": "StartUp"
                },
                "pipeline_override": {
                    "type": "object",
                    "description": "管线覆盖配置"
                },
                "timeout": {
                    "type": "integer",
                    "description": "任务超时时间（秒）", 
                    "minimum": 1,
                    "maximum": 3600,
                    "default": 300
                }
            },
            "required": ["task_name"]
        }),
    }
}

/// 处理自定义任务
pub async fn handle_custom_task(args: Value, queue_client: &MaaQueueClient) -> FunctionResponse {
    let start_time = Instant::now();
    info!("处理自定义任务");
    
    let task_name = match args.get("task_name").and_then(|v| v.as_str()) {
        Some(name) => name,
        None => {
            let error = MaaError::validation_error("缺少必要参数: task_name", Some("请提供有效的任务名称"));
            return FunctionResponse::error("maa_custom_task", error)
                .with_execution_time(start_time.elapsed().as_millis() as u64);
        }
    };
        
    let entry = args.get("entry")
        .and_then(|v| v.as_str())
        .unwrap_or("StartUp");
        
    let timeout = args.get("timeout")
        .and_then(|v| v.as_i64())
        .unwrap_or(300) as i32;

    debug!("自定义任务参数: task_name={}, entry={}, timeout={}", 
           task_name, entry, timeout);

    match queue_client.custom_task(task_name.to_string(), "{}".to_string()).await {
        Ok(result) => {
            info!("自定义任务完成: {}", task_name);
            let response_data = json!({
                "status": "success",
                "message": format!("自定义任务 {} 执行完成", task_name),
                "task_name": task_name,
                "entry": entry,
                "timeout": timeout,
                "details": result
            });
            FunctionResponse::success("maa_custom_task", response_data)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        },
        Err(e) => {
            let error = MaaError::maa_core_error(&format!("自定义任务失败: {}", e), Some("检查任务名称和参数"));
            debug!("自定义任务失败: {}", e);
            FunctionResponse::error("maa_custom_task", error)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        }
    }
}

/// 创建视频识别工具定义
pub fn create_video_recognition_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_video_recognition".to_string(),
        description: "视频识别和分析系统".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "video_path": {
                    "type": "string",
                    "description": "视频文件路径"
                },
                "recognition_type": {
                    "type": "string",
                    "description": "识别类型",
                    "enum": ["battle", "gacha", "infrastructure", "general"],
                    "default": "general"
                },
                "output_format": {
                    "type": "string",
                    "description": "输出格式",
                    "enum": ["json", "xml", "text"],
                    "default": "json"
                },
                "enable_ocr": {
                    "type": "boolean",
                    "description": "是否启用OCR文字识别",
                    "default": true
                },
                "frame_interval": {
                    "type": "integer",
                    "description": "帧间隔（毫秒）",
                    "minimum": 100,
                    "maximum": 10000,
                    "default": 1000
                }
            },
            "required": ["video_path"]
        }),
    }
}

/// 处理视频识别任务
pub async fn handle_video_recognition(args: Value, queue_client: &MaaQueueClient) -> FunctionResponse {
    let start_time = Instant::now();
    info!("🎥 处理视频识别任务");
    
    let video_path = match args.get("video_path").and_then(|v| v.as_str()) {
        Some(path) => path,
        None => {
            let error = MaaError::validation_error("缺少必要参数: video_path", Some("请提供有效的视频文件路径"));
            return FunctionResponse::error("maa_video_recognition", error)
                .with_execution_time(start_time.elapsed().as_millis() as u64);
        }
    };
        
    let recognition_type = args.get("recognition_type")
        .and_then(|v| v.as_str())
        .unwrap_or("general");
        
    let enable_ocr = args.get("enable_ocr")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
        
    let _frame_interval = args.get("frame_interval")
        .and_then(|v| v.as_i64())
        .unwrap_or(1000) as i32;

    debug!("视频识别参数: video_path={}, recognition_type={}, enable_ocr={}", 
           video_path, recognition_type, enable_ocr);

    match queue_client.video_recognition(video_path.to_string()).await {
        Ok(result) => {
            info!("视频识别任务完成: {}", video_path);
            let response_data = json!({
                "status": "success",
                "message": format!("视频 {} 识别完成", video_path),
                "video_path": video_path,
                "recognition_type": recognition_type,
                "enable_ocr": enable_ocr,
                "details": result
            });
            FunctionResponse::success("maa_video_recognition", response_data)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        },
        Err(e) => {
            let error = MaaError::maa_core_error(&format!("视频识别失败: {}", e), Some("检查视频文件路径和格式"));
            debug!("视频识别失败: {}", e);
            FunctionResponse::error("maa_video_recognition", error)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        }
    }
}

/// 创建系统管理工具定义
pub fn create_system_management_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_system_management".to_string(),
        description: "MAA系统管理和维护".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "description": "系统管理操作",
                    "enum": ["status", "restart", "update", "clean", "backup", "restore"],
                    "default": "status"
                },
                "component": {
                    "type": "string",
                    "description": "目标组件",
                    "enum": ["core", "resource", "config", "cache", "log", "all"],
                    "default": "all"
                },
                "backup_path": {
                    "type": "string",
                    "description": "备份路径（backup/restore操作时使用）"
                },
                "force": {
                    "type": "boolean",
                    "description": "是否强制执行操作",
                    "default": false
                }
            },
            "required": []
        }),
    }
}

/// 处理系统管理任务
pub async fn handle_system_management(args: Value, queue_client: &MaaQueueClient) -> FunctionResponse {
    let start_time = Instant::now();
    info!("⚙️ 处理系统管理任务");
    
    let action = args.get("action")
        .and_then(|v| v.as_str())
        .unwrap_or("status");
        
    let component = args.get("component")
        .and_then(|v| v.as_str())
        .unwrap_or("all");
        
    let force = args.get("force")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    debug!("系统管理参数: action={}, component={}, force={}", 
           action, component, force);

    let _params = json!({
        "action": action,
        "component": component,
        "force": force
    });

    match queue_client.system_management(action.to_string()).await {
        Ok(result) => {
            info!("系统管理任务完成: {} -> {}", action, component);
            let response_data = json!({
                "status": "success",
                "message": format!("系统管理操作 {} 完成", action),
                "action": action,
                "component": component,
                "force": force,
                "details": result
            });
            FunctionResponse::success("maa_system_management", response_data)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        },
        Err(e) => {
            let error = MaaError::maa_core_error(&format!("系统管理操作失败: {}", e), Some("检查系统状态和权限"));
            debug!("系统管理操作失败: {}", e);
            FunctionResponse::error("maa_system_management", error)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        }
    }
}

/// 创建截图工具定义
pub fn create_screenshot_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_take_screenshot".to_string(),
        description: "拍摄MAA当前游戏画面截图，展示牛牛眼中的世界".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "save_to_disk": {
                    "type": "boolean",
                    "description": "是否保存截图到磁盘",
                    "default": true
                },
                "include_base64": {
                    "type": "boolean",
                    "description": "是否在响应中包含base64编码的图片数据",
                    "default": true
                }
            },
            "required": []
        }),
    }
}

/// 处理截图任务 (简化版本)
pub async fn handle_screenshot(args: Value, queue_client: &crate::function_tools::queue_client::MaaQueueClient) -> FunctionResponse {
    let start_time = Instant::now();
    info!("处理截图任务");
    
    let include_base64 = args.get("include_base64")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    match queue_client.take_screenshot().await {
        Ok(image_data) => {
            info!("获取到MAA截图数据: {} bytes", image_data.len());
            
            use crate::maa_core::screenshot;
            match screenshot::save_maa_screenshot(image_data) {
                Ok(screenshot_info) => {
                    let mut response_data = json!({
                        "status": "success",
                        "message": "截图完成",
                        "screenshot_id": screenshot_info.id,
                        "timestamp": screenshot_info.timestamp,
                        "file_size": screenshot_info.file_size
                    });
                    
                    if include_base64 {
                        if let Some(thumbnail_base64) = screenshot_info.thumbnail_base64 {
                            response_data["base64_data"] = json!(thumbnail_base64);
                        }
                    }
                    
                    FunctionResponse::success("maa_take_screenshot", response_data)
                        .with_execution_time(start_time.elapsed().as_millis() as u64)
                },
                Err(e) => {
                    let error = MaaError::maa_core_error(&format!("保存截图失败: {}", e), Some("检查磁盘空间和权限"));
                    FunctionResponse::error("maa_take_screenshot", error)
                        .with_execution_time(start_time.elapsed().as_millis() as u64)
                }
            }
        },
        Err(e) => {
            let error = MaaError::maa_core_error(&format!("MAA截图失败: {}", e), Some("检查MAA连接状态和游戏窗口"));
            FunctionResponse::error("maa_take_screenshot", error)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        }
    }
}