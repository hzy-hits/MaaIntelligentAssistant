//! 增强的MAA Function Calling工具集
//!
//! 基于maa-cli的16种任务类型，提供完整的MAA功能覆盖

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, info};

use crate::maa_adapter::MaaBackend;
use super::{FunctionDefinition, FunctionCall, FunctionResponse};
use super::maa_startup::{MaaStartUpTask, StartUpTaskParams};
use super::maa_combat::MaaCombatTask;
use super::maa_recruit::MaaRecruitTask;
use super::maa_infrastructure::MaaInfrastructureTask;
use super::maa_roguelike::MaaRoguelikeTask;
use super::maa_copilot_enhanced::MaaCopilotTask;
use super::maa_other_tools::*;

/// 增强的MAA Function Calling服务器
pub struct EnhancedMaaFunctionServer {
    maa_backend: Arc<MaaBackend>,
}

impl EnhancedMaaFunctionServer {
    pub fn new(maa_backend: Arc<MaaBackend>) -> Self {
        Self { maa_backend }
    }

    /// 获取所有16个MAA任务Function Calling工具定义
    pub fn get_function_definitions(&self) -> Vec<FunctionDefinition> {
        vec![
            // 核心游戏功能 (4个)
            self.define_startup_tool(),
            self.define_combat_tool(),
            self.define_recruit_tool(),
            self.define_infrastructure_tool(),
            
            // 高级自动化功能 (4个)
            self.define_roguelike_tool(),
            self.define_copilot_tool(),
            self.define_sss_copilot_tool(),
            self.define_reclamation_tool(),
            
            // 辅助功能 (4个)
            self.define_rewards_tool(),
            self.define_credit_store_tool(),
            self.define_depot_tool(),
            self.define_operator_box_tool(),
            
            // 系统功能 (4个)
            self.define_closedown_tool(),
            self.define_custom_task_tool(),
            self.define_video_recognition_tool(),
            self.define_system_management_tool(),
        ]
    }

    /// 执行增强的Function Call
    pub async fn execute_function(&self, call: FunctionCall) -> FunctionResponse {
        debug!("执行增强函数调用: {} with args: {:?}", call.name, call.arguments);

        let result = match call.name.as_str() {
            // 核心游戏功能
            "maa_startup" => self.handle_startup(call.arguments).await,
            "maa_combat_enhanced" => self.handle_combat_enhanced(call.arguments).await,
            "maa_recruit_enhanced" => self.handle_recruit_enhanced(call.arguments).await,
            "maa_infrastructure_enhanced" => self.handle_infrastructure_enhanced(call.arguments).await,
            
            // 高级自动化功能
            "maa_roguelike_enhanced" => self.handle_roguelike_enhanced(call.arguments).await,
            "maa_copilot_enhanced" => self.handle_copilot_enhanced(call.arguments).await,
            "maa_sss_copilot" => self.handle_sss_copilot(call.arguments).await,
            "maa_reclamation" => self.handle_reclamation(call.arguments).await,
            
            // 辅助功能
            "maa_rewards_enhanced" => self.handle_rewards_enhanced(call.arguments).await,
            "maa_credit_store_enhanced" => self.handle_credit_store_enhanced(call.arguments).await,
            "maa_depot_management" => self.handle_depot_management(call.arguments).await,
            "maa_operator_box" => self.handle_operator_box(call.arguments).await,
            
            // 系统功能
            "maa_closedown" => self.handle_closedown(call.arguments).await,
            "maa_custom_task" => self.handle_custom_task(call.arguments).await,
            "maa_video_recognition" => self.handle_video_recognition(call.arguments).await,
            "maa_system_management" => self.handle_system_management(call.arguments).await,
            
            _ => Err(format!("未知的增强函数: {}", call.name)),
        };

        match result {
            Ok(data) => FunctionResponse {
                success: true,
                result: Some(data),
                error: None,
                timestamp: chrono::Utc::now(),
            },
            Err(error) => FunctionResponse {
                success: false,
                result: None,
                error: Some(error),
                timestamp: chrono::Utc::now(),
            },
        }
    }

    // ===== 工具定义方法 =====

    fn define_startup_tool(&self) -> FunctionDefinition {
        FunctionDefinition {
            name: "maa_startup".to_string(),
            description: "管理游戏启动、账号切换、客户端选择等功能".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["start_game", "switch_account", "check_status"],
                        "description": "启动操作类型",
                        "default": "start_game"
                    },
                    "client_type": {
                        "type": "string",
                        "enum": ["Official", "Bilibili", "Txwy", "YoStarEN", "YoStarJP", "YoStarKR"],
                        "description": "客户端类型",
                        "default": "Official"
                    },
                    "account": {
                        "type": "string",
                        "description": "账号标识（支持部分匹配）"
                    },
                    "start_emulator": {
                        "type": "boolean",
                        "description": "是否尝试启动模拟器",
                        "default": true
                    },
                    "wait_timeout": {
                        "type": "number",
                        "description": "启动等待超时时间（秒）",
                        "default": 60,
                        "minimum": 10,
                        "maximum": 300
                    }
                },
                "required": ["action"]
            }),
        }
    }

    fn define_combat_tool(&self) -> FunctionDefinition {
        FunctionDefinition {
            name: "maa_combat_enhanced".to_string(),
            description: "执行自动战斗，支持复杂策略、资源管理、掉落统计等".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "stage": {
                        "type": "string",
                        "description": "关卡代码或自然语言描述（如'1-7'、'龙门币本'、'经验书关卡'）"
                    },
                    "strategy": {
                        "type": "object",
                        "properties": {
                            "mode": {
                                "type": "string",
                                "enum": ["times", "sanity", "material", "infinite"],
                                "description": "战斗策略：固定次数、消耗理智、获得材料、无限刷取",
                                "default": "times"
                            },
                            "target_value": {
                                "type": "number",
                                "description": "目标值（次数/理智/材料数量）",
                                "default": 1
                            },
                            "target_material": {
                                "type": "string",
                                "description": "目标材料名称（仅material模式）"
                            }
                        }
                    },
                    "resources": {
                        "type": "object",
                        "properties": {
                            "use_medicine": {
                                "type": "boolean",
                                "description": "是否使用理智药剂",
                                "default": false
                            },
                            "medicine_limit": {
                                "type": "number",
                                "description": "理智药剂使用上限",
                                "default": 999
                            },
                            "use_stone": {
                                "type": "boolean",
                                "description": "是否使用源石恢复理智",
                                "default": false
                            },
                            "stone_limit": {
                                "type": "number",
                                "description": "源石使用上限",
                                "default": 0
                            }
                        }
                    },
                    "automation": {
                        "type": "object",
                        "properties": {
                            "auto_agent": {
                                "type": "boolean",
                                "description": "是否自动选择代理指挥",
                                "default": true
                            },
                            "backup_stage": {
                                "type": "string",
                                "description": "代理指挥失败时的后备关卡"
                            },
                            "drop_tracking": {
                                "type": "boolean",
                                "description": "是否启用掉落统计",
                                "default": true
                            },
                            "upload_data": {
                                "type": "boolean",
                                "description": "是否上报掉落数据",
                                "default": false
                            }
                        }
                    }
                },
                "required": ["stage"]
            }),
        }
    }

    fn define_recruit_tool(&self) -> FunctionDefinition {
        FunctionDefinition {
            name: "maa_recruit_enhanced".to_string(),
            description: "智能公开招募管理，支持标签分析、策略优化、结果预测".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "mode": {
                        "type": "string",
                        "enum": ["auto", "manual", "analyze_only"],
                        "description": "招募模式：自动、手动确认、仅分析",
                        "default": "auto"
                    },
                    "strategy": {
                        "type": "object",
                        "properties": {
                            "max_times": {
                                "type": "number",
                                "description": "最大招募次数，0表示用完所有招募票",
                                "default": 0
                            },
                            "min_star": {
                                "type": "number",
                                "enum": [1, 2, 3, 4, 5, 6],
                                "description": "最低星级要求",
                                "default": 3
                            },
                            "priority_tags": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "优先标签列表",
                                "default": ["高级资深干员", "资深干员", "支援机械"]
                            },
                            "avoid_robot": {
                                "type": "boolean",
                                "description": "是否避免1星机器人",
                                "default": true
                            }
                        }
                    },
                    "resources": {
                        "type": "object",
                        "properties": {
                            "use_permit": {
                                "type": "boolean",
                                "description": "是否使用加急许可证",
                                "default": false
                            },
                            "permit_limit": {
                                "type": "number",
                                "description": "加急许可证使用上限",
                                "default": 999
                            },
                            "refresh_tags": {
                                "type": "boolean",
                                "description": "是否刷新标签",
                                "default": false
                            }
                        }
                    }
                }
            }),
        }
    }

    fn define_infrastructure_tool(&self) -> FunctionDefinition {
        FunctionDefinition {
            name: "maa_infrastructure_enhanced".to_string(),
            description: "智能基建管理，支持全设施优化、效率分析、自动排班".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "operation_mode": {
                        "type": "string",
                        "enum": ["full_auto", "collect_only", "schedule_only", "custom"],
                        "description": "操作模式：全自动、仅收菜、仅排班、自定义",
                        "default": "full_auto"
                    },
                    "facilities": {
                        "type": "object",
                        "properties": {
                            "manufacturing": {
                                "type": "object",
                                "properties": {
                                    "enabled": {"type": "boolean", "default": true},
                                    "products": {
                                        "type": "array",
                                        "items": {"type": "string"},
                                        "description": "优先生产的物品",
                                        "default": ["经验书", "赤金", "源石碎片"]
                                    }
                                }
                            },
                            "trading": {
                                "type": "object",
                                "properties": {
                                    "enabled": {"type": "boolean", "default": true},
                                    "strategy": {
                                        "type": "string",
                                        "enum": ["efficiency", "speed", "balance"],
                                        "description": "贸易策略",
                                        "default": "efficiency"
                                    }
                                }
                            },
                            "dormitory": {
                                "type": "object",
                                "properties": {
                                    "enabled": {"type": "boolean", "default": true},
                                    "mood_threshold": {
                                        "type": "number",
                                        "description": "心情阈值",
                                        "default": 12,
                                        "minimum": 0,
                                        "maximum": 24
                                    }
                                }
                            }
                        }
                    },
                    "automation": {
                        "type": "object",
                        "properties": {
                            "use_drone": {
                                "type": "boolean",
                                "description": "是否使用无人机",
                                "default": false
                            },
                            "drone_facility": {
                                "type": "string",
                                "enum": ["manufacturing", "trading"],
                                "description": "无人机优先使用的设施",
                                "default": "manufacturing"
                            },
                            "auto_shift": {
                                "type": "boolean",
                                "description": "是否自动换班",
                                "default": true
                            }
                        }
                    }
                }
            }),
        }
    }

    fn define_roguelike_tool(&self) -> FunctionDefinition {
        FunctionDefinition {
            name: "maa_roguelike_enhanced".to_string(),
            description: "智能集成战略管理，支持策略选择、投资优化、风险控制".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "theme_config": {
                        "type": "object",
                        "properties": {
                            "theme": {
                                "type": "string",
                                "enum": ["phantom", "mizuki", "sami", "sarkaz", "jiegarden", "latest"],
                                "description": "主题选择",
                                "default": "latest"
                            },
                            "difficulty": {
                                "type": "number",
                                "description": "难度等级",
                                "minimum": 3,
                                "maximum": 10,
                                "default": 5
                            },
                            "adaptive_difficulty": {
                                "type": "boolean",
                                "description": "是否根据成功率自动调整难度",
                                "default": false
                            }
                        }
                    },
                    "squad_strategy": {
                        "type": "object",
                        "properties": {
                            "squad_type": {
                                "type": "string",
                                "enum": ["command", "assault", "remote", "adaptive"],
                                "description": "分队类型，adaptive表示根据主题自动选择",
                                "default": "adaptive"
                            },
                            "starting_operator": {
                                "type": "string",
                                "description": "起始干员名称，空表示自动选择"
                            },
                            "use_friend_support": {
                                "type": "boolean",
                                "description": "是否使用好友助战",
                                "default": true
                            }
                        }
                    },
                    "target_config": {
                        "type": "object",
                        "properties": {
                            "target_runs": {
                                "type": "number",
                                "description": "目标运行次数，0表示无限制",
                                "default": 1
                            },
                            "stop_on_success": {
                                "type": "boolean",
                                "description": "成功通关后是否停止",
                                "default": false
                            }
                        }
                    }
                }
            }),
        }
    }

    fn define_copilot_tool(&self) -> FunctionDefinition {
        FunctionDefinition {
            name: "maa_copilot_enhanced".to_string(),
            description: "智能作业（自动战斗脚本）执行，支持作业匹配、参数优化、成功率分析".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "copilot_source": {
                        "type": "object",
                        "properties": {
                            "source_type": {
                                "type": "string",
                                "enum": ["uri", "file", "search", "auto"],
                                "description": "作业来源类型",
                                "default": "auto"
                            },
                            "uri_list": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "作业URI列表，支持maa://格式"
                            },
                            "search_stage": {
                                "type": "string",
                                "description": "搜索关卡名称（search模式）"
                            },
                            "difficulty_preference": {
                                "type": "string",
                                "enum": ["easy", "normal", "hard", "auto"],
                                "description": "难度偏好",
                                "default": "auto"
                            }
                        }
                    },
                    "execution_config": {
                        "type": "object",
                        "properties": {
                            "formation": {
                                "type": "boolean",
                                "description": "是否自动编队",
                                "default": true
                            },
                            "use_sanity_potion": {
                                "type": "boolean",
                                "description": "是否使用理智药剂",
                                "default": false
                            },
                            "loop_times": {
                                "type": "number",
                                "description": "循环执行次数",
                                "default": 1
                            },
                            "support_unit_name": {
                                "type": "string",
                                "description": "支援单位名称"
                            }
                        }
                    },
                    "optimization": {
                        "type": "object",
                        "properties": {
                            "auto_retry": {
                                "type": "boolean",
                                "description": "失败时是否自动重试",
                                "default": true
                            },
                            "max_retries": {
                                "type": "number",
                                "description": "最大重试次数",
                                "default": 3
                            },
                            "operator_matching": {
                                "type": "string",
                                "enum": ["strict", "flexible", "adaptive"],
                                "description": "干员匹配策略",
                                "default": "adaptive"
                            }
                        }
                    }
                },
                "required": ["copilot_source"]
            }),
        }
    }

    fn define_sss_copilot_tool(&self) -> FunctionDefinition {
        FunctionDefinition {
            name: "maa_sss_copilot".to_string(),
            description: "保全派驻作业执行，支持特殊作业和高难度挑战".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "copilot_config": {
                        "type": "object",
                        "description": "保全派驻作业配置"
                    },
                    "difficulty": {
                        "type": "string",
                        "enum": ["normal", "hard", "nightmare"],
                        "description": "难度选择",
                        "default": "normal"
                    }
                },
                "required": ["copilot_config"]
            }),
        }
    }

    fn define_reclamation_tool(&self) -> FunctionDefinition {
        FunctionDefinition {
            name: "maa_reclamation".to_string(),
            description: "生息演算（沙中之火）自动化，支持道具制作和繁荣度管理".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "mode": {
                        "type": "string",
                        "enum": ["prosperity", "craft_tools"],
                        "description": "模式：刷繁荣度或制作道具",
                        "default": "prosperity"
                    },
                    "tools_to_craft": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "制作道具列表"
                    },
                    "craft_config": {
                        "type": "object",
                        "properties": {
                            "increase_mode": {
                                "type": "string",
                                "enum": ["click", "long_press"],
                                "description": "增加数量方式",
                                "default": "click"
                            },
                            "num_craft_batches": {
                                "type": "number",
                                "description": "每次制作批次数量",
                                "default": 1
                            }
                        }
                    }
                }
            }),
        }
    }

    fn define_rewards_tool(&self) -> FunctionDefinition {
        FunctionDefinition {
            name: "maa_rewards_enhanced".to_string(),
            description: "全自动奖励收集，支持智能过滤、批量处理、价值评估".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "collection_scope": {
                        "type": "array",
                        "items": {
                            "type": "string",
                            "enum": ["daily", "weekly", "mail", "mission", "event", "sign_in", "all"]
                        },
                        "description": "收集范围，all表示所有类型",
                        "default": ["all"]
                    },
                    "filters": {
                        "type": "object",
                        "properties": {
                            "priority_items": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "优先收集的物品关键词",
                                "default": ["源石", "寻访凭证", "理智药剂"]
                            },
                            "blacklist": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "黑名单物品关键词",
                                "default": ["家具", "装潢"]
                            }
                        }
                    },
                    "mail_management": {
                        "type": "object",
                        "properties": {
                            "auto_collect_attachments": {
                                "type": "boolean",
                                "description": "是否自动收取邮件附件",
                                "default": true
                            },
                            "auto_delete_empty": {
                                "type": "boolean",
                                "description": "是否自动删除空邮件",
                                "default": false
                            }
                        }
                    }
                }
            }),
        }
    }

    fn define_credit_store_tool(&self) -> FunctionDefinition {
        FunctionDefinition {
            name: "maa_credit_store_enhanced".to_string(),
            description: "智能信用商店管理，支持价值分析、策略购买、库存管理".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "operation_mode": {
                        "type": "string",
                        "enum": ["earn_only", "shop_only", "full_auto", "analyze_only"],
                        "description": "操作模式：仅获取信用、仅购买、全自动、仅分析",
                        "default": "full_auto"
                    },
                    "earning_strategy": {
                        "type": "object",
                        "properties": {
                            "visit_friends": {
                                "type": "boolean",
                                "description": "是否访问好友",
                                "default": true
                            },
                            "clear_of1": {
                                "type": "boolean",
                                "description": "是否通关OF-1",
                                "default": true
                            }
                        }
                    },
                    "purchase_strategy": {
                        "type": "object",
                        "properties": {
                            "strategy_type": {
                                "type": "string",
                                "enum": ["priority_list", "value_based", "balanced", "conservative"],
                                "description": "购买策略类型",
                                "default": "priority_list"
                            },
                            "priority_items": {
                                "type": "string",
                                "description": "优先购买物品关键词，用分号分隔",
                                "default": "碳;技能概要;芯片;龙门币"
                            },
                            "credit_threshold": {
                                "type": "number",
                                "description": "信用点保留阈值",
                                "default": 300
                            }
                        }
                    }
                }
            }),
        }
    }

    fn define_depot_tool(&self) -> FunctionDefinition {
        FunctionDefinition {
            name: "maa_depot_management".to_string(),
            description: "仓库整理和物品管理".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["scan", "organize", "report"],
                        "description": "操作类型：扫描、整理、报告",
                        "default": "scan"
                    }
                }
            }),
        }
    }

    fn define_operator_box_tool(&self) -> FunctionDefinition {
        FunctionDefinition {
            name: "maa_operator_box".to_string(),
            description: "干员管理，包括等级、技能、模组等信息".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["scan", "analyze", "recommend"],
                        "description": "操作类型：扫描、分析、推荐",
                        "default": "scan"
                    }
                }
            }),
        }
    }

    fn define_closedown_tool(&self) -> FunctionDefinition {
        FunctionDefinition {
            name: "maa_closedown".to_string(),
            description: "关闭游戏客户端".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "force": {
                        "type": "boolean",
                        "description": "是否强制关闭",
                        "default": false
                    }
                }
            }),
        }
    }

    fn define_custom_task_tool(&self) -> FunctionDefinition {
        FunctionDefinition {
            name: "maa_custom_task".to_string(),
            description: "执行自定义任务".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "task_config": {
                        "type": "object",
                        "description": "自定义任务配置"
                    }
                },
                "required": ["task_config"]
            }),
        }
    }

    fn define_video_recognition_tool(&self) -> FunctionDefinition {
        FunctionDefinition {
            name: "maa_video_recognition".to_string(),
            description: "视频识别和分析".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "video_path": {
                        "type": "string",
                        "description": "视频文件路径"
                    }
                },
                "required": ["video_path"]
            }),
        }
    }

    fn define_system_management_tool(&self) -> FunctionDefinition {
        FunctionDefinition {
            name: "maa_system_management".to_string(),
            description: "MAA系统管理，支持状态监控、配置管理、性能优化".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["status", "config", "optimize", "cleanup", "restart", "backup"],
                        "description": "系统操作类型"
                    },
                    "detailed": {
                        "type": "boolean",
                        "description": "是否返回详细信息",
                        "default": false
                    }
                },
                "required": ["operation"]
            }),
        }
    }

    // ===== 处理方法实现 =====

    async fn handle_startup(&self, args: Value) -> Result<Value, String> {
        info!("处理StartUp任务请求: {:?}", args);

        // 创建StartUp任务执行器
        let startup_task = MaaStartUpTask::new(self.maa_backend.clone());

        // 解析参数
        let params = MaaStartUpTask::parse_arguments(&args)
            .map_err(|e| format!("参数解析失败: {}", e))?;

        // 执行StartUp任务
        let response = startup_task.execute(params).await
            .map_err(|e| format!("StartUp任务执行失败: {}", e))?;

        // 返回执行结果
        if response.success {
            Ok(response.result.unwrap_or_else(|| json!({"status": "success"})))
        } else {
            Err(response.error.unwrap_or_else(|| "未知错误".to_string()))
        }
    }

    async fn handle_combat_enhanced(&self, args: Value) -> Result<Value, String> {
        info!("处理Combat Enhanced任务请求: {:?}", args);

        // 创建Combat任务执行器
        let combat_task = MaaCombatTask::new(self.maa_backend.clone());

        // 解析参数
        let params = MaaCombatTask::parse_arguments(&args)
            .map_err(|e| format!("参数解析失败: {}", e))?;

        // 执行Combat任务
        let response = combat_task.execute(params).await
            .map_err(|e| format!("Combat任务执行失败: {}", e))?;

        // 返回执行结果
        if response.success {
            Ok(response.result.unwrap_or_else(|| json!({"status": "success"})))
        } else {
            Err(response.error.unwrap_or_else(|| "未知错误".to_string()))
        }
    }

    async fn handle_recruit_enhanced(&self, args: Value) -> Result<Value, String> {
        info!("处理增强招募任务调用: {:?}", args);
        
        let recruit_task = MaaRecruitTask::new(Arc::clone(&self.maa_backend));
        
        // 解析参数
        let params = MaaRecruitTask::parse_arguments(&args)
            .map_err(|e| format!("招募参数解析失败: {}", e))?;
        
        // 执行任务
        let response = recruit_task.execute(params).await
            .map_err(|e| format!("招募任务执行失败: {}", e))?;
        
        if response.success {
            Ok(response.result.unwrap_or_else(|| json!({"status": "success"})))
        } else {
            Err(response.error.unwrap_or_else(|| "未知错误".to_string()))
        }
    }

    async fn handle_infrastructure_enhanced(&self, args: Value) -> Result<Value, String> {
        let task = MaaInfrastructureTask::new(Arc::clone(&self.maa_backend));
        let params = MaaInfrastructureTask::parse_arguments(&args)
            .map_err(|e| format!("基建参数解析失败: {}", e))?;
        let response = task.execute(params).await
            .map_err(|e| format!("基建任务执行失败: {}", e))?;
        
        if response.success {
            Ok(response.result.unwrap_or_else(|| json!({"status": "success"})))
        } else {
            Err(response.error.unwrap_or_else(|| "未知错误".to_string()))
        }
    }

    async fn handle_roguelike_enhanced(&self, args: Value) -> Result<Value, String> {
        let task = MaaRoguelikeTask::new(Arc::clone(&self.maa_backend));
        let params = MaaRoguelikeTask::parse_arguments(&args)
            .map_err(|e| format!("肉鸽参数解析失败: {}", e))?;
        let response = task.execute(params).await
            .map_err(|e| format!("肉鸽任务执行失败: {}", e))?;
        
        if response.success {
            Ok(response.result.unwrap_or_else(|| json!({"status": "success"})))
        } else {
            Err(response.error.unwrap_or_else(|| "未知错误".to_string()))
        }
    }

    async fn handle_copilot_enhanced(&self, args: Value) -> Result<Value, String> {
        let task = MaaCopilotTask::new(Arc::clone(&self.maa_backend));
        let params = MaaCopilotTask::parse_arguments(&args)
            .map_err(|e| format!("作业参数解析失败: {}", e))?;
        let response = task.execute(params).await
            .map_err(|e| format!("作业任务执行失败: {}", e))?;
        
        if response.success {
            Ok(response.result.unwrap_or_else(|| json!({"status": "success"})))
        } else {
            Err(response.error.unwrap_or_else(|| "未知错误".to_string()))
        }
    }

    async fn handle_sss_copilot(&self, args: Value) -> Result<Value, String> {
        let task = MaaSSSCopilotTask::new(Arc::clone(&self.maa_backend));
        let params = MaaSSSCopilotTask::parse_arguments(&args)
            .map_err(|e| format!("保全派驻参数解析失败: {}", e))?;
        let response = task.execute(params).await
            .map_err(|e| format!("保全派驻任务执行失败: {}", e))?;
        
        if response.success {
            Ok(response.result.unwrap_or_else(|| json!({"status": "success"})))
        } else {
            Err(response.error.unwrap_or_else(|| "未知错误".to_string()))
        }
    }

    async fn handle_reclamation(&self, args: Value) -> Result<Value, String> {
        let task = MaaReclamationTask::new(Arc::clone(&self.maa_backend));
        let params = MaaReclamationTask::parse_arguments(&args)
            .map_err(|e| format!("生息演算参数解析失败: {}", e))?;
        let response = task.execute(params).await
            .map_err(|e| format!("生息演算任务执行失败: {}", e))?;
        
        if response.success {
            Ok(response.result.unwrap_or_else(|| json!({"status": "success"})))
        } else {
            Err(response.error.unwrap_or_else(|| "未知错误".to_string()))
        }
    }

    async fn handle_rewards_enhanced(&self, args: Value) -> Result<Value, String> {
        let task = MaaRewardsTask::new(Arc::clone(&self.maa_backend));
        let params = MaaRewardsTask::parse_arguments(&args)
            .map_err(|e| format!("奖励收集参数解析失败: {}", e))?;
        let response = task.execute(params).await
            .map_err(|e| format!("奖励收集任务执行失败: {}", e))?;
        
        if response.success {
            Ok(response.result.unwrap_or_else(|| json!({"status": "success"})))
        } else {
            Err(response.error.unwrap_or_else(|| "未知错误".to_string()))
        }
    }

    async fn handle_credit_store_enhanced(&self, args: Value) -> Result<Value, String> {
        let task = MaaCreditStoreTask::new(Arc::clone(&self.maa_backend));
        let params = MaaCreditStoreTask::parse_arguments(&args)
            .map_err(|e| format!("信用商店参数解析失败: {}", e))?;
        let response = task.execute(params).await
            .map_err(|e| format!("信用商店任务执行失败: {}", e))?;
        
        if response.success {
            Ok(response.result.unwrap_or_else(|| json!({"status": "success"})))
        } else {
            Err(response.error.unwrap_or_else(|| "未知错误".to_string()))
        }
    }

    async fn handle_depot_management(&self, args: Value) -> Result<Value, String> {
        let task = MaaDepotTask::new(Arc::clone(&self.maa_backend));
        let params = MaaDepotTask::parse_arguments(&args)
            .map_err(|e| format!("仓库管理参数解析失败: {}", e))?;
        let response = task.execute(params).await
            .map_err(|e| format!("仓库管理任务执行失败: {}", e))?;
        
        if response.success {
            Ok(response.result.unwrap_or_else(|| json!({"status": "success"})))
        } else {
            Err(response.error.unwrap_or_else(|| "未知错误".to_string()))
        }
    }

    async fn handle_operator_box(&self, args: Value) -> Result<Value, String> {
        let task = MaaOperatorBoxTask::new(Arc::clone(&self.maa_backend));
        let params = MaaOperatorBoxTask::parse_arguments(&args)
            .map_err(|e| format!("干员管理参数解析失败: {}", e))?;
        let response = task.execute(params).await
            .map_err(|e| format!("干员管理任务执行失败: {}", e))?;
        
        if response.success {
            Ok(response.result.unwrap_or_else(|| json!({"status": "success"})))
        } else {
            Err(response.error.unwrap_or_else(|| "未知错误".to_string()))
        }
    }

    async fn handle_closedown(&self, args: Value) -> Result<Value, String> {
        let task = MaaCloseDownTask::new(Arc::clone(&self.maa_backend));
        let params = MaaCloseDownTask::parse_arguments(&args)
            .map_err(|e| format!("游戏关闭参数解析失败: {}", e))?;
        let response = task.execute(params).await
            .map_err(|e| format!("游戏关闭任务执行失败: {}", e))?;
        
        if response.success {
            Ok(response.result.unwrap_or_else(|| json!({"status": "success"})))
        } else {
            Err(response.error.unwrap_or_else(|| "未知错误".to_string()))
        }
    }

    async fn handle_custom_task(&self, args: Value) -> Result<Value, String> {
        let task = MaaCustomTaskTask::new(Arc::clone(&self.maa_backend));
        let params = MaaCustomTaskTask::parse_arguments(&args)
            .map_err(|e| format!("自定义任务参数解析失败: {}", e))?;
        let response = task.execute(params).await
            .map_err(|e| format!("自定义任务执行失败: {}", e))?;
        
        if response.success {
            Ok(response.result.unwrap_or_else(|| json!({"status": "success"})))
        } else {
            Err(response.error.unwrap_or_else(|| "未知错误".to_string()))
        }
    }

    async fn handle_video_recognition(&self, args: Value) -> Result<Value, String> {
        let task = MaaVideoRecognitionTask::new(Arc::clone(&self.maa_backend));
        let params = MaaVideoRecognitionTask::parse_arguments(&args)
            .map_err(|e| format!("视频识别参数解析失败: {}", e))?;
        let response = task.execute(params).await
            .map_err(|e| format!("视频识别任务执行失败: {}", e))?;
        
        if response.success {
            Ok(response.result.unwrap_or_else(|| json!({"status": "success"})))
        } else {
            Err(response.error.unwrap_or_else(|| "未知错误".to_string()))
        }
    }

    async fn handle_system_management(&self, args: Value) -> Result<Value, String> {
        let task = MaaSystemManagementTask::new(Arc::clone(&self.maa_backend));
        let params = MaaSystemManagementTask::parse_arguments(&args)
            .map_err(|e| format!("系统管理参数解析失败: {}", e))?;
        let response = task.execute(params).await
            .map_err(|e| format!("系统管理任务执行失败: {}", e))?;
        
        if response.success {
            Ok(response.result.unwrap_or_else(|| json!({"status": "success"})))
        } else {
            Err(response.error.unwrap_or_else(|| "未知错误".to_string()))
        }
    }
}

/// 创建增强Function Calling服务器
pub fn create_enhanced_function_server(maa_backend: Arc<MaaBackend>) -> EnhancedMaaFunctionServer {
    info!("创建增强MAA Function Calling服务器，支持16种任务类型");
    EnhancedMaaFunctionServer::new(maa_backend)
}