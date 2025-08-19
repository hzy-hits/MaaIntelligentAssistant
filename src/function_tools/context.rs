//! MAA 任务上下文管理
//!
//! 提供工具间状态共享和智能任务链推荐功能

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::Utc;
use serde_json::Value;
use super::types::{TaskContext, GameState};

/// 全局任务上下文管理器
pub struct ContextManager {
    contexts: Arc<Mutex<HashMap<String, TaskContext>>>,
}

impl ContextManager {
    /// 创建新的上下文管理器
    pub fn new() -> Self {
        Self {
            contexts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 获取或创建用户上下文
    pub fn get_or_create_context(&self, user_id: &str) -> TaskContext {
        let mut contexts = self.contexts.lock().unwrap();
        contexts.entry(user_id.to_string())
            .or_insert_with(|| TaskContext {
                user_id: Some(user_id.to_string()),
                session_id: Some(format!("session_{}", Utc::now().timestamp())),
                game_state: GameState::default(),
                last_operations: Vec::new(),
                recommendations: Vec::new(),
            })
            .clone()
    }

    /// 更新用户上下文
    pub fn update_context(&self, user_id: &str, context: TaskContext) {
        let mut contexts = self.contexts.lock().unwrap();
        contexts.insert(user_id.to_string(), context);
    }

    /// 记录操作到历史
    pub fn record_operation(&self, user_id: &str, operation: &str, result: &Value) {
        let mut context = self.get_or_create_context(user_id);
        context.last_operations.push(format!("{}: {:?}", operation, result));
        
        // 只保留最近的10个操作
        if context.last_operations.len() > 10 {
            context.last_operations.remove(0);
        }
        
        self.update_context(user_id, context);
    }

    /// 基于上下文生成任务推荐
    pub fn generate_recommendations(&self, user_id: &str, current_operation: &str) -> Vec<String> {
        let context = self.get_or_create_context(user_id);
        let mut recommendations = Vec::new();

        match current_operation {
            "maa_startup" => {
                recommendations.extend(vec![
                    "建议接下来执行 maa_rewards_enhanced 收集每日奖励".to_string(),
                    "可以执行 maa_infrastructure_enhanced 进行基建管理".to_string(),
                    "如果需要刷图，可以执行 maa_combat_enhanced".to_string(),
                ]);
            },
            "maa_combat_enhanced" => {
                if context.game_state.current_sanity.unwrap_or(0) < 20 {
                    recommendations.push("理智不足，建议使用理智药或等待恢复".to_string());
                }
                recommendations.extend(vec![
                    "战斗完成后可以执行基建收集".to_string(),
                    "可以考虑进行公开招募".to_string(),
                ]);
            },
            "maa_infrastructure_enhanced" => {
                recommendations.extend(vec![
                    "基建收集完成，可以进行刷图任务".to_string(),
                    "检查是否有公开招募可以进行".to_string(),
                ]);
            },
            "maa_recruit_enhanced" => {
                recommendations.extend(vec![
                    "招募完成，建议进行日常刷图".to_string(),
                    "可以检查信用商店是否有物品需要购买".to_string(),
                ]);
            },
            _ => {
                recommendations.push("可以根据当前需要选择相应的任务".to_string());
            }
        }

        // 基于历史操作调整推荐
        if context.last_operations.iter().any(|op| op.contains("combat")) {
            recommendations.push("今日已进行过战斗，可以关注其他日常任务".to_string());
        }

        recommendations
    }

    /// 生成任务链建议
    pub fn suggest_task_chain(&self, _user_id: &str, goal: &str) -> Vec<String> {
        match goal {
            "日常任务" => vec![
                "maa_startup".to_string(),
                "maa_rewards_enhanced".to_string(),
                "maa_infrastructure_enhanced".to_string(),
                "maa_recruit_enhanced".to_string(),
                "maa_combat_enhanced".to_string(),
            ],
            "快速升级" => vec![
                "maa_startup".to_string(),
                "maa_combat_enhanced(stage=1-7)".to_string(),
                "maa_infrastructure_enhanced".to_string(),
            ],
            "材料收集" => vec![
                "maa_startup".to_string(),
                "maa_combat_enhanced(根据目标材料选择关卡)".to_string(),
                "maa_infrastructure_enhanced".to_string(),
            ],
            "肉鸽推进" => vec![
                "maa_startup".to_string(),
                "maa_roguelike_enhanced".to_string(),
            ],
            _ => vec![
                "maa_startup".to_string(),
                "根据具体需求选择后续任务".to_string(),
            ],
        }
    }

    /// 更新游戏状态
    pub fn update_game_state(&self, user_id: &str, sanity: Option<i32>, medicine: Option<i32>, stone: Option<i32>) {
        let mut context = self.get_or_create_context(user_id);
        
        if let Some(s) = sanity {
            context.game_state.current_sanity = Some(s);
        }
        if let Some(m) = medicine {
            context.game_state.medicine_count = Some(m);
        }
        if let Some(st) = stone {
            context.game_state.stone_count = Some(st);
        }
        
        self.update_context(user_id, context);
    }

    /// 检查是否应该提醒用户
    pub fn check_reminders(&self, user_id: &str) -> Vec<String> {
        let context = self.get_or_create_context(user_id);
        let mut reminders = Vec::new();

        // 检查理智是否接近满值
        if let (Some(current), Some(max)) = (context.game_state.current_sanity, context.game_state.max_sanity) {
            if current >= max - 10 {
                reminders.push("理智即将满值，建议及时使用".to_string());
            }
        }

        // 检查是否长时间未登录
        if let Some(last_login) = context.game_state.last_login {
            let hours_since_login = (Utc::now() - last_login).num_hours();
            if hours_since_login > 12 {
                reminders.push("长时间未登录，建议检查基建收益和理智恢复".to_string());
            }
        }

        // 检查招募票是否较多
        if let Some(tickets) = context.game_state.recruit_tickets {
            if tickets > 50 {
                reminders.push("招募票较多，建议进行公开招募".to_string());
            }
        }

        reminders
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
}

use once_cell::sync::Lazy;

/// 全局上下文管理器实例
pub static GLOBAL_CONTEXT: Lazy<ContextManager> = Lazy::new(|| ContextManager::new());

/// 便捷函数：记录操作
pub fn record_operation(user_id: &str, operation: &str, result: &Value) {
    GLOBAL_CONTEXT.record_operation(user_id, operation, result);
}

/// 便捷函数：生成推荐
pub fn generate_recommendations(user_id: &str, current_operation: &str) -> Vec<String> {
    GLOBAL_CONTEXT.generate_recommendations(user_id, current_operation)
}

/// 便捷函数：建议任务链
pub fn suggest_task_chain(user_id: &str, goal: &str) -> Vec<String> {
    GLOBAL_CONTEXT.suggest_task_chain(user_id, goal)
}

/// 便捷函数：更新游戏状态
pub fn update_game_state(user_id: &str, sanity: Option<i32>, medicine: Option<i32>, stone: Option<i32>) {
    GLOBAL_CONTEXT.update_game_state(user_id, sanity, medicine, stone);
}

/// 便捷函数：检查提醒
pub fn check_reminders(user_id: &str) -> Vec<String> {
    GLOBAL_CONTEXT.check_reminders(user_id)
}