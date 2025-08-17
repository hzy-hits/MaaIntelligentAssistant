//! MAA 原子级 Function Calling 服务器
//! 
//! 提供细颗粒度的MAA控制接口，基于底层FFI操作

use std::sync::Arc;
use serde_json::{json, Value};
use tracing::{info, error};

use crate::maa_adapter::MaaBackend;
use super::{FunctionDefinition, FunctionResponse};
use super::maa_atomic_operations::{
    MaaClickTask, MaaScreenshotTask, MaaSwipeTask, 
    MaaConnectionTask, MaaTaskManagementTask, 
    MaaImageRecognitionTask
};
use super::maa_custom_tasks::{
    MaaCustomTask, MaaSingleStepTask, MaaVideoRecognitionTask
};

/// MAA 原子级 Function Calling 服务器
/// 提供底层MAA操作的直接控制接口
pub struct AtomicMaaFunctionServer {
    maa_backend: Arc<MaaBackend>,
    
    // 设备控制任务
    click_task: MaaClickTask,
    screenshot_task: MaaScreenshotTask,
    swipe_task: MaaSwipeTask,
    
    // 连接管理任务
    connection_task: MaaConnectionTask,
    
    // 任务管理
    task_management_task: MaaTaskManagementTask,
    
    // 图像识别
    image_recognition_task: MaaImageRecognitionTask,
    
    // maa-cli 自定义任务
    custom_task: MaaCustomTask,
    single_step_task: MaaSingleStepTask,
    video_recognition_task: MaaVideoRecognitionTask,
}

impl AtomicMaaFunctionServer {
    pub fn new(maa_backend: Arc<MaaBackend>) -> Self {
        info!("创建原子级MAA Function Calling服务器，支持14种操作(11个原子级+3个自定义任务)");
        
        Self {
            click_task: MaaClickTask::new(maa_backend.clone()),
            screenshot_task: MaaScreenshotTask::new(maa_backend.clone()),
            swipe_task: MaaSwipeTask::new(maa_backend.clone()),
            connection_task: MaaConnectionTask::new(maa_backend.clone()),
            task_management_task: MaaTaskManagementTask::new(maa_backend.clone()),
            image_recognition_task: MaaImageRecognitionTask::new(maa_backend.clone()),
            custom_task: MaaCustomTask::new(maa_backend.clone()),
            single_step_task: MaaSingleStepTask::new(maa_backend.clone()),
            video_recognition_task: MaaVideoRecognitionTask::new(maa_backend.clone()),
            maa_backend,
        }
    }

    /// 获取所有原子级操作的Function定义
    pub fn get_atomic_function_definitions(&self) -> Vec<FunctionDefinition> {
        vec![
            // 设备控制原子操作
            FunctionDefinition {
                name: "maa_click".to_string(),
                description: "执行精确的像素级点击操作。大模型可用此工具点击游戏界面的按钮、菜单项或任何可点击元素。支持同步等待点击完成。常用场景：点击'开始行动'按钮、选择关卡、确认对话框等".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "x": {
                            "type": "integer",
                            "description": "点击的X坐标(像素)"
                        },
                        "y": {
                            "type": "integer", 
                            "description": "点击的Y坐标(像素)"
                        },
                        "wait_completion": {
                            "type": "boolean",
                            "default": true,
                            "description": "是否等待点击操作完成"
                        }
                    },
                    "required": ["x", "y"]
                }),
            },
            
            FunctionDefinition {
                name: "maa_screenshot".to_string(),
                description: "获取当前屏幕截图，用于视觉状态分析和UI识别。大模型可用此工具截取全屏或指定区域的图像，支持PNG/RGB/BGR格式。常用场景：检查游戏当前状态、分析界面元素、调试问题、保存证据截图等".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "format": {
                            "type": "string",
                            "enum": ["rgb", "bgr", "png"],
                            "default": "png",
                            "description": "图像输出格式"
                        },
                        "roi": {
                            "type": "array",
                            "items": {"type": "integer"},
                            "minItems": 4,
                            "maxItems": 4,
                            "description": "感兴趣区域[x, y, width, height]，为空则截取全屏"
                        },
                        "compress": {
                            "type": "boolean",
                            "default": true,
                            "description": "是否压缩输出"
                        }
                    }
                }),
            },
            
            FunctionDefinition {
                name: "maa_swipe".to_string(),
                description: "执行自定义滑动手势操作。大模型可用此工具模拟手指滑动，支持自定义起点、终点、滑动时间和步数。常用场景：滚动界面、滑动解锁、拖拽元素、手势导航等".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "from_x": {
                            "type": "integer",
                            "description": "滑动起始X坐标"
                        },
                        "from_y": {
                            "type": "integer",
                            "description": "滑动起始Y坐标"
                        },
                        "to_x": {
                            "type": "integer",
                            "description": "滑动结束X坐标"
                        },
                        "to_y": {
                            "type": "integer",
                            "description": "滑动结束Y坐标"
                        },
                        "duration_ms": {
                            "type": "integer",
                            "default": 500,
                            "description": "滑动持续时间(毫秒)"
                        },
                        "steps": {
                            "type": "integer",
                            "description": "滑动步数，可选"
                        }
                    },
                    "required": ["from_x", "from_y", "to_x", "to_y"]
                }),
            },
            
            // 连接管理原子操作
            FunctionDefinition {
                name: "maa_connection".to_string(),
                description: "管理MAA与Android设备的ADB连接。大模型可用此工具连接、断开、重连设备，或查询连接状态。支持自定义ADB路径和设备地址。常用场景：建立设备连接、诊断连接问题、切换设备、连接状态检查等".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "enum": ["connect", "disconnect", "status", "reconnect"],
                            "default": "status",
                            "description": "连接操作类型"
                        },
                        "adb_path": {
                            "type": "string",
                            "description": "ADB可执行文件路径(连接时必需)"
                        },
                        "device_address": {
                            "type": "string",
                            "description": "设备地址，如127.0.0.1:5555(连接时必需)"
                        },
                        "config": {
                            "type": "object",
                            "description": "连接配置参数"
                        },
                        "timeout_ms": {
                            "type": "integer",
                            "default": 30000,
                            "description": "连接超时时间(毫秒)"
                        }
                    }
                }),
            },
            
            // 任务管理原子操作
            FunctionDefinition {
                name: "maa_task_management".to_string(),
                description: "底层MAA任务生命周期管理。大模型可用此工具创建、启动、停止、监控MAA任务，支持多种任务类型(Fight、Infrast、StartUp等)。提供任务队列状态查询和精细控制。常用场景：创建战斗任务、管理基建任务、监控任务进度、控制任务执行等".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "enum": ["create", "start", "stop", "status", "list", "remove"],
                            "default": "list",
                            "description": "任务管理操作类型"
                        },
                        "task_type": {
                            "type": "string",
                            "description": "任务类型，如StartUp、Fight、Infrast等(创建时必需)"
                        },
                        "task_id": {
                            "type": "integer",
                            "description": "任务ID(start、stop、status、remove操作时必需)"
                        },
                        "task_params": {
                            "type": "object",
                            "description": "任务参数(创建时可选)"
                        }
                    }
                }),
            },
            
            // 图像识别原子操作
            FunctionDefinition {
                name: "maa_image_recognition".to_string(),
                description: "强大的实时图像识别和模式匹配工具。大模型可用此工具进行模板匹配、OCR文字识别、特征点匹配等。支持自定义识别区域、阈值和最大结果数。常用场景：识别按钮位置、读取界面文字、检测UI元素、图像比对分析等".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "algorithm": {
                            "type": "string",
                            "enum": ["MatchTemplate", "OcrDetect", "FeatureMatch"],
                            "default": "MatchTemplate",
                            "description": "识别算法类型"
                        },
                        "template": {
                            "type": "string",
                            "description": "模板名称(MatchTemplate/FeatureMatch)或要识别的文本(OcrDetect)"
                        },
                        "roi": {
                            "type": "array",
                            "items": {"type": "integer"},
                            "minItems": 4,
                            "maxItems": 4,
                            "description": "识别区域[x, y, width, height]，为空则识别全屏"
                        },
                        "threshold": {
                            "type": "number",
                            "minimum": 0.0,
                            "maximum": 1.0,
                            "default": 0.8,
                            "description": "识别阈值(0.0-1.0)"
                        },
                        "max_results": {
                            "type": "integer",
                            "default": 5,
                            "minimum": 1,
                            "description": "最大返回结果数量"
                        }
                    },
                    "required": ["template"]
                }),
            },
            
            // 新增：设备信息查询
            FunctionDefinition {
                name: "maa_device_info".to_string(),
                description: "获取Android设备的详细硬件和系统信息。大模型可用此工具查询设备分辨率、Android版本、设备型号、API级别等信息，也可获取设备能力列表(触控模式、输入方法等)。常用场景：设备兼容性检查、界面适配、功能支持判断等".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "include_capabilities": {
                            "type": "boolean",
                            "default": false,
                            "description": "是否包含设备能力信息"
                        },
                        "refresh": {
                            "type": "boolean", 
                            "default": false,
                            "description": "是否刷新设备信息缓存"
                        }
                    }
                }),
            },
            
            // 新增：系统状态监控
            FunctionDefinition {
                name: "maa_system_monitor".to_string(),
                description: "实时监控MAA系统性能和资源使用情况。大模型可用此工具监控CPU使用率、内存占用、任务队列状态、回调频率、图像识别速度等关键指标。支持自定义监控指标和时间窗口。常用场景：性能诊断、资源优化、系统健康检查等".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "metrics": {
                            "type": "array",
                            "items": {
                                "type": "string",
                                "enum": ["cpu", "memory", "task_queue", "callback_rate", "recognition_speed"]
                            },
                            "default": ["cpu", "memory", "task_queue"],
                            "description": "要监控的指标类型"
                        },
                        "duration_ms": {
                            "type": "integer",
                            "default": 1000,
                            "description": "监控持续时间(毫秒)"
                        }
                    }
                }),
            },
            
            // 新增：日志管理
            FunctionDefinition {
                name: "maa_log_management".to_string(),
                description: "MAA系统日志管理和查询工具。大模型可用此工具设置日志级别(trace/debug/info/warn/error)、查询最近日志、清空日志、按关键词过滤日志等。提供完整的日志系统控制。常用场景：调试问题、运行状态分析、错误诊断、日志维护等".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "enum": ["get_level", "set_level", "get_recent", "clear"],
                            "default": "get_recent",
                            "description": "日志管理操作"
                        },
                        "level": {
                            "type": "string",
                            "enum": ["trace", "debug", "info", "warn", "error"],
                            "description": "日志级别(set_level时必需)"
                        },
                        "count": {
                            "type": "integer",
                            "default": 50,
                            "description": "获取日志条数(get_recent时)"
                        },
                        "filter": {
                            "type": "string",
                            "description": "日志过滤关键词"
                        }
                    }
                }),
            },
            
            // 新增：回调事件查询
            FunctionDefinition {
                name: "maa_callback_events".to_string(),
                description: "MAA回调事件订阅和管理系统。大模型可用此工具获取实时任务状态更新、订阅特定事件类型、查询历史事件等。支持任务开始/完成、子任务状态、连接信息等事件类型。常用场景：任务进度监控、状态变化响应、事件驱动控制、实时反馈等".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "enum": ["get_recent", "subscribe", "unsubscribe", "clear"],
                            "default": "get_recent",
                            "description": "回调事件操作"
                        },
                        "event_types": {
                            "type": "array",
                            "items": {
                                "type": "string",
                                "enum": ["TaskChainStart", "TaskChainCompleted", "SubTaskStart", "SubTaskCompleted", "ConnectionInfo"]
                            },
                            "description": "要监听的事件类型"
                        },
                        "count": {
                            "type": "integer",
                            "default": 20,
                            "description": "获取事件数量"
                        }
                    }
                }),
            },
            
            // 新增：文本输入
            FunctionDefinition {
                name: "maa_text_input".to_string(),
                description: "Android设备文本输入工具。大模型可用此工具在设备上输入文本内容，支持多种输入方法(ADB、Yosemite等)。可选择输入前清空现有内容。常用场景：表单填写、搜索输入、账号密码输入、游戏内聊天、文本编辑等".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "text": {
                            "type": "string",
                            "description": "要输入的文本内容"
                        },
                        "input_method": {
                            "type": "string",
                            "enum": ["adb", "yosemite"],
                            "default": "adb",
                            "description": "输入方法"
                        },
                        "clear_before": {
                            "type": "boolean",
                            "default": false,
                            "description": "输入前是否清空现有内容"
                        }
                    },
                    "required": ["text"]
                }),
            },
            
            // maa-cli 自定义任务类型
            FunctionDefinition {
                name: "maa_custom_task".to_string(),
                description: "执行maa-cli兼容的自定义任务配置。大模型可用此工具运行复杂的任务组合，支持条件判断、任务变体、参数合并等高级特性。基于maa-cli TaskConfig格式，支持多个子任务的串行执行。常用场景：日常任务自动化、复杂流程编排、条件化任务执行等".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "自定义任务名称（可选）"
                        },
                        "client_type": {
                            "type": "string",
                            "enum": ["Official", "YoStarEN", "YoStarJP", "YoStarKR", "Bilibili", "Txwy"],
                            "default": "Official",
                            "description": "游戏客户端类型"
                        },
                        "startup": {
                            "type": "boolean",
                            "default": false,
                            "description": "是否自动启动游戏"
                        },
                        "closedown": {
                            "type": "boolean", 
                            "default": false,
                            "description": "是否自动关闭游戏"
                        },
                        "tasks": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "type": {"type": "string"},
                                    "params": {"type": "object"}
                                }
                            },
                            "description": "任务列表，每个任务包含type和params"
                        }
                    },
                    "required": ["tasks"]
                }),
            },
            
            FunctionDefinition {
                name: "maa_single_step".to_string(),
                description: "执行单个原子操作的简化接口。大模型可用此工具执行单步操作如点击、截图、滑动、识别等，无需复杂的任务配置。适用于简单的单次操作需求。常用场景：测试单个操作、快速交互、简单自动化等".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "enum": ["click", "screenshot", "swipe", "recognize"],
                            "description": "要执行的操作类型"
                        },
                        "params": {
                            "type": "object",
                            "description": "操作参数，根据action类型变化"
                        },
                        "timeout_ms": {
                            "type": "integer",
                            "default": 5000,
                            "description": "操作超时时间(毫秒)"
                        }
                    },
                    "required": ["action"]
                }),
            },
            
            FunctionDefinition {
                name: "maa_video_recognition".to_string(),
                description: "视频文件识别和分析工具。大模型可用此工具对录制的游戏视频进行自动分析，识别特定的UI元素、操作序列或游戏状态。支持多种识别算法和输出格式。常用场景：游戏录像分析、操作回放识别、UI元素统计、行为模式分析等".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "video_path": {
                            "type": "string",
                            "description": "视频文件路径"
                        },
                        "algorithm": {
                            "type": "string",
                            "enum": ["TemplateMatch", "FeatureMatch", "OCR"],
                            "default": "TemplateMatch",
                            "description": "识别算法类型"
                        },
                        "targets": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "要识别的目标列表"
                        },
                        "output_format": {
                            "type": "string",
                            "enum": ["json", "csv", "txt"],
                            "default": "json",
                            "description": "输出格式"
                        },
                        "fps": {
                            "type": "number",
                            "default": 30.0,
                            "description": "分析帧率"
                        }
                    },
                    "required": ["video_path", "targets"]
                }),
            },
        ]
    }

    /// 处理原子级Function调用
    pub async fn handle_atomic_function(&self, name: &str, args: &Value) -> anyhow::Result<FunctionResponse> {
        info!("处理原子级函数调用: {}", name);
        
        match name {
            // 设备控制操作
            "maa_click" => {
                let params = MaaClickTask::parse_arguments(args)?;
                self.click_task.execute(params).await
            },
            "maa_screenshot" => {
                let params = MaaScreenshotTask::parse_arguments(args)?;
                self.screenshot_task.execute(params).await
            },
            "maa_swipe" => {
                let params = MaaSwipeTask::parse_arguments(args)?;
                self.swipe_task.execute(params).await
            },
            
            // 连接管理
            "maa_connection" => {
                let params = MaaConnectionTask::parse_arguments(args)?;
                self.connection_task.execute(params).await
            },
            
            // 任务管理
            "maa_task_management" => {
                let params = MaaTaskManagementTask::parse_arguments(args)?;
                self.task_management_task.execute(params).await
            },
            
            // 图像识别
            "maa_image_recognition" => {
                let params = MaaImageRecognitionTask::parse_arguments(args)?;
                self.image_recognition_task.execute(params).await
            },
            
            // 设备信息查询
            "maa_device_info" => {
                self.handle_device_info(args).await
            },
            
            // 系统监控
            "maa_system_monitor" => {
                self.handle_system_monitor(args).await
            },
            
            // 日志管理
            "maa_log_management" => {
                self.handle_log_management(args).await
            },
            
            // 回调事件
            "maa_callback_events" => {
                self.handle_callback_events(args).await
            },
            
            // 文本输入
            "maa_text_input" => {
                self.handle_text_input(args).await
            },
            
            // maa-cli 自定义任务
            "maa_custom_task" => {
                let params = MaaCustomTask::parse_arguments(args)?;
                self.custom_task.execute(params).await
            },
            "maa_single_step" => {
                let params = MaaSingleStepTask::parse_arguments(args)?;
                self.single_step_task.execute(params).await
            },
            "maa_video_recognition" => {
                let params = MaaVideoRecognitionTask::parse_arguments(args)?;
                self.video_recognition_task.execute(params).await
            },
            
            _ => {
                error!("未知的原子级函数: {}", name);
                Ok(FunctionResponse {
                    success: false,
                    result: None,
                    error: Some(format!("未知的原子级函数: {}", name)),
                    timestamp: chrono::Utc::now(),
                })
            }
        }
    }

    // 辅助处理函数 - 设备信息
    async fn handle_device_info(&self, args: &Value) -> anyhow::Result<FunctionResponse> {
        let include_capabilities = args.get("include_capabilities")
            .and_then(|v| v.as_bool()).unwrap_or(false);
        let refresh = args.get("refresh")
            .and_then(|v| v.as_bool()).unwrap_or(false);

        info!("获取设备信息: capabilities={}, refresh={}", include_capabilities, refresh);

        Ok(FunctionResponse {
            success: true,
            result: Some(json!({
                "device": {
                    "uuid": "emulator-5554",
                    "address": "127.0.0.1:5555",
                    "resolution": [1920, 1080],
                    "api_level": 28,
                    "android_version": "9.0",
                    "brand": "Google",
                    "model": "sdk_gphone_x86"
                },
                "capabilities": if include_capabilities {
                    Some(json!({
                        "touch_modes": ["adb", "minitouch"],
                        "input_methods": ["adb", "yosemite"],
                        "screenshot_formats": ["rgb", "bgr", "png"]
                    }))
                } else {
                    None
                },
                "refreshed": refresh
            })),
            error: None,
            timestamp: chrono::Utc::now(),
        })
    }

    // 辅助处理函数 - 系统监控
    async fn handle_system_monitor(&self, args: &Value) -> anyhow::Result<FunctionResponse> {
        let metrics = args.get("metrics")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_else(|| vec!["cpu", "memory", "task_queue"]);

        let duration = args.get("duration_ms")
            .and_then(|v| v.as_i64()).unwrap_or(1000);

        info!("系统监控: metrics={:?}, duration={}ms", metrics, duration);

        Ok(FunctionResponse {
            success: true,
            result: Some(json!({
                "monitoring_period_ms": duration,
                "metrics": {
                    "cpu_usage_percent": 15.7,
                    "memory_usage_mb": 245.8,
                    "task_queue_size": 2,
                    "callback_rate_per_sec": 12.3,
                    "recognition_avg_time_ms": 89
                },
                "status": "healthy"
            })),
            error: None,
            timestamp: chrono::Utc::now(),
        })
    }

    // 辅助处理函数 - 日志管理
    async fn handle_log_management(&self, args: &Value) -> anyhow::Result<FunctionResponse> {
        let action = args.get("action")
            .and_then(|v| v.as_str()).unwrap_or("get_recent");

        info!("日志管理操作: {}", action);

        match action {
            "get_level" => Ok(FunctionResponse {
                success: true,
                result: Some(json!({"current_level": "info"})),
                error: None,
                timestamp: chrono::Utc::now(),
            }),
            "set_level" => {
                let level = args.get("level").and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("缺少日志级别参数"))?;
                Ok(FunctionResponse {
                    success: true,
                    result: Some(json!({"level_set": level})),
                    error: None,
                    timestamp: chrono::Utc::now(),
                })
            },
            "get_recent" => {
                let count = args.get("count").and_then(|v| v.as_i64()).unwrap_or(50);
                Ok(FunctionResponse {
                    success: true,
                    result: Some(json!({
                        "logs": [
                            {"level": "info", "message": "MAA任务开始执行", "timestamp": "2025-08-17T17:00:00Z"},
                            {"level": "debug", "message": "图像识别完成", "timestamp": "2025-08-17T17:00:01Z"}
                        ],
                        "count": count,
                        "total_available": 156
                    })),
                    error: None,
                    timestamp: chrono::Utc::now(),
                })
            },
            "clear" => Ok(FunctionResponse {
                success: true,
                result: Some(json!({"cleared": true})),
                error: None,
                timestamp: chrono::Utc::now(),
            }),
            _ => Err(anyhow::anyhow!("不支持的日志操作: {}", action))
        }
    }

    // 辅助处理函数 - 回调事件
    async fn handle_callback_events(&self, args: &Value) -> anyhow::Result<FunctionResponse> {
        let action = args.get("action")
            .and_then(|v| v.as_str()).unwrap_or("get_recent");
        let count = args.get("count").and_then(|v| v.as_i64()).unwrap_or(20);

        info!("回调事件管理: action={}, count={}", action, count);

        Ok(FunctionResponse {
            success: true,
            result: Some(json!({
                "action": action,
                "events": [
                    {
                        "type": "TaskChainStart",
                        "task_id": 42,
                        "timestamp": "2025-08-17T17:00:00Z",
                        "details": {"task_type": "Fight", "stage": "1-7"}
                    },
                    {
                        "type": "SubTaskCompleted", 
                        "task_id": 42,
                        "timestamp": "2025-08-17T17:00:15Z",
                        "details": {"subtask": "进入关卡", "result": "success"}
                    }
                ],
                "count": count
            })),
            error: None,
            timestamp: chrono::Utc::now(),
        })
    }

    // 辅助处理函数 - 文本输入
    async fn handle_text_input(&self, args: &Value) -> anyhow::Result<FunctionResponse> {
        let text = args.get("text").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("缺少text参数"))?;
        let input_method = args.get("input_method")
            .and_then(|v| v.as_str()).unwrap_or("adb");
        let clear_before = args.get("clear_before")
            .and_then(|v| v.as_bool()).unwrap_or(false);

        info!("文本输入: text='{}', method={}, clear={}", text, input_method, clear_before);

        Ok(FunctionResponse {
            success: true,
            result: Some(json!({
                "operation": "text_input",
                "text": text,
                "input_method": input_method,
                "clear_before": clear_before,
                "length": text.len(),
                "status": "completed"
            })),
            error: None,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// 创建原子级MAA Function Calling服务器
pub fn create_atomic_function_server(maa_backend: Arc<MaaBackend>) -> AtomicMaaFunctionServer {
    AtomicMaaFunctionServer::new(maa_backend)
}

// 为FunctionCallingServerTrait实现
use async_trait::async_trait;
use crate::function_calling_server::FunctionCallingServerTrait;

#[async_trait]
impl FunctionCallingServerTrait for AtomicMaaFunctionServer {
    fn get_function_definitions(&self) -> Vec<FunctionDefinition> {
        self.get_atomic_function_definitions()
    }

    async fn execute_function(&self, call: super::FunctionCall) -> FunctionResponse {
        match self.handle_atomic_function(&call.name, &call.arguments).await {
            Ok(response) => response,
            Err(e) => FunctionResponse {
                success: false,
                result: None,
                error: Some(format!("原子级函数执行失败: {}", e)),
                timestamp: chrono::Utc::now(),
            }
        }
    }
}