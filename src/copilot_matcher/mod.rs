//! 作业匹配专家Agent模块
//! 
//! 实现基于三阶段匹配算法的作业匹配系统：
//! 1. Simple Match - 基础配置匹配
//! 2. Level Match - 干员等级和技能匹配
//! 3. Smart Match - 智能替换匹配
//! 
//! 支持缓存和TTL机制，提供高效的作业匹配服务。

pub mod types;
pub mod api_client;
pub mod cache;
pub mod matcher;

// 重新导出核心类型和特征
pub use types::{
    CopilotData,
    OperatorRequirement,
    StageOperator,
    MatchStage,
    MatchResult,
    MatchScore,
    CopilotError,
    CopilotResult,
};

pub use api_client::{
    ApiClient,
    ApiConfig,
    ApiClientTrait,
};

pub use cache::{
    CacheManager,
    CacheEntry,
    CacheConfig,
    CacheManagerTrait,
};

pub use matcher::{
    CopilotMatcher,
    CopilotMatcherTrait,
    MatcherConfig,
};

/// 作业匹配器模块的便捷重导出
pub mod prelude {
    pub use super::{
        CopilotMatcher,
        CopilotMatcherTrait,
        CopilotData,
        OperatorRequirement,
        StageOperator,
        MatchStage,
        MatchResult,
        MatchScore,
        CopilotError,
        CopilotResult,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_module_imports() {
        // 验证所有模块都能正确导入
        let _ = CopilotError::InvalidOperator("test".to_string());
        let _ = MatchStage::Simple;
    }
}