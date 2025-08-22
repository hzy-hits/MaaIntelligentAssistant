//! 系统功能模块 - V2简化版本
//!
//! 包含8个系统MAA功能定义：
//! - maa_closedown: 关闭游戏
//! - maa_custom_task: 自定义任务 
//! - maa_video_recognition: 视频识别
//! - maa_system_management: 系统管理
//! - maa_take_screenshot: 截图功能
//! - maa_get_task_list: 获取任务列表
//! - maa_adjust_task_params: 动态调整任务参数
//! - maa_emergency_home: 紧急返回主界面

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

/// 创建获取任务列表工具定义
pub fn create_get_task_list_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_get_task_list".to_string(),
        description: "获取当前MAA运行中的任务列表和状态信息".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "refresh": {
                    "type": "boolean",
                    "description": "是否强制刷新任务列表",
                    "default": false
                },
                "include_completed": {
                    "type": "boolean",
                    "description": "是否包含已完成的任务",
                    "default": false
                }
            },
            "required": []
        }),
    }
}

/// 创建动态调整任务参数工具定义
pub fn create_adjust_task_params_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_adjust_task_params".to_string(),
        description: "动态调整运行中任务的参数，支持智能策略".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "task_id": {
                    "type": "integer",
                    "description": "要调整的任务ID"
                },
                "strategy": {
                    "type": "string",
                    "description": "调整策略: reduce_difficulty(降低难度), increase_efficiency(提高效率), emergency_stop(紧急停止)",
                    "enum": ["reduce_difficulty", "increase_efficiency", "emergency_stop", "custom"]
                },
                "custom_params": {
                    "type": "object",
                    "description": "自定义参数(当strategy=custom时使用)",
                    "properties": {
                        "medicine": {
                            "type": "integer",
                            "description": "理智药数量",
                            "minimum": 0
                        },
                        "times": {
                            "type": "integer", 
                            "description": "执行次数",
                            "minimum": 0
                        },
                        "enable": {
                            "type": "boolean",
                            "description": "是否启用任务"
                        }
                    }
                },
                "context": {
                    "type": "object",
                    "description": "调整上下文信息",
                    "properties": {
                        "available_medicine": {
                            "type": "integer",
                            "description": "可用理智药数量"
                        },
                        "reason": {
                            "type": "string",
                            "description": "调整原因"
                        }
                    }
                }
            },
            "required": ["task_id", "strategy"]
        }),
    }
}

/// 创建紧急返回主界面工具定义
pub fn create_emergency_home_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_emergency_home".to_string(),
        description: "紧急情况下快速返回游戏主界面，中断当前所有操作".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "reason": {
                    "type": "string",
                    "description": "紧急返回的原因",
                    "default": "user_request"
                },
                "force": {
                    "type": "boolean",
                    "description": "是否强制返回(跳过安全检查)",
                    "default": false
                },
                "stop_tasks": {
                    "type": "boolean",
                    "description": "是否同时停止所有运行中的任务",
                    "default": true
                }
            },
            "required": []
        }),
    }
}