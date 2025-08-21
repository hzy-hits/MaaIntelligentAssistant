//! 核心游戏功能模块
//!
//! 包含4个核心MAA游戏功能：
//! - maa_startup: 游戏启动管理
//! - maa_combat_enhanced: 增强战斗系统
//! - maa_recruit_enhanced: 智能招募管理
//! - maa_infrastructure_enhanced: 基建自动化

use serde_json::{json, Value};
use tracing::{debug, info};

use super::types::{FunctionDefinition, FunctionResponse, MaaError, ResourceUsage};
// V2架构：queue_client 已废弃，实际执行逻辑在 handler_v2.rs 中
use std::time::Instant;
use crate::config::CONFIG;

/// 创建启动任务工具定义
pub fn create_startup_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_startup".to_string(),
        description: "启动明日方舟游戏客户端并进入主界面。支持多种客户端类型、账号切换、自动启动模拟器等功能。适用场景：开始自动化任务前的准备工作、切换账号、游戏崩溃后重启。执行时间约60-120秒，需要确保设备已连接且游戏已安装。".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "client_type": {
                    "type": "string",
                    "description": "游戏客户端类型：Official(官服)、Bilibili(B服)、txwy(腾讯微游戏)、YoStarEN(国际服英文)、YoStarJP(日服)、YoStarKR(韩服)",
                    "enum": CONFIG.client.supported_clients,
                    "default": &CONFIG.client.default_client
                },
                "start_app": {
                    "type": "boolean", 
                    "description": "是否自动启动游戏应用程序。true=自动启动游戏客户端，false=只连接设备但不启动游戏",
                    "default": true
                },
                "close_app": {
                    "type": "boolean",
                    "description": "所有任务完成后是否自动关闭游戏应用程序。建议日常任务后设为true以节省资源",
                    "default": false
                },
                "account_name": {
                    "type": "string",
                    "description": "账号标识，支持部分匹配（如手机号后4位、用户名等）。用于多账号切换，留空则使用当前账号",
                    "required": false
                },
                "start_emulator": {
                    "type": "boolean",
                    "description": "连接失败时是否尝试启动模拟器。需要在MAA设置中预先配置模拟器启动参数",
                    "default": true
                }
            },
            "required": ["client_type"]
        }),
    }
}

/// 处理启动任务
pub async fn handle_startup(args: Value, queue_client: &MaaQueueClient) -> FunctionResponse {
    let start_time = Instant::now();
    info!("处理游戏启动任务");
    
    // 参数解析和验证
    let client_type = args.get("client_type")
        .and_then(|v| v.as_str())
        .unwrap_or(&CONFIG.client.default_client);
    
    // 验证客户端类型
    if !CONFIG.client.is_valid_client(client_type) {
        let supported = CONFIG.client.supported_clients.join(", ");
        let error = MaaError::parameter_error(
            &format!("不支持的客户端类型: {}", client_type),
            Some(&format!("支持的类型: {}", supported))
        );
        return FunctionResponse::error("maa_startup", error);
    }
    
    let start_app = args.get("start_app")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
        
    let close_app = args.get("close_app")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    debug!("启动参数: client_type={}, start_app={}, close_app={}", 
           client_type, start_app, close_app);

    match queue_client.startup(client_type.to_string(), start_app, close_app).await {
        Ok(result) => {
            info!("游戏启动任务完成");
            
            let response_data = json!({
                "status": "success",
                "message": "游戏启动任务已完成",
                "client_type": client_type,
                "details": result
            });
            
            FunctionResponse::success("maa_startup", response_data)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
                .with_next_actions(vec![
                    "建议接下来执行奖励收集或基建管理".to_string()
                ])
        },
        Err(e) => {
            let error = MaaError::maa_core_error(
                &format!("游戏启动失败: {}", e),
                Some("请检查游戏客户端是否已安装且设备已连接")
            );
            debug!("游戏启动失败: {}", e);
            FunctionResponse::error("maa_startup", error)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        }
    }
}

/// 创建增强战斗工具定义
pub fn create_combat_enhanced_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_combat_enhanced".to_string(),
        description: "增强的自动战斗系统，支持主线关卡、资源本、副本、活动关卡等全部战斗类型。包含智能理智管理、自动使用理智药/源石、掉落统计、代理指挥等高级功能。支持中文关卡名称识别（如“龙门币本”→CE-5）。".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "stage": {
                    "type": "string",
                    "description": "关卡代码或中文名称。支持格式：1-7(主线)、CE-5(龙门币本)、LS-5(经验书本)、CA-5(技能书本)、AP-5(红票本)、PR-A-1(芯片本)、H6-4(困难关卡)等。中文别名：狗粮=1-7、龙门币本=CE-5、经验书本=LS-5、技能书本=CA-5、红票本=AP-5",
                    "examples": CONFIG.stages.example_stages
                },
                "medicine": {
                    "type": "integer",
                    "description": "理智药使用数量上限。0=不使用理智药，999=无限制使用。每瓶理智药回复60理智，价值约200龙门币",
                    "minimum": 0,
                    "maximum": 999,
                    "default": 0
                },
                "expiring_medicine": {
                    "type": "integer", 
                    "description": "48小时内过期理智药使用数量",
                    "minimum": 0,
                    "maximum": 999,
                    "default": 0
                },
                "stone": {
                    "type": "integer",
                    "description": "源石使用数量上限。0=不使用源石。每颗源石回复135理智（按照理智上限计算）。请谨慎设置，源石珍贵！", 
                    "minimum": 0,
                    "maximum": 99,
                    "default": 0
                },
                "times": {
                    "type": "integer",
                    "description": "作战次数。-1或0=用完理智为止，正数=固定次数。注意：设置固定次数时不会自动使用理智药/源石，需要手动设置medicine/stone参数",
                    "minimum": -1,
                    "maximum": 9999,
                    "default": -1
                },
                "target_material": {
                    "type": "string",
                    "description": "目标掉落材料名称，获得后停止战斗。支持中文材料名（如“固源岩”、“酒石”、“糖聚块”）。留空则不检查特定材料",
                    "required": false,
                    "examples": ["固源岩", "酒石", "糖聚块", "龙门币"]
                },
                "drop_stats": {
                    "type": "object", 
                    "description": "掉落统计和上报设置",
                    "properties": {
                        "enable": {"type": "boolean", "default": true, "description": "是否启用掉落统计"},
                        "upload_penguin": {"type": "boolean", "default": false, "description": "是否上报企鹅物流"},
                        "upload_yituliu": {"type": "boolean", "default": false, "description": "是否上报一图流"}
                    }
                },
                "auto_agent": {
                    "type": "boolean",
                    "description": "是否自动选择代理指挥。用于支持PRTS代理指挥卡的关卡，失败后会自动手操",
                    "default": true
                },
                "backup_stage": {
                    "type": "string",
                    "description": "代理指挥失败时的后备关卡。建议设置为简单的低消耗关卡如1-7",
                    "required": false,
                    "examples": CONFIG.stages.common_stages
                }
            },
            "required": ["stage"]
        }),
    }
}

/// 处理增强战斗任务
pub async fn handle_combat_enhanced(args: Value, queue_client: &MaaQueueClient) -> FunctionResponse {
    let start_time = Instant::now();
    info!("⚔️ 处理增强战斗任务");
    
    // 参数验证
    let stage = match args.get("stage").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => {
            let error = MaaError::parameter_error(
                "缺少必需参数: stage",
                Some("请提供关卡代码，如 1-7、CE-5、龙门币本 等")
            );
            return FunctionResponse::error("maa_combat_enhanced", error);
        }
    };
    
    let medicine = args.get("medicine")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
        
    let stone = args.get("stone")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
        
    let times = args.get("times")
        .and_then(|v| v.as_i64())
        .unwrap_or(1) as i32;

    debug!("战斗参数: stage={}, medicine={}, stone={}, times={}", 
           stage, medicine, stone, times);

    match queue_client.combat(stage.to_string(), medicine, stone, times).await {
        Ok(result) => {
            // 检查MAA Worker返回的状态
            let worker_status = result.get("status").and_then(|s| s.as_str()).unwrap_or("unknown");
            
            let response_data = match worker_status {
                "running" => {
                    info!("增强战斗任务已启动: {} (后台执行中)", stage);
                    json!({
                        "status": "running",
                        "message": format!("关卡 {} 战斗任务已启动，正在后台执行", stage),
                        "stage": stage,
                        "times_requested": times,
                        "details": result
                    })
                },
                "completed" | "success" => {
                    info!("增强战斗任务完成: {}", stage);
                    json!({
                        "status": "success",
                        "message": format!("关卡 {} 战斗任务完成", stage),
                        "stage": stage,
                        "times_completed": times,
                        "details": result
                    })
                },
                _ => {
                    info!("增强战斗任务状态: {} - {}", stage, worker_status);
                    json!({
                        "status": worker_status,
                        "message": format!("关卡 {} 任务状态: {}", stage, worker_status),
                        "stage": stage,
                        "details": result
                    })
                }
            };
            
            let resource_usage = ResourceUsage {
                sanity_used: Some(times * 6), // 估算理智消耗
                medicine_used: Some(medicine),
                stone_used: Some(stone),
                recruit_tickets_used: None,
                items_gained: std::collections::HashMap::new(),
            };
            
            FunctionResponse::success("maa_combat_enhanced", response_data)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
                .with_resource_usage(resource_usage)
                .with_recommendations(vec![
                    format!("如果理智不足，可以考虑使用理智药或源石")
                ])
        },
        Err(e) => {
            let error = MaaError::maa_core_error(
                &format!("战斗任务失败: {}", e),
                Some("请检查关卡名称是否正确、理智是否足够、设备连接是否正常")
            );
            debug!("战斗任务失败: {}", e);
            FunctionResponse::error("maa_combat_enhanced", error)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        }
    }
}

/// 创建增强招募工具定义
pub fn create_recruit_enhanced_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_recruit_enhanced".to_string(),
        description: "智能公开招募管理系统，支持标签分析、智能策略选择、结果预测、自动化招募等功能。可自动识别高级资深干员、资深干员、支援机械等稀有标签，并自动避免1星机器人。支持加急许可证使用和标签刷新。".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "max_times": {
                    "type": "integer",
                    "description": "最大招募次数，0表示用完所有招募票。每次招募消耗1张招募票",
                    "minimum": 0,
                    "maximum": 300,
                    "default": 0
                },
                "min_star": {
                    "type": "integer",
                    "description": "最低星级要求。只招募达到此星级或更高的干员，低于此星级的组合将被跳过",
                    "enum": [1, 2, 3, 4, 5, 6],
                    "default": 3
                },
                "priority_tags": {
                    "type": "array",
                    "description": "优先标签列表。包含这些标签的组合会被优先考虑",
                    "items": {"type": "string"},
                    "default": ["高级资深干员", "资深干员", "支援机械", "爆发", "新手"],
                    "examples": [["高级资深干员"], ["资深干员", "支援机械"]]
                },
                "avoid_robot": {
                    "type": "boolean",
                    "description": "是否避充1星机器人。true=将跳过只能招募到机器人的标签组合（如支援机械单独选择）",
                    "default": true
                },
                "expedite": {
                    "type": "boolean", 
                    "description": "是否使用加急许可证。true=立即完成招募，false=等待9小时或8小时。加急许可证较为珍贵，建议仅在高级资深或资深干员时使用",
                    "default": false
                },
                "expedite_limit": {
                    "type": "integer",
                    "description": "加急许可证使用上限。仅在expedite=true时生效",
                    "minimum": 0,
                    "maximum": 999,
                    "default": 999
                },
                "refresh_tags": {
                    "type": "boolean",
                    "description": "无适合标签组合时是否刷新标签。消耗1张招募票",
                    "default": false
                },
                "notify_rare": {
                    "type": "boolean",
                    "description": "出现稀有标签时是否通知用户。稀有标签包括高级资深干员、资深干员、支援机械等",
                    "default": true
                }
            },
            "required": []
        }),
    }
}

/// 处理增强招募任务
pub async fn handle_recruit_enhanced(args: Value, queue_client: &MaaQueueClient) -> FunctionResponse {
    let start_time = Instant::now();
    info!("处理增强招募任务");
    
    let times = args.get("times")
        .and_then(|v| v.as_i64())
        .unwrap_or(1) as i32;
        
    let expedite = args.get("expedite")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
        
    let skip_robot = args.get("skip_robot")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    debug!("招募参数: times={}, expedite={}, skip_robot={}", 
           times, expedite, skip_robot);

    match queue_client.recruit(times, expedite, skip_robot).await {
        Ok(result) => {
            info!("增强招募任务完成");
            let response_data = json!({
                "status": "success",
                "message": "招募任务已完成",
                "times": times,
                "expedite": expedite,
                "details": result
            });
            FunctionResponse::success("maa_recruit_enhanced", response_data)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        },
        Err(e) => {
            let error = MaaError::maa_core_error(
                &format!("招募任务失败: {}", e),
                Some("请检查招募票数量和网络连接")
            );
            debug!("招募任务失败: {}", e);
            FunctionResponse::error("maa_recruit_enhanced", error)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        }
    }
}

/// 创建增强基建工具定义
pub fn create_infrastructure_enhanced_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_infrastructure_enhanced".to_string(),
        description: "智能基建管理系统，支持全设施自动化、效率分析、智能排班、产物收集等功能。包括制造站（生产经验书/赤金/源石碎片）、贸易站（赁买龙门币）、发电站、宿舍（恢复心情）、接待室（线索交流）、办公室（线索收集）、控制中枢等设施。自动排班、收集产物、管理无人机等。".to_string(),
        parameters: json!({
            "type": "object", 
            "properties": {
                "operation_mode": {
                    "type": "string",
                    "description": "操作模式。full_auto=全自动（收集+排班），collect_only=仅收集产物，schedule_only=仅排班换班，custom=自定义设施",
                    "enum": ["full_auto", "collect_only", "schedule_only", "custom"],
                    "default": "full_auto"
                },
                "facilities": {
                    "type": "array",
                    "description": "要管理的设施类型列表。Mfg=制造站，Trade=贸易站，Power=发电站，Control=控制中枢，Reception=接待室，Office=办公室，Dorm=宿舍",
                    "items": {
                        "type": "string",
                        "enum": ["Mfg", "Trade", "Power", "Control", "Reception", "Office", "Dorm"]
                    },
                    "default": ["Mfg", "Trade", "Power", "Reception", "Office", "Dorm"]
                },
                "auto_shift": {
                    "type": "boolean",
                    "description": "是否自动换班。true=根据干员心情和效率自动排班，false=仅收集产物不换班",
                    "default": true  
                },
                "mood_threshold": {
                    "type": "integer",
                    "description": "心情阈值。干员心情低于此值时会被换下恢复。值越低换班越频繁，但效率更高",
                    "minimum": 0,
                    "maximum": 24,
                    "default": 12
                },
                "use_drone": {
                    "type": "boolean",
                    "description": "是否使用无人机加速生产。建议无人机满时启用，优先用于制造站",
                    "default": false
                },
                "drone_facility": {
                    "type": "string",
                    "description": "无人机优先使用设施。manufacturing=制造站（推荐），trading=贸易站",
                    "enum": ["manufacturing", "trading"],
                    "default": "manufacturing"
                },
                "trust_enabled": {
                    "type": "boolean",
                    "description": "宿舍中是否启用信赖度收集。建议开启，可提升干员信赖度",
                    "default": true
                },
                "plan_file": {
                    "type": "string",
                    "description": "基建排班配置文件名。需要在MAA中预先配置好排班方案",
                    "default": "plan.json"
                }
            },
            "required": []
        }),
    }
}

/// 处理增强基建任务
pub async fn handle_infrastructure_enhanced(args: Value, queue_client: &MaaQueueClient) -> FunctionResponse {
    let start_time = Instant::now();
    info!("处理增强基建任务");
    
    let default_facility = json!(["Mfg", "Trade", "Power"]);
    let facility = args.get("facility").unwrap_or(&default_facility);
    
    let dorm_trust_enabled = args.get("dorm_trust_enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
        
    let filename = args.get("filename")
        .and_then(|v| v.as_str())
        .unwrap_or("plan.json");

    debug!("基建参数: facility={:?}, dorm_trust_enabled={}, filename={}", 
           facility, dorm_trust_enabled, filename);

    // 转换facility Value到String数组
    let facility_list = if let Some(arr) = facility.as_array() {
        arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
    } else {
        vec![]
    };
    match queue_client.infrastructure(facility_list, "NotUse".to_string(), 0.3).await {
        Ok(result) => {
            info!("增强基建任务完成");
            let response_data = json!({
                "status": "success",
                "message": "基建任务已完成", 
                "facility": facility,
                "dorm_trust_enabled": dorm_trust_enabled,
                "details": result
            });
            FunctionResponse::success("maa_infrastructure_enhanced", response_data)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        },
        Err(e) => {
            let error = MaaError::maa_core_error(
                &format!("基建任务失败: {}", e),
                Some("请检查基建设置和游戏状态")
            );
            debug!("基建任务失败: {}", e);
            FunctionResponse::error("maa_infrastructure_enhanced", error)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
        }
    }
}