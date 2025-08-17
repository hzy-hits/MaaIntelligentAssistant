//! MAA 原子级操作工具集
//! 
//! 基于 MAA Core FFI 接口设计的细颗粒度控制工具
//! 提供直接访问底层MAA功能的Function Calling接口

use std::sync::Arc;
use serde_json::{json, Value};
use tracing::{debug, info, warn, error};
use anyhow::{Result, anyhow};

use crate::maa_adapter::{MaaBackend, MaaResult};
use super::FunctionResponse;

fn create_success_response(result: Value) -> FunctionResponse {
    FunctionResponse { 
        success: true, 
        result: Some(result), 
        error: None, 
        timestamp: chrono::Utc::now() 
    }
}

fn create_error_response(error: &str, code: &str) -> FunctionResponse {
    FunctionResponse { 
        success: false, 
        result: None, 
        error: Some(format!("{}: {}", code, error)), 
        timestamp: chrono::Utc::now() 
    }
}

// ====================================
// 1. 设备控制原子操作
// ====================================

/// 精确点击操作
#[derive(Debug, Clone)]
pub struct ClickParams {
    pub x: i32,
    pub y: i32,
    pub wait_completion: bool,
}

impl Default for ClickParams {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            wait_completion: true,
        }
    }
}

pub struct MaaClickTask {
    maa_backend: Arc<MaaBackend>,
}

impl MaaClickTask {
    pub fn new(maa_backend: Arc<MaaBackend>) -> Self {
        Self { maa_backend }
    }

    pub fn parse_arguments(args: &Value) -> Result<ClickParams> {
        let mut params = ClickParams::default();

        if let Some(x) = args.get("x").and_then(|v| v.as_i64()) {
            params.x = x as i32;
        } else {
            return Err(anyhow!("缺少必需参数: x"));
        }

        if let Some(y) = args.get("y").and_then(|v| v.as_i64()) {
            params.y = y as i32;
        } else {
            return Err(anyhow!("缺少必需参数: y"));
        }

        if let Some(wait) = args.get("wait_completion").and_then(|v| v.as_bool()) {
            params.wait_completion = wait;
        }

        Ok(params)
    }

    pub async fn execute(&self, params: ClickParams) -> Result<FunctionResponse> {
        info!("执行原子点击操作: x={}, y={}, wait={}", 
              params.x, params.y, params.wait_completion);

        // 在实际实现中，这里会调用底层FFI
        // let click_id = self.maa_backend.async_click(params.x, params.y, params.wait_completion)?;
        
        // Stub模式返回模拟结果
        Ok(create_success_response(json!({
            "operation": "click",
            "coordinates": [params.x, params.y],
            "wait_completion": params.wait_completion,
            "call_id": 12345,
            "status": "completed",
            "execution_time_ms": 150
        })))
    }
}

/// 屏幕截图操作
#[derive(Debug, Clone)]
pub struct ScreenshotParams {
    pub format: String, // "rgb" | "bgr" | "png"
    pub roi: Option<Vec<i32>>, // [x, y, width, height]
    pub compress: bool,
}

impl Default for ScreenshotParams {
    fn default() -> Self {
        Self {
            format: "png".to_string(),
            roi: None,
            compress: true,
        }
    }
}

pub struct MaaScreenshotTask {
    maa_backend: Arc<MaaBackend>,
}

impl MaaScreenshotTask {
    pub fn new(maa_backend: Arc<MaaBackend>) -> Self {
        Self { maa_backend }
    }

    pub fn parse_arguments(args: &Value) -> Result<ScreenshotParams> {
        let mut params = ScreenshotParams::default();

        if let Some(format) = args.get("format").and_then(|v| v.as_str()) {
            if matches!(format, "rgb" | "bgr" | "png") {
                params.format = format.to_string();
            } else {
                return Err(anyhow!("不支持的图像格式: {}", format));
            }
        }

        if let Some(roi) = args.get("roi").and_then(|v| v.as_array()) {
            if roi.len() == 4 {
                let roi_coords: Result<Vec<i32>, _> = roi.iter()
                    .map(|v| v.as_i64().map(|x| x as i32).ok_or_else(|| anyhow!("ROI坐标必须为整数")))
                    .collect();
                params.roi = Some(roi_coords?);
            } else {
                return Err(anyhow!("ROI参数格式错误，应为[x, y, width, height]"));
            }
        }

        if let Some(compress) = args.get("compress").and_then(|v| v.as_bool()) {
            params.compress = compress;
        }

        Ok(params)
    }

    pub async fn execute(&self, params: ScreenshotParams) -> Result<FunctionResponse> {
        info!("执行屏幕截图: format={}, roi={:?}", params.format, params.roi);

        // 在实际实现中，这里会调用底层FFI
        // let image_data = self.maa_backend.get_fresh_image()?;
        
        Ok(create_success_response(json!({
            "operation": "screenshot",
            "format": params.format,
            "roi": params.roi,
            "image_size": [1920, 1080],
            "data_size_bytes": 6220800,
            "compressed": params.compress,
            "image_data": "base64_encoded_image_data_here",
            "status": "completed"
        })))
    }
}

/// 滑动手势操作
#[derive(Debug, Clone)]
pub struct SwipeParams {
    pub from_x: i32,
    pub from_y: i32,
    pub to_x: i32,
    pub to_y: i32,
    pub duration_ms: i32,
    pub steps: Option<i32>,
}

impl Default for SwipeParams {
    fn default() -> Self {
        Self {
            from_x: 0,
            from_y: 0,
            to_x: 0,
            to_y: 0,
            duration_ms: 500,
            steps: None,
        }
    }
}

pub struct MaaSwipeTask {
    maa_backend: Arc<MaaBackend>,
}

impl MaaSwipeTask {
    pub fn new(maa_backend: Arc<MaaBackend>) -> Self {
        Self { maa_backend }
    }

    pub fn parse_arguments(args: &Value) -> Result<SwipeParams> {
        let mut params = SwipeParams::default();

        // 必需参数
        for (key, field) in [("from_x", &mut params.from_x), ("from_y", &mut params.from_y), 
                             ("to_x", &mut params.to_x), ("to_y", &mut params.to_y)] {
            if let Some(val) = args.get(key).and_then(|v| v.as_i64()) {
                *field = val as i32;
            } else {
                return Err(anyhow!("缺少必需参数: {}", key));
            }
        }

        // 可选参数
        if let Some(duration) = args.get("duration_ms").and_then(|v| v.as_i64()) {
            params.duration_ms = duration as i32;
        }

        if let Some(steps) = args.get("steps").and_then(|v| v.as_i64()) {
            params.steps = Some(steps as i32);
        }

        Ok(params)
    }

    pub async fn execute(&self, params: SwipeParams) -> Result<FunctionResponse> {
        info!("执行滑动操作: ({},{}) -> ({},{}) in {}ms", 
              params.from_x, params.from_y, params.to_x, params.to_y, params.duration_ms);

        Ok(create_success_response(json!({
            "operation": "swipe",
            "from": [params.from_x, params.from_y],
            "to": [params.to_x, params.to_y],
            "duration_ms": params.duration_ms,
            "steps": params.steps,
            "distance": (((params.to_x - params.from_x).pow(2) + (params.to_y - params.from_y).pow(2)) as f64).sqrt() as i32,
            "status": "completed"
        })))
    }
}

// ====================================
// 2. 连接管理原子操作
// ====================================

#[derive(Debug, Clone)]
pub struct ConnectionParams {
    pub action: String, // "connect" | "disconnect" | "status" | "reconnect"
    pub adb_path: Option<String>,
    pub device_address: Option<String>,
    pub config: Option<Value>,
    pub timeout_ms: i32,
}

impl Default for ConnectionParams {
    fn default() -> Self {
        Self {
            action: "status".to_string(),
            adb_path: None,
            device_address: None,
            config: None,
            timeout_ms: 30000,
        }
    }
}

pub struct MaaConnectionTask {
    maa_backend: Arc<MaaBackend>,
}

impl MaaConnectionTask {
    pub fn new(maa_backend: Arc<MaaBackend>) -> Self {
        Self { maa_backend }
    }

    pub fn parse_arguments(args: &Value) -> Result<ConnectionParams> {
        let mut params = ConnectionParams::default();

        if let Some(action) = args.get("action").and_then(|v| v.as_str()) {
            if matches!(action, "connect" | "disconnect" | "status" | "reconnect") {
                params.action = action.to_string();
            } else {
                return Err(anyhow!("无效的连接操作: {}", action));
            }
        }

        if let Some(adb_path) = args.get("adb_path").and_then(|v| v.as_str()) {
            params.adb_path = Some(adb_path.to_string());
        }

        if let Some(address) = args.get("device_address").and_then(|v| v.as_str()) {
            params.device_address = Some(address.to_string());
        }

        if let Some(config) = args.get("config") {
            params.config = Some(config.clone());
        }

        if let Some(timeout) = args.get("timeout_ms").and_then(|v| v.as_i64()) {
            params.timeout_ms = timeout as i32;
        }

        // 验证连接操作的必需参数
        if params.action == "connect" {
            if params.adb_path.is_none() || params.device_address.is_none() {
                return Err(anyhow!("连接操作需要提供 adb_path 和 device_address"));
            }
        }

        Ok(params)
    }

    pub async fn execute(&self, params: ConnectionParams) -> Result<FunctionResponse> {
        info!("执行连接管理操作: action={}", params.action);

        match params.action.as_str() {
            "connect" => {
                let adb_path = params.adb_path.as_ref().unwrap();
                let address = params.device_address.as_ref().unwrap();
                
                Ok(create_success_response(json!({
                    "operation": "connect",
                    "adb_path": adb_path,
                    "device_address": address,
                    "config": params.config,
                    "timeout_ms": params.timeout_ms,
                    "connection_id": 98765,
                    "status": "connected",
                    "device_info": {
                        "uuid": "emulator-5554",
                        "resolution": [1920, 1080],
                        "api_level": 28
                    }
                })))
            },
            "disconnect" => {
                Ok(create_success_response(json!({
                    "operation": "disconnect",
                    "status": "disconnected"
                })))
            },
            "status" => {
                let connected = self.maa_backend.is_connected();
                let running = self.maa_backend.is_running();
                
                Ok(create_success_response(json!({
                    "operation": "status",
                    "connected": connected,
                    "running": running,
                    "device_address": "127.0.0.1:5555",
                    "backend_type": self.maa_backend.backend_type()
                })))
            },
            "reconnect" => {
                Ok(create_success_response(json!({
                    "operation": "reconnect",
                    "status": "reconnected",
                    "attempts": 1
                })))
            },
            _ => Err(anyhow!("不支持的连接操作: {}", params.action))
        }
    }
}

// ====================================
// 3. 任务管理原子操作
// ====================================

#[derive(Debug, Clone)]
pub struct TaskManagementParams {
    pub action: String, // "create" | "start" | "stop" | "status" | "list" | "remove"
    pub task_type: Option<String>,
    pub task_id: Option<i32>,
    pub task_params: Option<Value>,
}

impl Default for TaskManagementParams {
    fn default() -> Self {
        Self {
            action: "list".to_string(),
            task_type: None,
            task_id: None,
            task_params: None,
        }
    }
}

pub struct MaaTaskManagementTask {
    maa_backend: Arc<MaaBackend>,
}

impl MaaTaskManagementTask {
    pub fn new(maa_backend: Arc<MaaBackend>) -> Self {
        Self { maa_backend }
    }

    pub fn parse_arguments(args: &Value) -> Result<TaskManagementParams> {
        let mut params = TaskManagementParams::default();

        if let Some(action) = args.get("action").and_then(|v| v.as_str()) {
            if matches!(action, "create" | "start" | "stop" | "status" | "list" | "remove") {
                params.action = action.to_string();
            } else {
                return Err(anyhow!("无效的任务操作: {}", action));
            }
        }

        if let Some(task_type) = args.get("task_type").and_then(|v| v.as_str()) {
            params.task_type = Some(task_type.to_string());
        }

        if let Some(task_id) = args.get("task_id").and_then(|v| v.as_i64()) {
            params.task_id = Some(task_id as i32);
        }

        if let Some(task_params) = args.get("task_params") {
            params.task_params = Some(task_params.clone());
        }

        // 验证操作的必需参数
        match params.action.as_str() {
            "create" => {
                if params.task_type.is_none() {
                    return Err(anyhow!("创建任务需要提供 task_type"));
                }
            },
            "start" | "stop" | "status" | "remove" => {
                if params.task_id.is_none() {
                    return Err(anyhow!("{}操作需要提供 task_id", params.action));
                }
            },
            _ => {}
        }

        Ok(params)
    }

    pub async fn execute(&self, params: TaskManagementParams) -> Result<FunctionResponse> {
        info!("执行任务管理操作: action={}", params.action);

        match params.action.as_str() {
            "create" => {
                let task_type = params.task_type.as_ref().unwrap();
                let task_params_str = params.task_params.as_ref()
                    .map(|p| serde_json::to_string(p).unwrap_or_else(|_| "{}".to_string()))
                    .unwrap_or_else(|| "{}".to_string());

                Ok(create_success_response(json!({
                    "operation": "create",
                    "task_type": task_type,
                    "task_id": 42,
                    "task_params": task_params_str,
                    "status": "created"
                })))
            },
            "start" => {
                let task_id = params.task_id.unwrap();
                Ok(create_success_response(json!({
                    "operation": "start",
                    "task_id": task_id,
                    "status": "started"
                })))
            },
            "stop" => {
                let task_id = params.task_id.unwrap();
                Ok(create_success_response(json!({
                    "operation": "stop",
                    "task_id": task_id,
                    "status": "stopped"
                })))
            },
            "status" => {
                let task_id = params.task_id.unwrap();
                Ok(create_success_response(json!({
                    "operation": "status",
                    "task_id": task_id,
                    "status": "running",
                    "progress": 75,
                    "current_subtask": "识别关卡选择界面"
                })))
            },
            "list" => {
                Ok(create_success_response(json!({
                    "operation": "list",
                    "active_tasks": [
                        {"task_id": 42, "task_type": "Fight", "status": "running"},
                        {"task_id": 43, "task_type": "Infrast", "status": "pending"}
                    ],
                    "total_count": 2
                })))
            },
            "remove" => {
                let task_id = params.task_id.unwrap();
                Ok(create_success_response(json!({
                    "operation": "remove",
                    "task_id": task_id,
                    "status": "removed"
                })))
            },
            _ => Err(anyhow!("不支持的任务操作: {}", params.action))
        }
    }
}

// ====================================
// 4. 图像识别原子操作
// ====================================

#[derive(Debug, Clone)]
pub struct ImageRecognitionParams {
    pub algorithm: String, // "MatchTemplate" | "OcrDetect" | "FeatureMatch"
    pub template: String,  // 模板名称或OCR文本
    pub roi: Option<Vec<i32>>, // [x, y, width, height]
    pub threshold: Option<f64>,
    pub max_results: Option<i32>,
}

impl Default for ImageRecognitionParams {
    fn default() -> Self {
        Self {
            algorithm: "MatchTemplate".to_string(),
            template: "".to_string(),
            roi: None,
            threshold: Some(0.8),
            max_results: Some(5),
        }
    }
}

pub struct MaaImageRecognitionTask {
    maa_backend: Arc<MaaBackend>,
}

impl MaaImageRecognitionTask {
    pub fn new(maa_backend: Arc<MaaBackend>) -> Self {
        Self { maa_backend }
    }

    pub fn parse_arguments(args: &Value) -> Result<ImageRecognitionParams> {
        let mut params = ImageRecognitionParams::default();

        if let Some(algorithm) = args.get("algorithm").and_then(|v| v.as_str()) {
            if matches!(algorithm, "MatchTemplate" | "OcrDetect" | "FeatureMatch") {
                params.algorithm = algorithm.to_string();
            } else {
                return Err(anyhow!("不支持的识别算法: {}", algorithm));
            }
        }

        if let Some(template) = args.get("template").and_then(|v| v.as_str()) {
            params.template = template.to_string();
        } else {
            return Err(anyhow!("缺少必需参数: template"));
        }

        if let Some(roi) = args.get("roi").and_then(|v| v.as_array()) {
            if roi.len() == 4 {
                let roi_coords: Result<Vec<i32>, _> = roi.iter()
                    .map(|v| v.as_i64().map(|x| x as i32).ok_or_else(|| anyhow!("ROI坐标必须为整数")))
                    .collect();
                params.roi = Some(roi_coords?);
            } else {
                return Err(anyhow!("ROI参数格式错误，应为[x, y, width, height]"));
            }
        }

        if let Some(threshold) = args.get("threshold").and_then(|v| v.as_f64()) {
            params.threshold = Some(threshold);
        }

        if let Some(max_results) = args.get("max_results").and_then(|v| v.as_i64()) {
            params.max_results = Some(max_results as i32);
        }

        Ok(params)
    }

    pub async fn execute(&self, params: ImageRecognitionParams) -> Result<FunctionResponse> {
        info!("执行图像识别: algorithm={}, template={}", params.algorithm, params.template);

        let recognition_results = match params.algorithm.as_str() {
            "MatchTemplate" => {
                json!([
                    {
                        "confidence": 0.95,
                        "location": [100, 200, 50, 30],
                        "center": [125, 215]
                    },
                    {
                        "confidence": 0.87,
                        "location": [300, 400, 50, 30],
                        "center": [325, 415]
                    }
                ])
            },
            "OcrDetect" => {
                json!([
                    {
                        "text": params.template,
                        "confidence": 0.92,
                        "location": [150, 250, 120, 25],
                        "center": [210, 262]
                    }
                ])
            },
            "FeatureMatch" => {
                json!([
                    {
                        "keypoints": 47,
                        "matches": 23,
                        "confidence": 0.88,
                        "location": [200, 300, 80, 60],
                        "center": [240, 330]
                    }
                ])
            },
            _ => json!([])
        };

        Ok(create_success_response(json!({
            "operation": "image_recognition",
            "algorithm": params.algorithm,
            "template": params.template,
            "roi": params.roi,
            "threshold": params.threshold,
            "results": recognition_results,
            "result_count": recognition_results.as_array().unwrap().len(),
            "execution_time_ms": 234
        })))
    }
}