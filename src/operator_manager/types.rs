//! Operator Manager Types
//!
//! This module defines the core data structures for operator management,
//! including operator information, filter criteria, and related types.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Operator information
/// 
/// Core data structure representing an operator with all their progression data.
/// This structure is designed to be compatible with MAA's operator recognition
/// and provides comprehensive information about operator development status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Operator {
    /// Operator name (unique identifier)
    pub name: String,
    
    /// Operator profession (Guard, Sniper, Caster, etc.)
    pub profession: String,
    
    /// Operator rarity (1-6 stars)
    pub rarity: u8,
    
    /// Elite level (0, 1, 2)
    pub elite: u8,
    
    /// Current level (1-90, depends on elite level)
    pub level: u8,
    
    /// Skill levels [skill1, skill2, skill3] (1-7, where 4-7 are mastery levels)
    pub skill_levels: Vec<u8>,
    
    /// Module information
    pub modules: Vec<ModuleInfo>,
    
    /// Potential level (1-6)
    pub potential: u8,
    
    /// Trust level (0-200)
    pub trust: u16,
    
    /// Last time this operator data was updated
    pub last_updated: DateTime<Utc>,
    
    /// Additional metadata (flexible storage for extra data)
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Operator {
    /// Create a new operator with default values
    pub fn new(name: String, profession: String, rarity: u8) -> Self {
        Self {
            name,
            profession,
            rarity,
            elite: 0,
            level: 1,
            skill_levels: vec![1, 1, 1],
            modules: Vec::new(),
            potential: 1,
            trust: 0,
            last_updated: Utc::now(),
            metadata: HashMap::new(),
        }
    }
    
    /// Check if operator is at maximum level for their elite
    pub fn is_max_level(&self) -> bool {
        match self.elite {
            0 => self.level >= 50,
            1 => self.level >= 80,
            2 => self.level >= 90,
            _ => false,
        }
    }
    
    /// Check if operator has any mastery skills (skill level > 7)
    pub fn has_mastery(&self) -> bool {
        self.skill_levels.iter().any(|&level| level > 7)
    }
    
    /// Check if operator has any modules
    pub fn has_modules(&self) -> bool {
        !self.modules.is_empty()
    }
    
    /// Get the highest skill level
    pub fn max_skill_level(&self) -> u8 {
        self.skill_levels.iter().max().copied().unwrap_or(1)
    }
    
    /// Calculate a development score based on progression
    pub fn development_score(&self) -> f32 {
        let elite_score = match self.elite {
            0 => 0.0,
            1 => 0.3,
            2 => 1.0,
            _ => 0.0,
        };
        
        let level_ratio = match self.elite {
            0 => self.level as f32 / 50.0,
            1 => self.level as f32 / 80.0,
            2 => self.level as f32 / 90.0,
            _ => 0.0,
        };
        
        let skill_score = self.skill_levels.iter()
            .map(|&level| level as f32 / 7.0)
            .sum::<f32>() / self.skill_levels.len() as f32;
        
        let trust_score = self.trust as f32 / 200.0;
        let module_score = if self.has_modules() { 0.2 } else { 0.0 };
        
        // Weighted average
        (elite_score * 0.3 + level_ratio * 0.2 + skill_score * 0.3 + trust_score * 0.1 + module_score) / 1.1
    }
}

/// Module information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleInfo {
    /// Module type/name (e.g., "CAM-X", "GUA-Y")
    pub module_type: String,
    
    /// Module level (1-3)
    pub level: u8,
    
    /// Whether the module is unlocked
    pub unlocked: bool,
    
    /// Module stage (X, Y, Delta, etc.)
    #[serde(default)]
    pub stage: Option<String>,
}

impl ModuleInfo {
    /// Create a new module info
    pub fn new(module_type: String, level: u8, unlocked: bool) -> Self {
        Self {
            module_type,
            level,
            unlocked,
            stage: None,
        }
    }
}

/// Operator filter criteria
/// 
/// Used to filter operators based on various criteria.
/// All filter fields are optional - only specified criteria are applied.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OperatorFilter {
    /// Filter by name (partial match, case-insensitive)
    pub name: Option<String>,
    
    /// Filter by profession(s)
    pub professions: Option<Vec<String>>,
    
    /// Minimum rarity (stars)
    pub min_rarity: Option<u8>,
    
    /// Maximum rarity (stars)
    pub max_rarity: Option<u8>,
    
    /// Minimum elite level
    pub min_elite: Option<u8>,
    
    /// Maximum elite level
    pub max_elite: Option<u8>,
    
    /// Minimum level
    pub min_level: Option<u8>,
    
    /// Maximum level
    pub max_level: Option<u8>,
    
    /// Minimum potential
    pub min_potential: Option<u8>,
    
    /// Maximum potential
    pub max_potential: Option<u8>,
    
    /// Minimum trust level
    pub min_trust: Option<u16>,
    
    /// Maximum trust level
    pub max_trust: Option<u16>,
    
    /// Must have at least this skill level (skill_index, min_level)
    pub skill_requirement: Option<(u8, u8)>,
    
    /// Must have modules
    pub has_modules: Option<bool>,
    
    /// Must be at max level for their elite
    pub max_level_only: Option<bool>,
    
    /// Must have mastery skills
    pub has_mastery: Option<bool>,
    
    /// Minimum development score
    pub min_development_score: Option<f32>,
}

impl OperatorFilter {
    /// Create an empty filter (matches all operators)
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Check if an operator matches this filter
    pub fn matches(&self, operator: &Operator) -> bool {
        // Name filter
        if let Some(ref filter_name) = self.name {
            if !operator.name.to_lowercase().contains(&filter_name.to_lowercase()) {
                return false;
            }
        }
        
        // Profession filter
        if let Some(ref professions) = self.professions {
            if !professions.iter().any(|p| p.to_lowercase() == operator.profession.to_lowercase()) {
                return false;
            }
        }
        
        // Rarity filters
        if let Some(min_rarity) = self.min_rarity {
            if operator.rarity < min_rarity {
                return false;
            }
        }
        
        if let Some(max_rarity) = self.max_rarity {
            if operator.rarity > max_rarity {
                return false;
            }
        }
        
        // Elite filters
        if let Some(min_elite) = self.min_elite {
            if operator.elite < min_elite {
                return false;
            }
        }
        
        if let Some(max_elite) = self.max_elite {
            if operator.elite > max_elite {
                return false;
            }
        }
        
        // Level filters
        if let Some(min_level) = self.min_level {
            if operator.level < min_level {
                return false;
            }
        }
        
        if let Some(max_level) = self.max_level {
            if operator.level > max_level {
                return false;
            }
        }
        
        // Potential filters
        if let Some(min_potential) = self.min_potential {
            if operator.potential < min_potential {
                return false;
            }
        }
        
        if let Some(max_potential) = self.max_potential {
            if operator.potential > max_potential {
                return false;
            }
        }
        
        // Trust filters
        if let Some(min_trust) = self.min_trust {
            if operator.trust < min_trust {
                return false;
            }
        }
        
        if let Some(max_trust) = self.max_trust {
            if operator.trust > max_trust {
                return false;
            }
        }
        
        // Skill requirement
        if let Some((skill_idx, min_skill_level)) = self.skill_requirement {
            match operator.skill_levels.get(skill_idx as usize) {
                Some(&level) if level >= min_skill_level => {},
                _ => return false,
            }
        }
        
        // Module requirement
        if let Some(has_modules) = self.has_modules {
            if has_modules != operator.has_modules() {
                return false;
            }
        }
        
        // Max level requirement
        if let Some(max_level_only) = self.max_level_only {
            if max_level_only && !operator.is_max_level() {
                return false;
            }
        }
        
        // Mastery requirement
        if let Some(has_mastery) = self.has_mastery {
            if has_mastery != operator.has_mastery() {
                return false;
            }
        }
        
        // Development score requirement
        if let Some(min_score) = self.min_development_score {
            if operator.development_score() < min_score {
                return false;
            }
        }
        
        true
    }
    
    /// Builder pattern methods for convenient filter construction
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
    
    pub fn with_profession(mut self, profession: String) -> Self {
        self.professions = Some(vec![profession]);
        self
    }
    
    pub fn with_professions(mut self, professions: Vec<String>) -> Self {
        self.professions = Some(professions);
        self
    }
    
    pub fn with_min_rarity(mut self, rarity: u8) -> Self {
        self.min_rarity = Some(rarity);
        self
    }
    
    pub fn with_min_elite(mut self, elite: u8) -> Self {
        self.min_elite = Some(elite);
        self
    }
    
    pub fn with_has_modules(mut self, has_modules: bool) -> Self {
        self.has_modules = Some(has_modules);
        self
    }
    
    pub fn with_has_mastery(mut self, has_mastery: bool) -> Self {
        self.has_mastery = Some(has_mastery);
        self
    }
}

/// Operator scan result
/// 
/// Contains the results of an operator scanning operation,
/// including newly found operators and update statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// All operators found during the scan
    pub operators: Vec<Operator>,
    
    /// Number of new operators discovered
    pub new_count: u32,
    
    /// Number of existing operators updated
    pub updated_count: u32,
    
    /// Number of operators that couldn't be processed
    pub failed_count: u32,
    
    /// Scan duration in milliseconds
    pub scan_duration_ms: u64,
    
    /// Timestamp when the scan completed
    pub completed_at: DateTime<Utc>,
    
    /// Any warnings or issues encountered during scanning
    pub warnings: Vec<String>,
}

impl ScanResult {
    /// Create a new scan result
    pub fn new(operators: Vec<Operator>, scan_duration_ms: u64) -> Self {
        Self {
            operators,
            new_count: 0,
            updated_count: 0,
            failed_count: 0,
            scan_duration_ms,
            completed_at: Utc::now(),
            warnings: Vec::new(),
        }
    }
    
    /// Total operators processed (successful + failed)
    pub fn total_processed(&self) -> u32 {
        self.new_count + self.updated_count + self.failed_count
    }
}

/// Operator summary statistics
/// 
/// Provides statistical overview of an operator collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorSummary {
    /// Total number of operators
    pub total_count: u32,
    
    /// Breakdown by rarity
    pub by_rarity: HashMap<u8, u32>,
    
    /// Breakdown by profession
    pub by_profession: HashMap<String, u32>,
    
    /// Breakdown by elite level
    pub by_elite: HashMap<u8, u32>,
    
    /// Number of operators at max level
    pub max_level_count: u32,
    
    /// Number of operators with modules
    pub module_count: u32,
    
    /// Number of operators with mastery skills
    pub mastery_count: u32,
    
    /// Average trust level
    pub avg_trust: f32,
    
    /// Average development score
    pub avg_development_score: f32,
    
    /// Highest rarity operators
    pub highest_rarity_ops: Vec<String>,
}

impl OperatorSummary {
    /// Generate summary from a collection of operators
    pub fn from_operators(operators: &[Operator]) -> Self {
        let total_count = operators.len() as u32;
        
        let mut by_rarity = HashMap::new();
        let mut by_profession = HashMap::new();
        let mut by_elite = HashMap::new();
        let mut max_level_count = 0;
        let mut module_count = 0;
        let mut mastery_count = 0;
        let mut total_trust = 0u64;
        let mut total_dev_score = 0f32;
        let mut max_rarity = 0u8;
        
        for op in operators {
            *by_rarity.entry(op.rarity).or_insert(0) += 1;
            *by_profession.entry(op.profession.clone()).or_insert(0) += 1;
            *by_elite.entry(op.elite).or_insert(0) += 1;
            
            if op.is_max_level() {
                max_level_count += 1;
            }
            
            if op.has_modules() {
                module_count += 1;
            }
            
            if op.has_mastery() {
                mastery_count += 1;
            }
            
            total_trust += op.trust as u64;
            total_dev_score += op.development_score();
            max_rarity = max_rarity.max(op.rarity);
        }
        
        let avg_trust = if total_count > 0 {
            total_trust as f32 / total_count as f32
        } else {
            0.0
        };
        
        let avg_development_score = if total_count > 0 {
            total_dev_score / total_count as f32
        } else {
            0.0
        };
        
        // Find highest rarity operators
        let highest_rarity_ops: Vec<String> = operators
            .iter()
            .filter(|op| op.rarity == max_rarity)
            .map(|op| op.name.clone())
            .collect();
        
        Self {
            total_count,
            by_rarity,
            by_profession,
            by_elite,
            max_level_count,
            module_count,
            mastery_count,
            avg_trust,
            avg_development_score,
            highest_rarity_ops,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_operator_creation() {
        let op = Operator::new("Amiya".to_string(), "Caster".to_string(), 5);
        assert_eq!(op.name, "Amiya");
        assert_eq!(op.profession, "Caster");
        assert_eq!(op.rarity, 5);
        assert_eq!(op.elite, 0);
        assert_eq!(op.level, 1);
        assert_eq!(op.skill_levels, vec![1, 1, 1]);
        assert_eq!(op.potential, 1);
        assert_eq!(op.trust, 0);
    }
    
    #[test]
    fn test_operator_max_level() {
        let mut op = Operator::new("Test".to_string(), "Guard".to_string(), 6);
        
        // Elite 0
        op.elite = 0;
        op.level = 50;
        assert!(op.is_max_level());
        
        op.level = 49;
        assert!(!op.is_max_level());
        
        // Elite 2
        op.elite = 2;
        op.level = 90;
        assert!(op.is_max_level());
        
        op.level = 89;
        assert!(!op.is_max_level());
    }
    
    #[test]
    fn test_operator_mastery() {
        let mut op = Operator::new("Test".to_string(), "Guard".to_string(), 6);
        
        // No mastery
        op.skill_levels = vec![7, 6, 5];
        assert!(!op.has_mastery());
        
        // Has mastery
        op.skill_levels = vec![10, 6, 5];
        assert!(op.has_mastery());
    }
    
    #[test]
    fn test_operator_modules() {
        let mut op = Operator::new("Test".to_string(), "Guard".to_string(), 6);
        
        // No modules
        assert!(!op.has_modules());
        
        // Has modules
        op.modules.push(ModuleInfo::new("GUA-X".to_string(), 3, true));
        assert!(op.has_modules());
    }
    
    #[test]
    fn test_operator_filter_name() {
        let op = Operator::new("Amiya".to_string(), "Caster".to_string(), 5);
        
        let filter = OperatorFilter::new().with_name("amiya".to_string());
        assert!(filter.matches(&op));
        
        let filter = OperatorFilter::new().with_name("ami".to_string());
        assert!(filter.matches(&op));
        
        let filter = OperatorFilter::new().with_name("silver".to_string());
        assert!(!filter.matches(&op));
    }
    
    #[test]
    fn test_operator_filter_profession() {
        let op = Operator::new("Amiya".to_string(), "Caster".to_string(), 5);
        
        let filter = OperatorFilter::new().with_profession("Caster".to_string());
        assert!(filter.matches(&op));
        
        let filter = OperatorFilter::new().with_profession("caster".to_string());
        assert!(filter.matches(&op));
        
        let filter = OperatorFilter::new().with_profession("Guard".to_string());
        assert!(!filter.matches(&op));
    }
    
    #[test]
    fn test_operator_filter_rarity() {
        let op = Operator::new("Amiya".to_string(), "Caster".to_string(), 5);
        
        let filter = OperatorFilter::new().with_min_rarity(5);
        assert!(filter.matches(&op));
        
        let filter = OperatorFilter::new().with_min_rarity(6);
        assert!(!filter.matches(&op));
        
        let mut filter = OperatorFilter::new();
        filter.max_rarity = Some(5);
        assert!(filter.matches(&op));
        
        filter.max_rarity = Some(4);
        assert!(!filter.matches(&op));
    }
    
    #[test]
    fn test_operator_filter_complex() {
        let mut op = Operator::new("SilverAsh".to_string(), "Guard".to_string(), 6);
        op.elite = 2;
        op.level = 90;
        op.skill_levels = vec![7, 7, 10];
        op.modules.push(ModuleInfo::new("GUA-Y".to_string(), 3, true));
        
        let filter = OperatorFilter::new()
            .with_min_rarity(6)
            .with_min_elite(2)
            .with_has_modules(true)
            .with_has_mastery(true);
        
        assert!(filter.matches(&op));
        
        // Remove mastery
        op.skill_levels = vec![7, 7, 7];
        assert!(!filter.matches(&op));
    }
    
    #[test]
    fn test_operator_development_score() {
        let mut op = Operator::new("Test".to_string(), "Guard".to_string(), 6);
        
        // Base operator should have low score
        let base_score = op.development_score();
        assert!(base_score < 0.5);
        
        // Fully developed operator should have high score
        op.elite = 2;
        op.level = 90;
        op.skill_levels = vec![7, 7, 7];
        op.trust = 200;
        op.modules.push(ModuleInfo::new("GUA-X".to_string(), 3, true));
        
        let max_score = op.development_score();
        assert!(max_score > 0.9);
        assert!(max_score > base_score);
    }
    
    #[test]
    fn test_operator_summary() {
        let operators = vec![
            {
                let mut op = Operator::new("Amiya".to_string(), "Caster".to_string(), 5);
                op.elite = 2;
                op.level = 80;
                op.trust = 200;
                op
            },
            {
                let mut op = Operator::new("SilverAsh".to_string(), "Guard".to_string(), 6);
                op.elite = 2;
                op.level = 90;
                op.skill_levels = vec![7, 7, 10];
                op.trust = 180;
                op.modules.push(ModuleInfo::new("GUA-Y".to_string(), 3, true));
                op
            },
            {
                let mut op = Operator::new("Melantha".to_string(), "Guard".to_string(), 3);
                op.elite = 1;
                op.level = 55;
                op.trust = 100;
                op
            },
        ];
        
        let summary = OperatorSummary::from_operators(&operators);
        
        assert_eq!(summary.total_count, 3);
        assert_eq!(summary.by_rarity[&5], 1);
        assert_eq!(summary.by_rarity[&6], 1);
        assert_eq!(summary.by_rarity[&3], 1);
        assert_eq!(summary.by_profession["Caster"], 1);
        assert_eq!(summary.by_profession["Guard"], 2);
        assert_eq!(summary.max_level_count, 1); // Only SilverAsh at max level
        assert_eq!(summary.module_count, 1); // Only SilverAsh has modules
        assert_eq!(summary.mastery_count, 1); // Only SilverAsh has mastery
        assert!(summary.avg_trust > 0.0);
        assert!(summary.avg_development_score > 0.0);
        assert_eq!(summary.highest_rarity_ops, vec!["SilverAsh"]);
    }
    
    #[test]
    fn test_scan_result() {
        let operators = vec![
            Operator::new("Amiya".to_string(), "Caster".to_string(), 5),
            Operator::new("SilverAsh".to_string(), "Guard".to_string(), 6),
        ];
        
        let mut result = ScanResult::new(operators, 1500);
        result.new_count = 2;
        result.updated_count = 0;
        result.failed_count = 0;
        
        assert_eq!(result.total_processed(), 2);
        assert_eq!(result.scan_duration_ms, 1500);
        assert_eq!(result.operators.len(), 2);
    }
}