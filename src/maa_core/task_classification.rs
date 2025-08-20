//! MAA任务分类模块
//! 
//! 将MAA任务分为同步和异步两类，采用不同的处理策略

/// MAA任务执行模式
#[derive(Debug, Clone, PartialEq)]
pub enum TaskExecutionMode {
    /// 同步执行：立即等待完成并返回结果（适用于快速任务）
    Synchronous,
    /// 异步执行：立即返回任务ID，后台执行，通过事件通知完成（适用于长时间任务）
    Asynchronous,
}

/// 根据函数名称判断任务执行模式
pub fn get_task_execution_mode(function_name: &str) -> TaskExecutionMode {
    match function_name {
        // 同步任务：即时响应，快速完成（<10秒）
        "maa_take_screenshot" | 
        "maa_click" | 
        "maa_swipe" | 
        "maa_get_status" |
        "maa_get_image" |
        "maa_system_management" => TaskExecutionMode::Synchronous,
        
        // 异步任务：长时间执行（>30秒）
        "maa_startup" |
        "maa_combat_enhanced" |
        "maa_recruit_enhanced" |
        "maa_infrastructure_enhanced" |
        "maa_roguelike_enhanced" |
        "maa_copilot_enhanced" |
        "maa_sss_copilot" |
        "maa_reclamation" |
        "maa_rewards_enhanced" |
        "maa_credit_store_enhanced" |
        "maa_depot_management" |
        "maa_operator_box" |
        "maa_closedown" |
        "maa_custom_task" |
        "maa_video_recognition" => TaskExecutionMode::Asynchronous,
        
        // 默认：同步模式
        _ => TaskExecutionMode::Synchronous,
    }
}

/// 根据任务类型估算执行时间
pub fn estimate_task_duration(function_name: &str, params: &serde_json::Value) -> String {
    match function_name {
        "maa_startup" => "30-60秒（启动游戏）".to_string(),
        "maa_combat_enhanced" => {
            let times = params.get("times").and_then(|v| v.as_i64()).unwrap_or(1);
            format!("{}次战斗，约{}-{}分钟", times, times * 2, times * 5)
        },
        "maa_recruit_enhanced" => {
            let times = params.get("max_times").and_then(|v| v.as_i64()).unwrap_or(1);
            format!("{}次招募，约{}-{}分钟", times, times * 2, times * 3)
        },
        "maa_infrastructure_enhanced" => "5-10分钟（基建收取+换班）".to_string(),
        "maa_roguelike_enhanced" => "30-60分钟（集成战略）".to_string(),
        "maa_copilot_enhanced" => "5-15分钟（作业执行）".to_string(),
        "maa_rewards_enhanced" => "2-5分钟（收取奖励）".to_string(),
        "maa_depot_management" => "3-8分钟（仓库整理）".to_string(),
        "maa_take_screenshot" => "即时".to_string(),
        _ => "1-3分钟".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_task_classification() {
        // 同步任务
        assert_eq!(get_task_execution_mode("maa_take_screenshot"), TaskExecutionMode::Synchronous);
        assert_eq!(get_task_execution_mode("maa_system_management"), TaskExecutionMode::Synchronous);
        
        // 异步任务
        assert_eq!(get_task_execution_mode("maa_combat_enhanced"), TaskExecutionMode::Asynchronous);
        assert_eq!(get_task_execution_mode("maa_recruit_enhanced"), TaskExecutionMode::Asynchronous);
        
        // 默认
        assert_eq!(get_task_execution_mode("unknown_task"), TaskExecutionMode::Synchronous);
    }

    #[test]
    fn test_duration_estimation() {
        let combat_params = json!({"times": 5});
        assert!(estimate_task_duration("maa_combat_enhanced", &combat_params).contains("5次战斗"));
        
        let recruit_params = json!({"max_times": 3});
        assert!(estimate_task_duration("maa_recruit_enhanced", &recruit_params).contains("3次招募"));
    }
}