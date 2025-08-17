//! 作业匹配器的核心数据类型
//! 
//! 定义了作业匹配系统中使用的所有数据结构和枚举类型。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// 作业数据结构
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CopilotData {
    /// 作业ID
    pub id: String,
    /// 作业名称
    pub name: String,
    /// 关卡ID
    pub stage_id: String,
    /// 作业描述
    pub description: Option<String>,
    /// 干员配置
    pub operators: Vec<StageOperator>,
    /// 最低等级要求
    pub min_level: u32,
    /// 平均等级要求
    pub avg_level: f32,
    /// 精英化要求
    pub elite_requirements: HashMap<String, u32>,
    /// 技能等级要求
    pub skill_requirements: HashMap<String, u32>,
    /// 专精要求
    pub mastery_requirements: HashMap<String, u32>,
    /// 作业难度评分 (1-10)
    pub difficulty: u32,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 作业标签
    pub tags: Vec<String>,
    /// 是否推荐
    pub recommended: bool,
}

impl CopilotData {
    /// 创建新的作业数据
    pub fn new(
        id: String,
        name: String,
        stage_id: String,
        operators: Vec<StageOperator>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            stage_id,
            description: None,
            operators,
            min_level: 1,
            avg_level: 1.0,
            elite_requirements: HashMap::new(),
            skill_requirements: HashMap::new(),
            mastery_requirements: HashMap::new(),
            difficulty: 1,
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            recommended: false,
        }
    }

    /// 计算作业所需的总干员数量
    pub fn operator_count(&self) -> usize {
        self.operators.len()
    }

    /// 获取指定位置的干员
    pub fn get_operator_at_position(&self, position: u32) -> Option<&StageOperator> {
        self.operators.iter().find(|op| op.position == position)
    }

    /// 检查是否包含指定干员
    pub fn contains_operator(&self, operator_name: &str) -> bool {
        self.operators.iter().any(|op| op.name == operator_name)
    }
}

/// 干员需求结构
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OperatorRequirement {
    /// 干员名称
    pub name: String,
    /// 最低等级
    pub min_level: u32,
    /// 最低精英化等级
    pub min_elite: u32,
    /// 技能等级要求
    pub skill_level: u32,
    /// 专精要求 (技能序号, 专精等级)
    pub mastery: Option<(u32, u32)>,
    /// 是否必需（不可替换）
    pub required: bool,
    /// 替换优先级 (数值越低优先级越高)
    pub substitution_priority: u32,
}

impl OperatorRequirement {
    /// 创建新的干员需求
    pub fn new(name: String, min_level: u32) -> Self {
        Self {
            name,
            min_level,
            min_elite: 0,
            skill_level: 1,
            mastery: None,
            required: false,
            substitution_priority: 99,
        }
    }

    /// 设置为必需干员
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// 设置精英化要求
    pub fn with_elite(mut self, elite: u32) -> Self {
        self.min_elite = elite;
        self
    }

    /// 设置技能等级要求
    pub fn with_skill_level(mut self, level: u32) -> Self {
        self.skill_level = level;
        self
    }

    /// 设置专精要求
    pub fn with_mastery(mut self, skill: u32, level: u32) -> Self {
        self.mastery = Some((skill, level));
        self
    }

    /// 设置替换优先级
    pub fn with_substitution_priority(mut self, priority: u32) -> Self {
        self.substitution_priority = priority;
        self
    }
}

/// 舞台干员结构
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StageOperator {
    /// 干员名称
    pub name: String,
    /// 位置编号
    pub position: u32,
    /// 朝向 (上下左右)
    pub direction: String,
    /// 技能序号
    pub skill: u32,
    /// 等级要求
    pub level: u32,
    /// 精英化等级
    pub elite: u32,
    /// 潜能等级
    pub potential: u32,
    /// 专精等级
    pub mastery: u32,
    /// 是否为核心干员
    pub is_core: bool,
    /// 替换建议
    pub alternatives: Vec<String>,
}

impl StageOperator {
    /// 创建新的舞台干员
    pub fn new(name: String, position: u32) -> Self {
        Self {
            name,
            position,
            direction: "right".to_string(),
            skill: 1,
            level: 1,
            elite: 0,
            potential: 1,
            mastery: 0,
            is_core: false,
            alternatives: Vec::new(),
        }
    }

    /// 设置为核心干员
    pub fn core(mut self) -> Self {
        self.is_core = true;
        self
    }

    /// 设置技能
    pub fn with_skill(mut self, skill: u32) -> Self {
        self.skill = skill;
        self
    }

    /// 设置等级要求
    pub fn with_level(mut self, level: u32) -> Self {
        self.level = level;
        self
    }

    /// 设置精英化等级
    pub fn with_elite(mut self, elite: u32) -> Self {
        self.elite = elite;
        self
    }

    /// 设置专精等级
    pub fn with_mastery(mut self, mastery: u32) -> Self {
        self.mastery = mastery;
        self
    }

    /// 添加替换干员
    pub fn with_alternative(mut self, alternative: String) -> Self {
        self.alternatives.push(alternative);
        self
    }

    /// 批量添加替换干员
    pub fn with_alternatives(mut self, alternatives: Vec<String>) -> Self {
        self.alternatives.extend(alternatives);
        self
    }
}

/// 匹配阶段枚举
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum MatchStage {
    /// 简单匹配 - 基础配置匹配
    Simple,
    /// 等级匹配 - 干员等级和技能匹配
    Level,
    /// 智能匹配 - 智能替换匹配
    Smart,
}

impl std::fmt::Display for MatchStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatchStage::Simple => write!(f, "Simple"),
            MatchStage::Level => write!(f, "Level"),
            MatchStage::Smart => write!(f, "Smart"),
        }
    }
}

/// 匹配结果结构
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MatchResult {
    /// 作业数据
    pub copilot: CopilotData,
    /// 匹配得分
    pub score: MatchScore,
    /// 匹配阶段
    pub stage: MatchStage,
    /// 匹配详情
    pub details: String,
    /// 不匹配的干员列表
    pub missing_operators: Vec<String>,
    /// 需要替换的干员映射 (原干员 -> 替换干员)
    pub substitutions: HashMap<String, String>,
    /// 匹配时间
    pub matched_at: DateTime<Utc>,
}

impl MatchResult {
    /// 创建新的匹配结果
    pub fn new(
        copilot: CopilotData,
        score: MatchScore,
        stage: MatchStage,
    ) -> Self {
        Self {
            copilot,
            score,
            stage,
            details: String::new(),
            missing_operators: Vec::new(),
            substitutions: HashMap::new(),
            matched_at: Utc::now(),
        }
    }

    /// 设置匹配详情
    pub fn with_details(mut self, details: String) -> Self {
        self.details = details;
        self
    }

    /// 添加缺失干员
    pub fn with_missing_operator(mut self, operator: String) -> Self {
        self.missing_operators.push(operator);
        self
    }

    /// 添加干员替换
    pub fn with_substitution(mut self, original: String, substitute: String) -> Self {
        self.substitutions.insert(original, substitute);
        self
    }

    /// 检查是否为完美匹配
    pub fn is_perfect_match(&self) -> bool {
        self.missing_operators.is_empty() && self.substitutions.is_empty()
    }

    /// 检查是否可用（有缺失但可以替换）
    pub fn is_usable(&self) -> bool {
        self.score.total >= 0.6 // 总分大于等于60%认为可用
    }
}

/// 匹配得分结构
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MatchScore {
    /// 总分 (0.0 - 1.0)
    pub total: f32,
    /// 干员匹配得分
    pub operator_match: f32,
    /// 等级匹配得分
    pub level_match: f32,
    /// 技能匹配得分
    pub skill_match: f32,
    /// 精英化匹配得分
    pub elite_match: f32,
    /// 专精匹配得分
    pub mastery_match: f32,
    /// 配置匹配得分
    pub config_match: f32,
}

impl MatchScore {
    /// 创建新的匹配得分
    pub fn new() -> Self {
        Self {
            total: 0.0,
            operator_match: 0.0,
            level_match: 0.0,
            skill_match: 0.0,
            elite_match: 0.0,
            mastery_match: 0.0,
            config_match: 0.0,
        }
    }

    /// 计算总分
    pub fn calculate_total(&mut self) {
        // 加权计算总分
        self.total = (
            self.operator_match * 0.3 +     // 干员匹配 30%
            self.level_match * 0.2 +        // 等级匹配 20%
            self.skill_match * 0.15 +       // 技能匹配 15%
            self.elite_match * 0.15 +       // 精英化匹配 15%
            self.mastery_match * 0.1 +      // 专精匹配 10%
            self.config_match * 0.1         // 配置匹配 10%
        ).min(1.0).max(0.0);
    }

    /// 获取匹配等级
    pub fn get_grade(&self) -> &'static str {
        match self.total {
            x if x >= 0.9 => "S",
            x if x >= 0.8 => "A",
            x if x >= 0.7 => "B", 
            x if x >= 0.6 => "C",
            x if x >= 0.5 => "D",
            _ => "F",
        }
    }
}

impl Default for MatchScore {
    fn default() -> Self {
        Self::new()
    }
}

/// 作业匹配器错误类型
#[derive(Debug, thiserror::Error)]
pub enum CopilotError {
    #[error("Invalid operator: {0}")]
    InvalidOperator(String),

    #[error("Stage not found: {0}")]
    StageNotFound(String),

    #[error("Copilot not found: {0}")]
    CopilotNotFound(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Timeout error: operation timed out after {0}s")]
    TimeoutError(u64),

    #[error("Invalid data format: {0}")]
    InvalidDataFormat(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<reqwest::Error> for CopilotError {
    fn from(err: reqwest::Error) -> Self {
        CopilotError::NetworkError(err.to_string())
    }
}

impl From<serde_json::Error> for CopilotError {
    fn from(err: serde_json::Error) -> Self {
        CopilotError::SerializationError(err.to_string())
    }
}

impl From<sled::Error> for CopilotError {
    fn from(err: sled::Error) -> Self {
        CopilotError::CacheError(err.to_string())
    }
}

/// 作业匹配器结果类型
pub type CopilotResult<T> = Result<T, CopilotError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copilot_data_creation() {
        let operators = vec![
            StageOperator::new("夏".to_string(), 1)
                .core()
                .with_skill(3)
                .with_level(60)
                .with_elite(2)
                .with_mastery(3),
        ];

        let copilot = CopilotData::new(
            "test_001".to_string(),
            "测试作业".to_string(),
            "1-7".to_string(),
            operators,
        );

        assert_eq!(copilot.id, "test_001");
        assert_eq!(copilot.name, "测试作业");
        assert_eq!(copilot.stage_id, "1-7");
        assert_eq!(copilot.operator_count(), 1);
        assert!(copilot.contains_operator("夏"));
        assert!(!copilot.contains_operator("陈"));
    }

    #[test]
    fn test_operator_requirement_builder() {
        let req = OperatorRequirement::new("夏".to_string(), 50)
            .required()
            .with_elite(2)
            .with_skill_level(7)
            .with_mastery(3, 3)
            .with_substitution_priority(1);

        assert_eq!(req.name, "夏");
        assert_eq!(req.min_level, 50);
        assert_eq!(req.min_elite, 2);
        assert_eq!(req.skill_level, 7);
        assert_eq!(req.mastery, Some((3, 3)));
        assert!(req.required);
        assert_eq!(req.substitution_priority, 1);
    }

    #[test]
    fn test_stage_operator_builder() {
        let operator = StageOperator::new("陈".to_string(), 5)
            .core()
            .with_skill(2)
            .with_level(80)
            .with_elite(2)
            .with_mastery(2)
            .with_alternatives(vec!["山".to_string(), "煌".to_string()]);

        assert_eq!(operator.name, "陈");
        assert_eq!(operator.position, 5);
        assert!(operator.is_core);
        assert_eq!(operator.skill, 2);
        assert_eq!(operator.level, 80);
        assert_eq!(operator.elite, 2);
        assert_eq!(operator.mastery, 2);
        assert_eq!(operator.alternatives.len(), 2);
    }

    #[test]
    fn test_match_score_calculation() {
        let mut score = MatchScore::new();
        score.operator_match = 1.0;
        score.level_match = 0.8;
        score.skill_match = 0.9;
        score.elite_match = 0.7;
        score.mastery_match = 0.6;
        score.config_match = 0.8;

        score.calculate_total();

        // 验证总分计算正确
        let expected = 1.0 * 0.3 + 0.8 * 0.2 + 0.9 * 0.15 + 0.7 * 0.15 + 0.6 * 0.1 + 0.8 * 0.1;
        assert!((score.total - expected).abs() < f32::EPSILON);
        assert_eq!(score.get_grade(), "A");
    }

    #[test]
    fn test_match_result_creation() {
        let operators = vec![
            StageOperator::new("夏".to_string(), 1),
        ];

        let copilot = CopilotData::new(
            "test_001".to_string(),
            "测试作业".to_string(),
            "1-7".to_string(),
            operators,
        );

        let mut score = MatchScore::new();
        score.operator_match = 0.9;
        score.calculate_total();

        let result = MatchResult::new(copilot, score, MatchStage::Simple)
            .with_details("Perfect match".to_string())
            .with_missing_operator("陈".to_string())
            .with_substitution("山".to_string(), "煌".to_string());

        assert_eq!(result.stage, MatchStage::Simple);
        assert_eq!(result.details, "Perfect match");
        assert_eq!(result.missing_operators.len(), 1);
        assert_eq!(result.substitutions.len(), 1);
        assert!(!result.is_perfect_match());
    }

    #[test]
    fn test_match_stage_display() {
        assert_eq!(MatchStage::Simple.to_string(), "Simple");
        assert_eq!(MatchStage::Level.to_string(), "Level");
        assert_eq!(MatchStage::Smart.to_string(), "Smart");
    }

    #[test]
    fn test_error_conversions() {
        // Test serde_json error conversion
        let json_error = serde_json::from_str::<CopilotData>("invalid json").unwrap_err();
        let copilot_error: CopilotError = json_error.into();
        
        match copilot_error {
            CopilotError::SerializationError(_) => (),
            _ => panic!("Expected SerializationError"),
        }
    }
}