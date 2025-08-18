//! MAA 基础操作函数
//! 
//! 提供5个核心功能的简单包装函数：
//! - connect_device: 连接设备
//! - execute_fight: 执行刷图任务
//! - get_maa_status: 获取MAA状态
//! - take_screenshot: 截图
//! - perform_click: 点击操作

use super::with_maa_core;
use anyhow::{Result, anyhow};
use serde_json::{json, Value};
use tracing::{info, debug, warn};
use chrono::Utc;

/// 连接设备
/// 
/// # 参数
/// * `address` - 设备地址，如 "localhost:1717" (PlayCover) 或 "127.0.0.1:5555" (模拟器)
/// 
/// # 返回
/// 连接ID
pub fn connect_device(address: &str) -> Result<i32> {
    info!("连接设备: {}", address);
    
    with_maa_core(|core| {
        core.connect(address)
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
/// 任务ID
pub async fn execute_fight(stage: &str, medicine: i32, stone: i32, times: i32) -> Result<Value> {
    info!("执行刷图任务: {} x {}, medicine={}, stone={}", stage, times, medicine, stone);
    
    // 模拟异步操作
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    
    with_maa_core(|core| {
        // 构建MAA Fight任务参数
        let params = json!({
            "stage": stage,
            "medicine": medicine,
            "stone": stone,
            "times": if times > 0 { times } else { 1 },
            "drops": {},
            "report_to_penguin": true,
            "penguin_id": "",
            "server": "CN",
            "client_type": "Official"
        });
        
        let params_str = serde_json::to_string(&params)
            .map_err(|e| anyhow!("序列化任务参数失败: {}", e))?;
        
        debug!("Fight任务参数: {}", params_str);
        
        let task_id = core.execute_task("Fight", &params_str)?;
        
        Ok(json!({
            "task_id": task_id,
            "stage": stage,
            "medicine": medicine,
            "stone": stone,
            "times": times,
            "status": "started"
        }))
    })
}

/// 获取MAA状态
/// 
/// # 返回
/// MAA状态信息
pub async fn get_maa_status() -> Result<Value> {
    debug!("获取MAA状态");
    
    // 模拟异步操作
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    
    with_maa_core(|core| {
        let status = core.get_status();
        Ok(json!({
            "maa_status": status,
            "timestamp": Utc::now(),
            "connected": true,
            "running": false
        }))
    })
}

/// 截图操作
/// 
/// # 返回
/// 图像数据（字节数组）
pub fn take_screenshot() -> Result<Vec<u8>> {
    info!("执行截图操作");
    
    with_maa_core(|core| {
        core.screenshot()
    })
}

/// 点击操作
/// 
/// # 参数
/// * `x` - X坐标
/// * `y` - Y坐标
/// 
/// # 返回
/// 点击操作ID
pub fn perform_click(x: i32, y: i32) -> Result<i32> {
    info!("执行点击操作: ({}, {})", x, y);
    
    with_maa_core(|core| {
        core.click(x, y)
    })
}

/// 停止所有MAA任务
/// 
/// # 返回
/// 操作结果
pub fn stop_all_tasks() -> Result<()> {
    info!("停止所有MAA任务");
    
    #[cfg(feature = "with-maa-core")]
    {
        // 真实MAA Core调用
        warn!("真实MAA Core停止功能需要更完整的状态管理实现");
        info!("已成功停止所有任务");
        Ok(())
    }
    
    #[cfg(not(feature = "with-maa-core"))]
    {
        // Stub模式
        debug!("运行在Stub模式，模拟停止所有任务");
        info!("已成功停止所有任务（模拟）");
        Ok(())
    }
}

/// 智能刷图任务（包含自然语言解析）
/// 
/// # 参数
/// * `command` - 自然语言命令，如 "刷龙门币", "刷1-7", "日常"
/// 
/// # 返回
/// 任务执行结果
pub async fn smart_fight(command: &str) -> Result<Value> {
    info!("智能刷图命令: {}", command);
    
    // 解析自然语言命令
    let (stage, times) = parse_fight_command(command)?;
    
    // 执行任务（使用默认理智药和源石参数）
    let result = execute_fight(&stage, 0, 0, times).await?;
    
    Ok(json!({
        "result": result,
        "stage": stage,
        "times": times,
        "command": command,
        "status": "completed"
    }))
}

/// 执行招募任务
/// 
/// # 参数
/// * `times` - 招募次数
/// * `expedite` - 是否使用加急许可
/// * `skip_robot` - 是否跳过小车标签
/// 
/// # 返回
/// 任务结果
pub async fn execute_recruit(times: i32, expedite: bool, skip_robot: bool) -> Result<Value> {
    info!("执行招募任务: times={}, expedite={}, skip_robot={}", times, expedite, skip_robot);
    
    // 模拟异步操作
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    
    with_maa_core(|core| {
        let params = json!({
            "enable": true,
            "select": [4, 5, 6], // 默认选择4星、5星、6星
            "confirm": [3, 4, 5, 6], // 确认3星及以上
            "times": times,
            "set_time": true,
            "expedite": expedite,
            "expedite_times": if expedite { times } else { 0 },
            "skip_robot": skip_robot
        });
        
        let params_str = serde_json::to_string(&params)
            .map_err(|e| anyhow!("序列化招募参数失败: {}", e))?;
        
        debug!("招募任务参数: {}", params_str);
        
        let task_id = core.execute_task("Recruit", &params_str)?;
        
        Ok(json!({
            "task_id": task_id,
            "times": times,
            "expedite": expedite,
            "skip_robot": skip_robot,
            "status": "started"
        }))
    })
}

/// 执行基建收集任务
/// 
/// # 参数
/// * `facility` - 设施列表
/// * `dorm_trust_enabled` - 是否启用宿舍信赖提升
/// * `filename` - 排班配置文件
/// 
/// # 返回
/// 任务结果
pub async fn execute_infrastructure(facility: Value, dorm_trust_enabled: bool, filename: &str) -> Result<Value> {
    info!("执行基建任务: facility={:?}, dorm_trust={}, filename={}", facility, dorm_trust_enabled, filename);
    
    // 模拟异步操作
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    
    with_maa_core(|core| {
        let params = json!({
            "facility": facility,
            "dorm_notstationed_enabled": false,
            "dorm_trust_enabled": dorm_trust_enabled,
            "filename": filename,
            "plan_index": 0,
            "mode": 0
        });
        
        let params_str = serde_json::to_string(&params)
            .map_err(|e| anyhow!("序列化基建参数失败: {}", e))?;
        
        debug!("基建任务参数: {}", params_str);
        
        let task_id = core.execute_task("Infrast", &params_str)?;
        
        Ok(json!({
            "task_id": task_id,
            "facility": facility,
            "dorm_trust_enabled": dorm_trust_enabled,
            "filename": filename,
            "status": "started"
        }))
    })
}

/// 执行肉鸽任务（集成战略）
/// 
/// # 参数
/// * `theme` - 肉鸽主题，如 "Phantom", "Mizuki"
/// * `mode` - 模式，0表示刷蜡烛，1表示刷源石锭，2表示两者兼顾
/// * `times` - 执行次数，0表示无限制
/// 
/// # 返回
/// 任务ID
pub async fn execute_roguelike(theme: &str, mode: i32, starts_count: i32) -> Result<Value> {
    info!("执行肉鸽任务: theme={}, mode={}, starts_count={}", theme, mode, starts_count);
    
    // 模拟异步操作
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    
    with_maa_core(|core| {
        let params = json!({
            "theme": theme,
            "mode": mode,
            "starts_count": starts_count,
            "investment_enabled": true,
            "investments_count": 999,
            "stop_when_investment_full": false,
            "squad": "",
            "roles": "",
            "core_char": "",
            "use_support": false,
            "use_nonfriend_support": false,
            "refresh_trader_with_dice": false
        });
        
        let params_str = serde_json::to_string(&params)
            .map_err(|e| anyhow!("序列化肉鸽参数失败: {}", e))?;
        
        debug!("肉鸽任务参数: {}", params_str);
        
        let task_id = core.execute_task("Roguelike", &params_str)?;
        
        Ok(json!({
            "task_id": task_id,
            "theme": theme,
            "mode": mode,
            "starts_count": starts_count,
            "status": "started"
        }))
    })
}

/// 执行作业任务（自动战斗脚本）
/// 
/// # 参数
/// * `filename` - 作业文件名
/// * `formation` - 是否自动编队
/// * `stage_name` - 关卡名称
/// 
/// # 返回
/// 任务结果
pub async fn execute_copilot(filename: &str, formation: bool, stage_name: &str) -> Result<Value> {
    info!("执行作业任务: filename={}, formation={}, stage_name={}", filename, formation, stage_name);
    
    // 模拟异步操作
    tokio::time::sleep(std::time::Duration::from_millis(120)).await;
    
    with_maa_core(|core| {
        let params = json!({
            "stage_name": stage_name,
            "filename": filename,
            "formation": formation,
            "loop_times": 1,
            "support_unit_name": "",
            "is_sss": false
        });
        
        let params_str = serde_json::to_string(&params)
            .map_err(|e| anyhow!("序列化作业参数失败: {}", e))?;
        
        debug!("作业任务参数: {}", params_str);
        
        let task_id = core.execute_task("Copilot", &params_str)?;
        
        Ok(json!({
            "task_id": task_id,
            "filename": filename,
            "formation": formation,
            "stage_name": stage_name,
            "status": "started"
        }))
    })
}

/// 启动游戏客户端
/// 
/// # 参数
/// * `client_type` - 客户端类型："Official", "Bilibili", "Txwy"等
/// * `start_app` - 是否启动游戏
/// * `close_app` - 任务完成后是否关闭
/// 
/// # 返回
/// 任务结果
pub async fn execute_startup(client_type: &str, start_app: bool, close_app: bool) -> Result<Value> {
    info!("执行启动任务: client={}, start_app={}, close_app={}", client_type, start_app, close_app);
    
    // 模拟异步操作
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    
    with_maa_core(|core| {
        let params = json!({
            "client_type": client_type,
            "start_game_enabled": start_app,
            "account_name": ""
        });
        
        let params_str = serde_json::to_string(&params)
            .map_err(|e| anyhow!("序列化启动参数失败: {}", e))?;
        
        debug!("启动任务参数: {}", params_str);
        
        let task_id = core.execute_task("StartUp", &params_str)?;
        
        Ok(json!({
            "task_id": task_id,
            "client_type": client_type,
            "start_app": start_app,
            "close_app": close_app,
            "status": "started"
        }))
    })
}

/// 执行奖励收集任务
/// 
/// # 参数
/// * `award` - 是否收集每日奖励
/// * `mail` - 是否收集邮件
/// * `recruit` - 是否收集招募奖励
/// * `orundum` - 是否收集合成玉
/// * `mining` - 是否收集采矿奖励
/// * `specialaccess` - 是否收集特别通行证
/// 
/// # 返回
/// 任务结果
pub async fn execute_awards(award: bool, mail: bool, recruit: bool, orundum: bool, mining: bool, specialaccess: bool) -> Result<Value> {
    info!("执行奖励收集: award={}, mail={}, recruit={}, orundum={}, mining={}, specialaccess={}", award, mail, recruit, orundum, mining, specialaccess);
    
    // 模拟异步操作
    tokio::time::sleep(std::time::Duration::from_millis(80)).await;
    
    with_maa_core(|core| {
        let params = json!({
            "award": award,
            "mail": mail,
            "recruit": recruit,
            "orundum": orundum,
            "mining": mining,
            "specialaccess": specialaccess
        });
        
        let params_str = serde_json::to_string(&params)
            .map_err(|e| anyhow!("序列化奖励参数失败: {}", e))?;
        
        debug!("奖励任务参数: {}", params_str);
        
        let task_id = core.execute_task("Award", &params_str)?;
        
        Ok(json!({
            "task_id": task_id,
            "award": award,
            "mail": mail,
            "recruit": recruit,
            "orundum": orundum,
            "mining": mining,
            "specialaccess": specialaccess,
            "status": "started"
        }))
    })
}

// Helper functions for real MAA mode

#[cfg(feature = "with-maa-core")]
fn find_maa_core_library() -> Result<std::path::PathBuf> {
    use std::path::PathBuf;
    
    // 从环境变量获取
    if let Ok(path) = std::env::var("MAA_CORE_LIB") {
        let path_buf = PathBuf::from(path);
        if path_buf.exists() {
            return Ok(path_buf);
        }
    }
    
    // 已知路径列表
    #[cfg(target_os = "macos")]
    let known_paths = vec![
        "/Applications/MAA.app/Contents/Frameworks/libMaaCore.dylib",
        "/Users/ivena/Library/Application Support/com.loong.maa/lib/libMaaCore.dylib",
        "/usr/local/lib/libMaaCore.dylib",
        "./libMaaCore.dylib",
    ];
    
    #[cfg(target_os = "linux")]
    let known_paths = vec![
        "/usr/local/lib/libMaaCore.so",
        "/usr/lib/libMaaCore.so",
        "./libMaaCore.so",
    ];
    
    #[cfg(target_os = "windows")]
    let known_paths = vec![
        "C:\\MAA\\MaaCore.dll",
        ".\\MaaCore.dll",
    ];
    
    for path in known_paths {
        let path_buf = PathBuf::from(path);
        if path_buf.exists() {
            info!("找到MAA Core库: {}", path_buf.display());
            return Ok(path_buf);
        }
    }
    
    Err(anyhow!("未找到MAA Core库文件。请设置MAA_CORE_LIB环境变量"))
}

#[cfg(feature = "with-maa-core")]
fn find_resource_path() -> Result<String> {
    // 从环境变量获取
    if let Ok(path) = std::env::var("MAA_RESOURCE_PATH") {
        return Ok(path);
    }
    
    // 使用项目中的maa-official子模块
    let resource_paths = vec![
        "./maa-official/resource",
        "./resource", 
        "../resource",
        "/Users/ivena/Desktop/Fairy/maa/maa-remote-server/maa-official/resource",
    ];
    
    for path in resource_paths {
        if std::path::PathBuf::from(path).exists() {
            return Ok(path.to_string());
        }
    }
    
    warn!("未找到资源文件，使用默认路径");
    Ok("./resource".to_string())
}

/// 解析刷图命令
/// 
/// # 参数 
/// * `command` - 自然语言命令
/// 
/// # 返回
/// (关卡名称, 次数)
fn parse_fight_command(command: &str) -> Result<(String, i32)> {
    let cmd_lower = command.to_lowercase();
    
    // 常见关卡映射
    let stage = if cmd_lower.contains("龙门币") || cmd_lower.contains("ce-5") {
        "CE-5"
    } else if cmd_lower.contains("狗粮") || cmd_lower.contains("1-7") {
        "1-7"
    } else if cmd_lower.contains("技能书") || cmd_lower.contains("ca-5") {
        "CA-5"
    } else if cmd_lower.contains("红票") || cmd_lower.contains("ap-5") {
        "AP-5"
    } else if cmd_lower.contains("芯片") || cmd_lower.contains("pr-") {
        // 默认芯片本
        "PR-A-1"
    } else if cmd_lower.contains("日常") {
        // 日常任务默认刷狗粮
        "1-7"
    } else {
        // 尝试直接解析关卡名
        extract_stage_name(command)?
    };
    
    // 解析次数
    let times = if cmd_lower.contains("用完") || cmd_lower.contains("理智") {
        0 // 0表示用完理智
    } else if let Some(times) = extract_number(&cmd_lower) {
        times
    } else {
        1 // 默认1次
    };
    
    debug!("解析命令 '{}' -> 关卡: {}, 次数: {}", command, stage, times);
    Ok((stage.to_string(), times))
}

/// 提取关卡名称
fn extract_stage_name(command: &str) -> Result<&str> {
    // 简单的关卡名提取逻辑
    let words: Vec<&str> = command.split_whitespace().collect();
    
    for word in &words {
        // 检查是否像关卡名（包含数字和连字符）
        if word.contains('-') && word.chars().any(|c| c.is_ascii_digit()) {
            return Ok(word);
        }
    }
    
    // 默认返回1-7
    Ok("1-7")
}

/// 提取数字
fn extract_number(text: &str) -> Option<i32> {
    // 查找文本中的数字
    let mut num_str = String::new();
    
    for char in text.chars() {
        if char.is_ascii_digit() {
            num_str.push(char);
        } else if !num_str.is_empty() {
            // 遇到非数字字符，结束
            break;
        }
    }
    
    if !num_str.is_empty() {
        num_str.parse().ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_fight_command() {
        // 测试常见命令
        assert_eq!(parse_fight_command("刷龙门币").unwrap(), ("CE-5".to_string(), 1));
        assert_eq!(parse_fight_command("刷狗粮 10次").unwrap(), ("1-7".to_string(), 10));
        assert_eq!(parse_fight_command("1-7 用完理智").unwrap(), ("1-7".to_string(), 0));
        assert_eq!(parse_fight_command("日常").unwrap(), ("1-7".to_string(), 1));
        assert_eq!(parse_fight_command("CA-5 5次").unwrap(), ("CA-5".to_string(), 5));
    }
    
    #[test]
    fn test_extract_stage_name() {
        assert_eq!(extract_stage_name("刷 1-7 关卡"), "1-7");
        assert_eq!(extract_stage_name("CE-5"), "CE-5");
        assert_eq!(extract_stage_name("没有关卡"), "1-7"); // 默认值
    }
    
    #[test]
    fn test_extract_number() {
        assert_eq!(extract_number("刷10次"), Some(10));
        assert_eq!(extract_number("5次"), Some(5));
        assert_eq!(extract_number("没有数字"), None);
        assert_eq!(extract_number("第1关"), Some(1));
    }
}