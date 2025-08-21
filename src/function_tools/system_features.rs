//! 系统功能模块 - V2简化版本
//!
//! 包含5个系统MAA功能定义：
//! - maa_closedown: 关闭游戏
//! - maa_custom_task: 自定义任务 
//! - maa_video_recognition: 视频识别
//! - maa_system_management: 系统管理
//! - maa_take_screenshot: 截图功能

use serde_json::json;
use super::types::FunctionDefinition;

/// 创建关闭游戏工具定义
pub fn create_closedown_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_closedown".to_string(),
        description: "关闭明日方舟游戏并清理资源".to_string(),
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
                    "description": "是否保存游戏状态",
                    "default": true
                }
            },
            "required": []
        }),
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
                "params": {
                    "type": "object",
                    "description": "任务参数JSON对象",
                    "default": {}
                }
            },
            "required": ["task_name"]
        }),
    }
}

/// 创建视频识别工具定义
pub fn create_video_recognition_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_video_recognition".to_string(),
        description: "对指定视频进行MAA识别分析".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "video_path": {
                    "type": "string",
                    "description": "视频文件路径"
                },
                "output_format": {
                    "type": "string",
                    "description": "输出格式 (json/text)",
                    "default": "json"
                }
            },
            "required": ["video_path"]
        }),
    }
}

/// 创建系统管理工具定义
pub fn create_system_management_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_system_management".to_string(),
        description: "MAA系统管理和状态控制".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "description": "操作类型: status(状态查询), restart(重启), stop(停止)",
                    "enum": ["status", "restart", "stop"],
                    "default": "status"
                }
            },
            "required": []
        }),
    }
}

/// 创建截图工具定义
pub fn create_screenshot_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_take_screenshot".to_string(),
        description: "获取当前游戏截图".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "format": {
                    "type": "string",
                    "description": "图片格式 (png/jpg)",
                    "default": "png"
                },
                "quality": {
                    "type": "integer",
                    "description": "图片质量 (1-100)",
                    "default": 90,
                    "minimum": 1,
                    "maximum": 100
                }
            },
            "required": []
        }),
    }
}