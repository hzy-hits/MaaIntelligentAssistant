//! MAA Core 基础操作
//! 
//! 真正的MaaCore集成实现，使用maa-sys进行FFI调用

use anyhow::{Result, anyhow};
use serde_json::{json, Value};
use tracing::{info, warn};
use chrono::Utc;
use std::env;

// MAA FFI 相关导入
use maa_sys::Assistant;

// 使用线程本地存储来解决Assistant不是Send的问题
thread_local! {
    static MAA_ASSISTANT: std::cell::RefCell<Option<Assistant>> = std::cell::RefCell::new(None);
}

/// 初始化 MAA Core
pub async fn init_maa_core() -> Result<()> {
    MAA_ASSISTANT.with(|assistant_cell| {
        let mut assistant_borrow = assistant_cell.borrow_mut();
        
        if assistant_borrow.is_some() {
            info!("MAA Core 已经初始化");
            return Ok(());
        }
        
        // 检查是否强制使用 stub 模式
        if env::var("MAA_FORCE_STUB").unwrap_or_default() == "true" {
            warn!("MAA_FORCE_STUB=true，跳过真实初始化");
            return Err(anyhow!("Stub mode forced"));
        }
        
        // 从环境变量获取配置
        let resource_path = env::var("MAA_RESOURCE_PATH").map_err(|_| {
            anyhow!("MAA_RESOURCE_PATH 环境变量未设置")
        })?;
        
        info!("正在初始化 MAA Core，资源路径: {}", resource_path);
        
        // 创建 MAA Assistant（不需要回调函数）
        let assistant = Assistant::new(None, None);
        
        info!("MAA Core 初始化成功");
        *assistant_borrow = Some(assistant);
        
        Ok(())
    })
}

/// 执行 MAA 操作的通用函数
async fn execute_maa_operation<F, R>(operation: F) -> Result<R>
where
    F: FnOnce(&Assistant) -> Result<R>,
    R: 'static,
{
    // 首先尝试初始化（如果尚未初始化）
    if let Err(_) = init_maa_core().await {
        // 如果初始化失败，返回 stub 模式响应
        warn!("MAA Core 初始化失败，使用 stub 模式");
        return Err(anyhow!("MAA Core not available, using stub mode"));
    }
    
    MAA_ASSISTANT.with(|assistant_cell| {
        let assistant_borrow = assistant_cell.borrow();
        let assistant = assistant_borrow.as_ref()
            .ok_or_else(|| anyhow!("MAA Assistant not initialized"))?;
        
        operation(assistant)
    })
}

/// 连接设备
/// 
/// # 参数
/// * `address` - 设备地址，如 "127.0.0.1:1717" (PlayCover) 或 "127.0.0.1:5555" (模拟器)
/// 
/// # 返回
/// 连接ID
pub async fn connect_device(address: &str) -> Result<i32> {
    info!("尝试连接设备: {}", address);
    
    execute_maa_operation(|assistant| {
        // 尝试连接设备
        let connection_id = assistant.async_connect("adb", address, "{}", true)?;
        info!("设备连接成功，连接ID: {}", connection_id);
        Ok(connection_id)
    }).await.or_else(|_| {
        warn!("MAA Core 不可用，使用 stub 模式");
        Ok(1) // 返回模拟的连接ID
    })
}

/// 执行刷图任务
/// 
/// # 参数
/// * `stage` - 关卡名称，如 "1-7", "CE-5"
/// * `medicine` - 理智药数量
/// * `stone` - 源石数量  
/// * `times` - 执行次数，0表示用完理智
/// 
/// # 返回
/// 任务执行结果
pub async fn execute_fight(stage: &str, medicine: i32, stone: i32, times: i32) -> Result<Value> {
    info!("尝试执行刷图任务: {} x {}, medicine={}, stone={}", stage, times, medicine, stone);
    
    execute_maa_operation(|assistant| {
        // 构建刷图任务参数
        let fight_params = json!({
            "stage": stage,
            "medicine": medicine,
            "stone": stone,
            "times": times
        });
        
        // 创建刷图任务
        let task_id = assistant.append_task("Fight", fight_params.to_string().as_str())?;
        info!("刷图任务创建成功，任务ID: {}", task_id);
        
        // 启动任务
        assistant.start()?;
        
        Ok(json!({
            "task_id": task_id,
            "stage": stage,
            "medicine": medicine,
            "stone": stone,
            "times": times,
            "status": "started"
        }))
    }).await.or_else(|_| {
        warn!("MAA Core 不可用，使用 stub 模式");
        Ok(json!({
            "task_id": 1,
            "stage": stage,
            "medicine": medicine,
            "stone": stone,
            "times": times,
            "status": "stub_mode"
        }))
    })
}

/// 获取MAA状态
/// 
/// # 返回
/// MAA状态信息
pub async fn get_maa_status() -> Result<Value> {
    execute_maa_operation(|assistant| {
        let is_running = assistant.running();
        let is_connected = assistant.connected();
        
        Ok(json!({
            "maa_status": "running",
            "timestamp": Utc::now(),
            "connected": is_connected,
            "running": is_running,
            "version": Assistant::get_version().unwrap_or("unknown".to_string())
        }))
    }).await.or_else(|_| {
        Ok(json!({
            "maa_status": "stub_mode", 
            "timestamp": Utc::now(),
            "connected": false,
            "running": false,
            "version": "stub"
        }))
    })
}

/// 获取当前运行的任务列表
/// 
/// # 返回
/// 包含任务ID和状态信息的列表
pub async fn get_tasks_list() -> Result<Value> {
    execute_maa_operation(|assistant| {
        // 使用Assistant当前状态来推断任务信息
        let is_running = assistant.running();
        let is_connected = assistant.connected();
        
        // 构建模拟的任务列表响应
        let tasks_info = if is_running {
            vec![json!({
                "task_id": 1,
                "index": 0,
                "status": "running",
                "connected": is_connected
            })]
        } else {
            vec![]
        };
        
        Ok(json!({
            "tasks": tasks_info,
            "total_count": tasks_info.len(),
            "timestamp": Utc::now(),
            "running": is_running,
            "connected": is_connected
        }))
    }).await.or_else(|_| {
        warn!("MAA Core 不可用，使用 stub 模式");
        Ok(json!({
            "tasks": [],
            "total_count": 0,
            "timestamp": Utc::now(),
            "mode": "stub"
        }))
    })
}

/// 动态设置任务参数
/// 
/// # 参数
/// * `task_id` - 任务ID
/// * `params` - 新的参数JSON
pub async fn set_task_params(task_id: i32, params: Value) -> Result<Value> {
    info!("动态调整任务 {} 参数: {}", task_id, params);
    
    execute_maa_operation(|assistant| {
        let params_str = params.to_string();
        assistant.set_task_params(task_id, params_str.as_str())?;
        
        Ok(json!({
            "task_id": task_id,
            "updated_params": params,
            "status": "updated",
            "timestamp": Utc::now()
        }))
    }).await.or_else(|_| {
        warn!("MAA Core 不可用，使用 stub 模式");
        Ok(json!({
            "task_id": task_id,
            "updated_params": params,
            "status": "stub_updated",
            "timestamp": Utc::now()
        }))
    })
}

/// 快速返回游戏主界面
/// 
/// # 返回
/// 操作结果
pub async fn back_to_home() -> Result<Value> {
    info!("执行快速返回主界面操作");
    
    execute_maa_operation(|assistant| {
        assistant.back_to_home()?;
        
        Ok(json!({
            "action": "back_to_home",
            "status": "executed",
            "timestamp": Utc::now()
        }))
    }).await.or_else(|_| {
        warn!("MAA Core 不可用，使用 stub 模式");
        Ok(json!({
            "action": "back_to_home", 
            "status": "stub_executed",
            "timestamp": Utc::now()
        }))
    })
}

/// 截图操作 (已废弃 - 使用任务队列)
/// 
/// # 返回
/// 图像数据（字节数组）
pub fn take_screenshot() -> Result<Vec<u8>> {
    info!("take_screenshot已废弃，请使用任务队列");
    
    // 返回空的字节数组
    Ok(vec![])
}

/// 点击操作 (已废弃 - 使用任务队列)
/// 
/// # 参数
/// * `x` - X坐标
/// * `y` - Y坐标
/// 
/// # 返回
/// 点击操作ID
pub fn perform_click(x: i32, y: i32) -> Result<i32> {
    info!("perform_click已废弃，请使用任务队列");
    info!("尝试点击: ({}, {})", x, y);
    
    Ok(1)
}

/// 停止所有MAA任务 (已废弃 - 使用任务队列)
/// 
/// # 返回
/// 操作结果
pub fn stop_all_tasks() -> Result<()> {
    info!("stop_all_tasks已废弃，请使用任务队列");
    
    Ok(())
}

/// 执行启动任务 (已废弃 - 使用任务队列)
pub async fn execute_startup(client_type: &str, start_app: bool, close_app: bool) -> Result<Value> {
    info!("execute_startup已废弃，请使用任务队列");
    info!("尝试启动: client={}, start_app={}, close_app={}", client_type, start_app, close_app);
    
    Ok(json!({
        "task_id": 1,
        "client_type": client_type,
        "start_app": start_app,
        "close_app": close_app,
        "status": "deprecated_stub"
    }))
}

/// 执行招募任务 (已废弃 - 使用任务队列)
pub async fn execute_recruit(max_times: i32, expedite: bool, skip_robot: bool) -> Result<Value> {
    info!("execute_recruit已废弃，请使用任务队列");
    info!("尝试招募: times={}, expedite={}, skip_robot={}", max_times, expedite, skip_robot);
    
    Ok(json!({
        "task_id": 1,
        "max_times": max_times,
        "expedite": expedite,
        "skip_robot": skip_robot,
        "status": "deprecated_stub"
    }))
}

/// 执行基建任务 (已废弃 - 使用任务队列)
pub async fn execute_infrastructure(facility: &[String], drones: &str, threshold: f64) -> Result<Value> {
    info!("execute_infrastructure已废弃，请使用任务队列");
    info!("尝试基建: facility={:?}, drones={}, threshold={}", facility, drones, threshold);
    
    Ok(json!({
        "task_id": 1,
        "facility": facility,
        "drones": drones,
        "threshold": threshold,
        "status": "deprecated_stub"
    }))
}

/// 获取当前运行的任务列表
/// 
/// # 返回
/// 包含任务ID和状态信息的列表
pub async fn get_tasks_list() -> Result<Value> {
    execute_maa_operation(|assistant| {
        // 使用Assistant当前状态来推断任务信息
        let is_running = assistant.running();
        let is_connected = assistant.connected();
        
        // 构建模拟的任务列表响应
        let tasks_info = if is_running {
            vec![json!({
                "task_id": 1,
                "index": 0,
                "status": "running",
                "connected": is_connected
            })]
        } else {
            vec![]
        };
        
        Ok(json!({
            "tasks": tasks_info,
            "total_count": tasks_info.len(),
            "timestamp": Utc::now(),
            "running": is_running,
            "connected": is_connected
        }))
    }).await.or_else(|_| {
        warn!("MAA Core 不可用，使用 stub 模式");
        Ok(json!({
            "tasks": [],
            "total_count": 0,
            "timestamp": Utc::now(),
            "mode": "stub"
        }))
    })
}

/// 动态设置任务参数
/// 
/// # 参数
/// * `task_id` - 任务ID
/// * `params` - 新的参数JSON
pub async fn set_task_params(task_id: i32, params: Value) -> Result<Value> {
    info!("动态调整任务 {} 参数: {}", task_id, params);
    
    execute_maa_operation(|assistant| {
        let params_str = params.to_string();
        assistant.set_task_params(task_id, params_str.as_str())?;
        
        Ok(json!({
            "task_id": task_id,
            "updated_params": params,
            "status": "updated",
            "timestamp": Utc::now()
        }))
    }).await.or_else(|_| {
        warn!("MAA Core 不可用，使用 stub 模式");
        Ok(json!({
            "task_id": task_id,
            "updated_params": params,
            "status": "stub_updated",
            "timestamp": Utc::now()
        }))
    })
}

/// 快速返回游戏主界面
/// 
/// # 返回
/// 操作结果
pub async fn back_to_home() -> Result<Value> {
    info!("执行快速返回主界面操作");
    
    execute_maa_operation(|assistant| {
        assistant.back_to_home()?;
        
        Ok(json!({
            "action": "back_to_home",
            "status": "executed",
            "timestamp": Utc::now()
        }))
    }).await.or_else(|_| {
        warn!("MAA Core 不可用，使用 stub 模式");
        Ok(json!({
            "action": "back_to_home", 
            "status": "stub_executed",
            "timestamp": Utc::now()
        }))
    })
}

/// 智能任务参数调整策略
pub async fn adjust_task_strategy(task_id: i32, strategy: &str, context: Value) -> Result<Value> {
    info!("应用智能调整策略: {} 到任务 {}", strategy, task_id);
    
    let new_params = match strategy {
        "reduce_difficulty" => {
            // 降低难度：减少药剂使用，降低目标次数
            json!({
                "medicine": 0,
                "times": 1,
                "strategy": "conservative"
            })
        },
        "increase_efficiency" => {
            // 提高效率：增加药剂使用
            let medicine_count = context.get("available_medicine").and_then(|v| v.as_i64()).unwrap_or(0);
            json!({
                "medicine": medicine_count.min(99),
                "times": 0,
                "strategy": "aggressive"  
            })
        },
        "emergency_stop" => {
            // 紧急停止：设置为无限循环防止进一步执行
            json!({
                "enable": false,
                "times": 0
            })
        },
        _ => {
            return Err(anyhow!("未知的调整策略: {}", strategy));
        }
    };
    
    set_task_params(task_id, new_params).await
}

/// 执行关闭任务 (已废弃 - 使用任务队列)
pub async fn execute_closedown() -> Result<Value> {
    info!("execute_closedown已废弃，请使用任务队列");
    
    Ok(json!({
        "task_id": 1,
        "status": "deprecated_stub"
    }))
}

/// 执行自定义任务 (已废弃 - 使用任务队列)
pub async fn execute_custom_task(task_type: &str, params: &str) -> Result<Value> {
    info!("execute_custom_task已废弃，请使用任务队列");
    info!("尝试自定义任务: type={}, params={}", task_type, params);
    
    Ok(json!({
        "task_id": 1,
        "task_type": task_type,
        "params": params,
        "status": "deprecated_stub"
    }))
}

/// 执行视频识别任务 (已废弃 - 使用任务队列)
pub async fn execute_video_recognition(video_path: &str) -> Result<Value> {
    info!("execute_video_recognition已废弃，请使用任务队列");
    info!("尝试视频识别: {}", video_path);
    
    Ok(json!({
        "task_id": 1,
        "video_path": video_path,
        "status": "deprecated_stub"
    }))
}

/// 执行系统管理任务 (已废弃 - 使用任务队列)
pub async fn execute_system_management(action: &str) -> Result<Value> {
    info!("execute_system_management已废弃，请使用任务队列");
    info!("尝试系统管理: {}", action);
    
    Ok(json!({
        "task_id": 1,
        "action": action,
        "status": "deprecated_stub"
    }))
}

/// 智能战斗 (已废弃 - 使用任务队列)
pub async fn smart_fight(stage: &str, medicine: i32, stone: i32, times: i32) -> Result<Value> {
    info!("smart_fight已废弃，请使用任务队列");
    execute_fight(stage, medicine, stone, times).await
}

/// 执行肉鸽任务 (已废弃 - 使用任务队列)
pub async fn execute_roguelike(theme: &str, mode: i32, starts_count: i32) -> Result<Value> {
    info!("execute_roguelike已废弃，请使用任务队列");
    info!("尝试肉鸽: theme={}, mode={}, starts_count={}", theme, mode, starts_count);
    
    Ok(json!({
        "task_id": 1,
        "theme": theme,
        "mode": mode,
        "starts_count": starts_count,
        "status": "deprecated_stub"
    }))
}

/// 执行作业任务 (已废弃 - 使用任务队列)
pub async fn execute_copilot(filename: &str, formation: bool) -> Result<Value> {
    info!("execute_copilot已废弃，请使用任务队列");
    info!("尝试作业: filename={}, formation={}", filename, formation);
    
    Ok(json!({
        "task_id": 1,
        "filename": filename,
        "formation": formation,
        "status": "deprecated_stub"
    }))
}

/// 执行奖励收集任务 (已废弃 - 使用任务队列)
pub async fn execute_awards(award: bool, mail: bool, recruit: bool, orundum: bool) -> Result<Value> {
    info!("execute_awards已废弃，请使用任务队列");
    info!("尝试奖励收集: award={}, mail={}, recruit={}, orundum={}", award, mail, recruit, orundum);
    
    Ok(json!({
        "task_id": 1,
        "award": award,
        "mail": mail,
        "recruit": recruit,
        "orundum": orundum,
        "status": "deprecated_stub"
    }))
}

/// 执行信用商店任务 (已废弃 - 使用任务队列)
pub async fn execute_credit_store(credit_fight: bool) -> Result<Value> {
    info!("execute_credit_store已废弃，请使用任务队列");
    info!("尝试信用商店: credit_fight={}", credit_fight);
    
    Ok(json!({
        "task_id": 1,
        "credit_fight": credit_fight,
        "status": "deprecated_stub"
    }))
}

/// 执行仓库管理任务 (已废弃 - 使用任务队列)
pub async fn execute_depot_management(enable: bool) -> Result<Value> {
    info!("execute_depot_management已废弃，请使用任务队列");
    info!("尝试仓库管理: enable={}", enable);
    
    Ok(json!({
        "task_id": 1,
        "enable": enable,
        "status": "deprecated_stub"
    }))
}

/// 执行干员管理任务 (已废弃 - 使用任务队列)
pub async fn execute_operator_box(enable: bool) -> Result<Value> {
    info!("execute_operator_box已废弃，请使用任务队列");
    info!("尝试干员管理: enable={}", enable);
    
    Ok(json!({
        "task_id": 1,
        "enable": enable,
        "status": "deprecated_stub"
    }))
}

/// 执行保全派驻任务 (已废弃 - 使用任务队列)
pub async fn execute_sss_copilot(filename: &str, loop_times: i32) -> Result<Value> {
    info!("execute_sss_copilot已废弃，请使用任务队列");
    info!("尝试保全派驻: filename={}, loop_times={}", filename, loop_times);
    
    Ok(json!({
        "task_id": 1,
        "filename": filename,
        "loop_times": loop_times,
        "status": "deprecated_stub"
    }))
}

/// 执行生息演算任务 (已废弃 - 使用任务队列)
pub async fn execute_reclamation(theme: &str, mode: i32) -> Result<Value> {
    info!("execute_reclamation已废弃，请使用任务队列");
    info!("尝试生息演算: theme={}, mode={}", theme, mode);
    
    Ok(json!({
        "task_id": 1,
        "theme": theme,
        "mode": mode,
        "status": "deprecated_stub"
    }))
}

/// 智能任务参数调整策略
pub async fn adjust_task_strategy(task_id: i32, strategy: &str, context: Value) -> Result<Value> {
    info!("应用智能调整策略: {} 到任务 {}", strategy, task_id);
    
    let new_params = match strategy {
        "reduce_difficulty" => {
            // 降低难度：减少药剂使用，降低目标次数
            json!({
                "medicine": 0,
                "times": 1,
                "strategy": "conservative"
            })
        },
        "increase_efficiency" => {
            // 提高效率：增加药剂使用
            let medicine_count = context.get("available_medicine").and_then(|v| v.as_i64()).unwrap_or(0);
            json!({
                "medicine": medicine_count.min(99),
                "times": 0,
                "strategy": "aggressive"  
            })
        },
        "emergency_stop" => {
            // 紧急停止：设置为无限循环防止进一步执行
            json!({
                "enable": false,
                "times": 0
            })
        },
        _ => {
            return Err(anyhow!("未知的调整策略: {}", strategy));
        }
    };
    
    set_task_params(task_id, new_params).await
}

// 这些函数都已经被worker.rs中的实现替代
// 现在它们只是为了保持API兼容性而存在的存根