//! 新的任务分类系统
//! 
//! 基于实际使用模式重新设计任务分类：
//! - 同步任务：立即执行并返回结果 (startup, closedown, screenshot)
//! - 异步任务：长时间运行，需要通过SSE推送结果给前端

use serde::{Serialize, Deserialize};

/// 任务执行模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskExecutionMode {
    /// 同步任务：立即执行并返回结果
    /// 适用于: maa_startup, maa_closedown, maa_take_screenshot
    Synchronous,
    
    /// 异步任务：长时间运行，需要SSE推送结果
    /// 适用于: 战斗、招募、基建等所有游戏操作
    Asynchronous,
}

/// 任务优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    /// 普通优先级：异步游戏任务
    Normal = 1,
    /// 高优先级：同步即时任务
    High = 10,
}

/// 任务分类器 - 根据Function Call名称确定执行模式和优先级
pub fn classify_task(function_name: &str) -> (TaskExecutionMode, TaskPriority) {
    match function_name {
        // 同步高优先级任务
        "maa_startup" | "maa_closedown" | "maa_take_screenshot" => {
            (TaskExecutionMode::Synchronous, TaskPriority::High)
        },
        
        // 异步普通优先级任务 - 核心游戏功能 (需要长时间运行)
        "maa_combat_enhanced" | "maa_recruit_enhanced" | "maa_infrastructure_enhanced" => {
            (TaskExecutionMode::Asynchronous, TaskPriority::Normal)
        },
        
        // 异步普通优先级任务 - 高级自动化 (需要长时间运行)
        "maa_roguelike_enhanced" | "maa_copilot_enhanced" | "maa_sss_copilot" | "maa_reclamation" => {
            (TaskExecutionMode::Asynchronous, TaskPriority::Normal)
        },
        
        // 异步普通优先级任务 - 辅助功能 (需要长时间运行)
        "maa_rewards_enhanced" | "maa_credit_store_enhanced" | "maa_depot_management" | "maa_operator_box" => {
            (TaskExecutionMode::Asynchronous, TaskPriority::Normal)
        },
        
        // 异步普通优先级任务 - 系统功能 (需要长时间运行)
        "maa_custom_task" | "maa_video_recognition" | "maa_system_management" => {
            (TaskExecutionMode::Asynchronous, TaskPriority::Normal)
        },
        
        // 默认为异步普通优先级
        _ => (TaskExecutionMode::Asynchronous, TaskPriority::Normal),
    }
}

/// 获取任务执行模式
pub fn get_task_execution_mode(function_name: &str) -> TaskExecutionMode {
    classify_task(function_name).0
}

/// 获取任务优先级
pub fn get_task_priority(function_name: &str) -> TaskPriority {
    classify_task(function_name).1
}

/// 判断是否为同步任务
pub fn is_synchronous_task(function_name: &str) -> bool {
    matches!(get_task_execution_mode(function_name), TaskExecutionMode::Synchronous)
}

/// 判断是否为异步任务
pub fn is_asynchronous_task(function_name: &str) -> bool {
    matches!(get_task_execution_mode(function_name), TaskExecutionMode::Asynchronous)
}

/// 估算任务持续时间（秒）
pub fn estimate_task_duration(function_name: &str) -> u32 {
    match function_name {
        // 同步任务 - 快速执行
        "maa_startup" => 60,           // 启动需要1分钟
        "maa_closedown" => 10,         // 关闭需要10秒
        "maa_take_screenshot" => 3,    // 截图需要3秒
        
        // 异步任务 - 长时间运行
        "maa_combat_enhanced" => 600,         // 战斗10分钟
        "maa_recruit_enhanced" => 300,        // 招募5分钟
        "maa_infrastructure_enhanced" => 480, // 基建8分钟
        "maa_roguelike_enhanced" => 1800,     // 肉鸽30分钟
        "maa_copilot_enhanced" => 900,        // 作业15分钟
        "maa_sss_copilot" => 1200,            // 保全20分钟
        "maa_reclamation" => 600,             // 生息10分钟
        "maa_rewards_enhanced" => 180,        // 奖励3分钟
        "maa_credit_store_enhanced" => 120,   // 信用商店2分钟
        "maa_depot_management" => 300,        // 仓库管理5分钟
        "maa_operator_box" => 60,             // 干员管理1分钟
        "maa_custom_task" => 300,             // 自定义任务5分钟
        "maa_video_recognition" => 600,       // 视频识别10分钟
        "maa_system_management" => 60,        // 系统管理1分钟
        
        _ => 300, // 默认5分钟
    }
}

/// 获取任务类型描述
pub fn get_task_type_description(function_name: &str) -> &'static str {
    match function_name {
        "maa_startup" => "游戏启动",
        "maa_closedown" => "游戏关闭", 
        "maa_take_screenshot" => "游戏截图",
        "maa_combat_enhanced" => "自动战斗",
        "maa_recruit_enhanced" => "公开招募",
        "maa_infrastructure_enhanced" => "基建管理",
        "maa_roguelike_enhanced" => "集成战略",
        "maa_copilot_enhanced" => "作业执行",
        "maa_sss_copilot" => "保全派驻",
        "maa_reclamation" => "生息演算",
        "maa_rewards_enhanced" => "奖励收集",
        "maa_credit_store_enhanced" => "信用商店",
        "maa_depot_management" => "仓库管理",
        "maa_operator_box" => "干员管理",
        "maa_custom_task" => "自定义任务",
        "maa_video_recognition" => "视频识别",
        "maa_system_management" => "系统管理",
        _ => "未知任务",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synchronous_tasks() {
        // 测试同步任务分类
        assert_eq!(get_task_execution_mode("maa_startup"), TaskExecutionMode::Synchronous);
        assert_eq!(get_task_execution_mode("maa_closedown"), TaskExecutionMode::Synchronous);
        assert_eq!(get_task_execution_mode("maa_take_screenshot"), TaskExecutionMode::Synchronous);
        
        assert_eq!(get_task_priority("maa_startup"), TaskPriority::High);
        assert_eq!(get_task_priority("maa_closedown"), TaskPriority::High);
        assert_eq!(get_task_priority("maa_take_screenshot"), TaskPriority::High);
    }

    #[test]
    fn test_asynchronous_tasks() {
        // 测试异步任务分类
        assert_eq!(get_task_execution_mode("maa_combat_enhanced"), TaskExecutionMode::Asynchronous);
        assert_eq!(get_task_execution_mode("maa_recruit_enhanced"), TaskExecutionMode::Asynchronous);
        assert_eq!(get_task_execution_mode("maa_roguelike_enhanced"), TaskExecutionMode::Asynchronous);
        
        assert_eq!(get_task_priority("maa_combat_enhanced"), TaskPriority::Normal);
        assert_eq!(get_task_priority("maa_recruit_enhanced"), TaskPriority::Normal);
        assert_eq!(get_task_priority("maa_roguelike_enhanced"), TaskPriority::Normal);
    }

    #[test]
    fn test_task_duration_estimates() {
        // 测试时间估算
        assert_eq!(estimate_task_duration("maa_take_screenshot"), 3);
        assert_eq!(estimate_task_duration("maa_startup"), 60);
        assert_eq!(estimate_task_duration("maa_combat_enhanced"), 600);
        assert_eq!(estimate_task_duration("maa_roguelike_enhanced"), 1800);
    }

    #[test]
    fn test_convenience_functions() {
        assert!(is_synchronous_task("maa_startup"));
        assert!(!is_synchronous_task("maa_combat_enhanced"));
        
        assert!(is_asynchronous_task("maa_combat_enhanced"));
        assert!(!is_asynchronous_task("maa_startup"));
    }
}