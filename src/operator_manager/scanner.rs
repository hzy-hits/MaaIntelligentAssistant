//! Operator Scanner
//!
//! This module provides operator scanning functionality using MAA's operator recognition capabilities.
//! It handles the scanning process, result parsing, and integration with the cache system.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;
use tracing::{debug, info, warn};

use crate::maa_adapter::{MaaAdapterTrait, MaaTaskType, TaskParams};
use crate::operator_manager::{
    types::{Operator, ModuleInfo, ScanResult},
    errors::{OperatorError, OperatorResult},
    cache::OperatorCache,
};

/// Scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerConfig {
    /// Maximum time to wait for scan completion (in seconds)
    pub scan_timeout_seconds: u64,
    
    /// Whether to perform a full scan or incremental scan
    pub full_scan: bool,
    
    /// Whether to include operator details (skills, modules, etc.)
    pub include_details: bool,
    
    /// Whether to cache scan results
    pub cache_results: bool,
    
    /// Retry attempts for failed operations
    pub max_retries: u32,
    
    /// Delay between retry attempts (in milliseconds)
    pub retry_delay_ms: u64,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            scan_timeout_seconds: 60,
            full_scan: true,
            include_details: true,
            cache_results: true,
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }
}

/// Raw operator data from MAA
/// 
/// This represents the raw data structure returned by MAA's operator scanning.
/// It needs to be parsed and converted to our internal Operator structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawOperatorData {
    /// Operator name
    name: String,
    
    /// Operator class/profession
    #[serde(rename = "class")]
    profession: Option<String>,
    
    /// Rarity (stars)
    rarity: Option<u8>,
    
    /// Elite level
    elite: Option<u8>,
    
    /// Current level
    level: Option<u8>,
    
    /// Skill information
    skills: Option<Vec<RawSkillData>>,
    
    /// Module information
    modules: Option<Vec<RawModuleData>>,
    
    /// Potential level
    potential: Option<u8>,
    
    /// Trust/Intimacy level
    trust: Option<u16>,
    
    /// Additional raw data from MAA
    #[serde(flatten)]
    raw_data: HashMap<String, serde_json::Value>,
}

/// Raw skill data from MAA
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawSkillData {
    /// Skill index (0, 1, 2)
    index: u8,
    
    /// Skill level (1-7, where 4-7 are mastery levels)
    level: u8,
    
    /// Skill name
    name: Option<String>,
}

/// Raw module data from MAA
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawModuleData {
    /// Module type/name
    name: String,
    
    /// Module level
    level: u8,
    
    /// Whether the module is unlocked
    unlocked: bool,
    
    /// Module stage
    stage: Option<String>,
}

/// Scan progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    /// Current stage of scanning
    pub stage: String,
    
    /// Progress percentage (0.0 - 1.0)
    pub progress: f32,
    
    /// Number of operators processed so far
    pub operators_processed: u32,
    
    /// Estimated time remaining (in seconds)
    pub eta_seconds: Option<u64>,
    
    /// Current operator being processed
    pub current_operator: Option<String>,
}

/// Operator scanner
pub struct OperatorScanner {
    /// MAA adapter for scanning operations
    maa_adapter: Arc<dyn MaaAdapterTrait + Send + Sync>,
    
    /// Cache for storing results
    cache: Arc<OperatorCache>,
    
    /// Scanner configuration
    config: ScannerConfig,
}

impl OperatorScanner {
    /// Create a new operator scanner
    pub fn new(
        maa_adapter: Arc<dyn MaaAdapterTrait + Send + Sync>,
        cache: Arc<OperatorCache>,
        config: ScannerConfig,
    ) -> Self {
        Self {
            maa_adapter,
            cache,
            config,
        }
    }
    
    /// Scan all operators
    pub async fn scan_operators(&self) -> OperatorResult<ScanResult> {
        let start_time = Instant::now();
        info!("Starting operator scan (full: {})", self.config.full_scan);
        
        // Create scan task parameters
        let scan_params = self.create_scan_params()?;
        
        // Execute the scan with timeout
        let raw_results = match timeout(
            Duration::from_secs(self.config.scan_timeout_seconds),
            self.execute_scan(scan_params)
        ).await {
            Ok(result) => result?,
            Err(_) => {
                return Err(OperatorError::timeout(
                    "operator_scan",
                    self.config.scan_timeout_seconds * 1000
                ));
            }
        };
        
        // Parse and validate results
        let operators = self.parse_scan_results(raw_results).await?;
        
        // Create scan result
        let scan_duration_ms = start_time.elapsed().as_millis() as u64;
        let mut scan_result = ScanResult::new(operators.clone(), scan_duration_ms);
        
        // Compare with cache to determine new/updated counts
        self.update_scan_statistics(&mut scan_result).await?;
        
        // Cache the results if enabled
        if self.config.cache_results {
            self.cache_scan_results(&scan_result).await?;
        }
        
        info!(
            "Operator scan completed: {} total, {} new, {} updated, duration: {}ms",
            scan_result.operators.len(),
            scan_result.new_count,
            scan_result.updated_count,
            scan_result.scan_duration_ms
        );
        
        Ok(scan_result)
    }
    
    /// Scan a specific operator by name
    pub async fn scan_operator(&self, name: &str) -> OperatorResult<Option<Operator>> {
        info!("Scanning specific operator: {}", name);
        
        // Create targeted scan parameters
        let scan_params = self.create_targeted_scan_params(name)?;
        
        // Execute the scan
        let raw_results = self.execute_scan(scan_params).await?;
        
        // Parse results and find the target operator
        let operators = self.parse_scan_results(raw_results).await?;
        
        let target_operator = operators.into_iter()
            .find(|op| op.name.to_lowercase() == name.to_lowercase());
        
        if let Some(ref operator) = target_operator {
            // Cache the result
            if self.config.cache_results {
                self.cache.store_operator(operator.clone()).await?;
            }
            
            info!("Successfully scanned operator: {}", name);
        } else {
            warn!("Operator not found in scan results: {}", name);
        }
        
        Ok(target_operator)
    }
    
    /// Check if scanning is supported and MAA is ready
    pub async fn check_scan_availability(&self) -> OperatorResult<bool> {
        debug!("Checking scan availability");
        
        // Check MAA adapter status
        let status = self.maa_adapter.get_status().await
            .map_err(|e| OperatorError::maa_operation("status_check", e.to_string()))?;
        
        // Check if MAA is connected and ready
        match status {
            crate::maa_adapter::MaaStatus::Connected => {
                debug!("MAA is connected and ready for scanning");
                Ok(true)
            },
            crate::maa_adapter::MaaStatus::Idle => {
                debug!("MAA is idle but ready for scanning");
                Ok(true)
            },
            crate::maa_adapter::MaaStatus::Connecting => {
                debug!("MAA is connecting, not ready yet");
                Ok(false)
            },
            other => {
                warn!("MAA is not ready for scanning: {:?}", other);
                Ok(false)
            }
        }
    }
    
    /// Create scan task parameters
    fn create_scan_params(&self) -> OperatorResult<TaskParams> {
        let mut params = HashMap::new();
        
        // Basic scan parameters
        params.insert("type".to_string(), serde_json::Value::String("operator_scan".to_string()));
        params.insert("full_scan".to_string(), serde_json::Value::Bool(self.config.full_scan));
        params.insert("include_details".to_string(), serde_json::Value::Bool(self.config.include_details));
        
        // Convert to TaskParams
        let raw_params = serde_json::to_string(&params)
            .map_err(|e| OperatorError::serialization("task_params", e.to_string()))?;
        
        let task_params = TaskParams {
            raw: raw_params,
            parsed: params,
            settings: HashMap::new(),
        };
        
        Ok(task_params)
    }
    
    /// Create targeted scan parameters for a specific operator
    fn create_targeted_scan_params(&self, operator_name: &str) -> OperatorResult<TaskParams> {
        let mut params = HashMap::new();
        
        params.insert("type".to_string(), serde_json::Value::String("operator_scan".to_string()));
        params.insert("target_operator".to_string(), serde_json::Value::String(operator_name.to_string()));
        params.insert("include_details".to_string(), serde_json::Value::Bool(self.config.include_details));
        
        let raw_params = serde_json::to_string(&params)
            .map_err(|e| OperatorError::serialization("task_params", e.to_string()))?;
        
        let task_params = TaskParams {
            raw: raw_params,
            parsed: params,
            settings: HashMap::new(),
        };
        
        Ok(task_params)
    }
    
    /// Execute the scan operation with retry logic
    async fn execute_scan(&self, params: TaskParams) -> OperatorResult<Vec<RawOperatorData>> {
        let mut last_error = None;
        
        for attempt in 1..=self.config.max_retries {
            debug!("Scan attempt {} of {}", attempt, self.config.max_retries);
            
            match self.try_scan(&params).await {
                Ok(results) => return Ok(results),
                Err(e) => {
                    warn!("Scan attempt {} failed: {}", attempt, e);
                    last_error = Some(e);
                    
                    if attempt < self.config.max_retries {
                        tokio::time::sleep(Duration::from_millis(self.config.retry_delay_ms)).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| {
            OperatorError::scan_failed("All scan attempts failed without specific error")
        }))
    }
    
    /// Single scan attempt
    async fn try_scan(&self, params: &TaskParams) -> OperatorResult<Vec<RawOperatorData>> {
        // Create and start the scan task
        let task_type = MaaTaskType::Custom {
            task_name: "operator_scan".to_string(),
            task_params: params.raw.clone(),
        };
        
        let task_id = self.maa_adapter.create_task(task_type, params.clone()).await
            .map_err(|e| OperatorError::maa_operation("create_task", e.to_string()))?;
        
        debug!("Created scan task: {}", task_id);
        
        // Start the task
        self.maa_adapter.start_task(task_id).await
            .map_err(|e| OperatorError::maa_operation("start_task", e.to_string()))?;
        
        // Wait for completion and get results
        self.wait_for_task_completion(task_id).await
    }
    
    /// Wait for task completion and extract results
    async fn wait_for_task_completion(&self, task_id: i32) -> OperatorResult<Vec<RawOperatorData>> {
        let start_time = Instant::now();
        let timeout_duration = Duration::from_secs(self.config.scan_timeout_seconds);
        
        loop {
            // Check for timeout
            if start_time.elapsed() > timeout_duration {
                return Err(OperatorError::timeout("task_completion", self.config.scan_timeout_seconds * 1000));
            }
            
            // Get task status
            let status = self.maa_adapter.get_status().await
                .map_err(|e| OperatorError::maa_operation("get_status", e.to_string()))?;
            
            match status {
                crate::maa_adapter::MaaStatus::Running { task_id: running_id, .. } if running_id == task_id => {
                    debug!("Task {} still running", task_id);
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    continue;
                },
                crate::maa_adapter::MaaStatus::Completed { task_id: completed_id, result, .. } if completed_id == task_id => {
                    debug!("Task {} completed", task_id);
                    return self.parse_task_result(&result);
                },
                crate::maa_adapter::MaaStatus::Failed { task_id: failed_id, error, .. } if failed_id == task_id => {
                    return Err(OperatorError::maa_operation("task_execution", error));
                },
                _ => {
                    debug!("Task status: {:?}", status);
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }
        }
    }
    
    /// Parse task result into raw operator data
    fn parse_task_result(&self, result: &str) -> OperatorResult<Vec<RawOperatorData>> {
        // Parse the JSON result from MAA
        let json_result: serde_json::Value = serde_json::from_str(result)
            .map_err(|e| OperatorError::serialization("parse_result", e.to_string()))?;
        
        // Extract operator data array
        let operators_json = json_result.get("operators")
            .ok_or_else(|| OperatorError::invalid_data("operators", "Missing operators field in scan result"))?;
        
        let raw_operators: Vec<RawOperatorData> = serde_json::from_value(operators_json.clone())
            .map_err(|e| OperatorError::serialization("parse_operators", e.to_string()))?;
        
        debug!("Parsed {} operators from scan result", raw_operators.len());
        Ok(raw_operators)
    }
    
    /// Parse raw scan results into validated Operator structures
    async fn parse_scan_results(&self, raw_data: Vec<RawOperatorData>) -> OperatorResult<Vec<Operator>> {
        let mut operators = Vec::new();
        let mut failed_count = 0u32;
        
        for raw_operator in raw_data {
            match self.parse_single_operator(raw_operator).await {
                Ok(operator) => operators.push(operator),
                Err(e) => {
                    warn!("Failed to parse operator: {}", e);
                    failed_count += 1;
                }
            }
        }
        
        if failed_count > 0 {
            warn!("Failed to parse {} operators", failed_count);
        }
        
        Ok(operators)
    }
    
    /// Parse a single raw operator into a validated Operator
    async fn parse_single_operator(&self, raw: RawOperatorData) -> OperatorResult<Operator> {
        // Validate required fields
        if raw.name.trim().is_empty() {
            return Err(OperatorError::invalid_data("name", "Operator name cannot be empty"));
        }
        
        // Parse skills
        let mut skill_levels = vec![1, 1, 1]; // Default skill levels
        if let Some(skills) = raw.skills {
            for skill in skills {
                if skill.index < 3 {
                    skill_levels[skill.index as usize] = skill.level;
                }
            }
        }
        
        // Parse modules
        let mut modules = Vec::new();
        if let Some(raw_modules) = raw.modules {
            for raw_module in raw_modules {
                modules.push(ModuleInfo {
                    module_type: raw_module.name,
                    level: raw_module.level,
                    unlocked: raw_module.unlocked,
                    stage: raw_module.stage,
                });
            }
        }
        
        // Create metadata from raw data
        let mut metadata = HashMap::new();
        for (key, value) in raw.raw_data {
            metadata.insert(key, value);
        }
        
        // Create the operator
        let operator = Operator {
            name: raw.name,
            profession: raw.profession.unwrap_or_else(|| "Unknown".to_string()),
            rarity: raw.rarity.unwrap_or(1),
            elite: raw.elite.unwrap_or(0),
            level: raw.level.unwrap_or(1),
            skill_levels,
            modules,
            potential: raw.potential.unwrap_or(1),
            trust: raw.trust.unwrap_or(0),
            last_updated: Utc::now(),
            metadata,
        };
        
        // Validate the operator
        self.validate_operator(&operator)?;
        
        Ok(operator)
    }
    
    /// Validate an operator's data
    fn validate_operator(&self, operator: &Operator) -> OperatorResult<()> {
        // Validate rarity
        if operator.rarity < 1 || operator.rarity > 6 {
            return Err(OperatorError::validation_with_format(
                "rarity",
                format!("Invalid rarity: {}", operator.rarity),
                "1-6"
            ));
        }
        
        // Validate elite level
        if operator.elite > 2 {
            return Err(OperatorError::validation_with_format(
                "elite",
                format!("Invalid elite level: {}", operator.elite),
                "0-2"
            ));
        }
        
        // Validate level based on elite
        let max_level = match operator.elite {
            0 => 50,
            1 => 80,
            2 => 90,
            _ => 90,
        };
        
        if operator.level > max_level {
            return Err(OperatorError::validation_with_format(
                "level",
                format!("Invalid level {} for elite {}", operator.level, operator.elite),
                format!("1-{}", max_level)
            ));
        }
        
        // Validate skill levels
        for (i, &skill_level) in operator.skill_levels.iter().enumerate() {
            if skill_level < 1 || skill_level > 7 {
                return Err(OperatorError::validation_with_format(
                    "skill_levels",
                    format!("Invalid skill level {} for skill {}", skill_level, i + 1),
                    "1-7"
                ));
            }
        }
        
        // Validate potential
        if operator.potential < 1 || operator.potential > 6 {
            return Err(OperatorError::validation_with_format(
                "potential",
                format!("Invalid potential: {}", operator.potential),
                "1-6"
            ));
        }
        
        // Validate trust
        if operator.trust > 200 {
            return Err(OperatorError::validation_with_format(
                "trust",
                format!("Invalid trust: {}", operator.trust),
                "0-200"
            ));
        }
        
        Ok(())
    }
    
    /// Update scan statistics by comparing with cache
    async fn update_scan_statistics(&self, scan_result: &mut ScanResult) -> OperatorResult<()> {
        let cached_names = self.cache.get_cached_operator_names().await?;
        let cached_set: std::collections::HashSet<_> = cached_names.into_iter().collect();
        
        for operator in &scan_result.operators {
            if cached_set.contains(&operator.name) {
                // Check if this is an update by comparing with cached version
                match self.cache.get_operator(&operator.name).await? {
                    Some(cached_op) if cached_op != *operator => {
                        scan_result.updated_count += 1;
                    },
                    Some(_) => {
                        // No changes detected
                    },
                    None => {
                        // Shouldn't happen, but treat as new
                        scan_result.new_count += 1;
                    }
                }
            } else {
                scan_result.new_count += 1;
            }
        }
        
        Ok(())
    }
    
    /// Cache scan results
    async fn cache_scan_results(&self, scan_result: &ScanResult) -> OperatorResult<()> {
        // Store individual operators
        self.cache.store_operators_batch(scan_result.operators.clone()).await?;
        
        // Store scan result
        self.cache.store_scan_result(scan_result.clone()).await?;
        
        debug!("Cached scan results: {} operators", scan_result.operators.len());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maa_adapter::MaaStatus;
    use crate::operator_manager::cache::{OperatorCache, CacheConfig};
    use tempfile::TempDir;
    use std::sync::Arc;
    use async_trait::async_trait;
    
    // Mock MAA adapter for testing
    struct MockMaaAdapter {
        should_fail: bool,
        call_count: std::sync::Mutex<u32>,
    }
    
    #[async_trait]
    impl MaaAdapterTrait for MockMaaAdapter {
        async fn new(_config: crate::maa_adapter::MaaConfig) -> Result<Self, crate::maa_adapter::MaaError> {
            Ok(Self { 
                should_fail: false,
                call_count: std::sync::Mutex::new(0),
            })
        }
        
        async fn connect(&mut self, _device: &str) -> Result<(), crate::maa_adapter::MaaError> {
            Ok(())
        }
        
        async fn disconnect(&mut self) -> Result<(), crate::maa_adapter::MaaError> {
            Ok(())
        }
        
        async fn screenshot(&self) -> Result<Vec<u8>, crate::maa_adapter::MaaError> {
            Ok(vec![])
        }
        
        async fn click(&self, _x: i32, _y: i32) -> Result<(), crate::maa_adapter::MaaError> {
            Ok(())
        }
        
        async fn swipe(&self, _from_x: i32, _from_y: i32, _to_x: i32, _to_y: i32, _duration: u32) -> Result<(), crate::maa_adapter::MaaError> {
            Ok(())
        }
        
        async fn create_task(&self, _task_type: MaaTaskType, _params: TaskParams) -> Result<i32, crate::maa_adapter::MaaError> {
            Ok(1)
        }
        
        async fn start_task(&self, _task_id: i32) -> Result<(), crate::maa_adapter::MaaError> {
            Ok(())
        }
        
        async fn stop_task(&self, _task_id: i32) -> Result<(), crate::maa_adapter::MaaError> {
            Ok(())
        }
        
        async fn get_status(&self) -> Result<MaaStatus, crate::maa_adapter::MaaError> {
            let mut count = self.call_count.lock().unwrap();
            *count += 1;
            
            if self.should_fail {
                Ok(MaaStatus::Failed { 
                    task_id: 1, 
                    error: "Mock error".to_string(),
                    failed_at: chrono::Utc::now(),
                })
            } else if *count == 1 {
                // First call - return Idle for availability check
                Ok(MaaStatus::Idle)
            } else {
                // Subsequent calls - return Completed for scan results
                Ok(MaaStatus::Completed { 
                    task_id: 1, 
                    result: r#"{"operators": [
                        {
                            "name": "Amiya",
                            "class": "Caster",
                            "rarity": 5,
                            "elite": 2,
                            "level": 80,
                            "skills": [
                                {"index": 0, "level": 7},
                                {"index": 1, "level": 7},
                                {"index": 2, "level": 7}
                            ],
                            "modules": [],
                            "potential": 6,
                            "trust": 200
                        }
                    ]}"#.to_string(),
                    completed_at: chrono::Utc::now(),
                })
            }
        }
        
        async fn get_task(&self, _task_id: i32) -> Result<Option<crate::maa_adapter::MaaTask>, crate::maa_adapter::MaaError> {
            Ok(None)
        }
        
        async fn get_all_tasks(&self) -> Result<Vec<crate::maa_adapter::MaaTask>, crate::maa_adapter::MaaError> {
            Ok(vec![])
        }
        
        async fn get_device_info(&self) -> Result<Option<crate::maa_adapter::DeviceInfo>, crate::maa_adapter::MaaError> {
            Ok(None)
        }
    }
    
    async fn create_test_scanner() -> (OperatorScanner, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let cache_config = CacheConfig {
            db_path: temp_dir.path().join("test_cache").to_string_lossy().to_string(),
            ..CacheConfig::default()
        };
        
        let cache = Arc::new(OperatorCache::new(cache_config).await.unwrap());
        let maa_adapter = Arc::new(MockMaaAdapter { 
            should_fail: false,
            call_count: std::sync::Mutex::new(0),
        });
        let scanner_config = ScannerConfig::default();
        
        let scanner = OperatorScanner::new(maa_adapter, cache, scanner_config);
        (scanner, temp_dir)
    }
    
    #[tokio::test]
    async fn test_scanner_creation() {
        let (scanner, _temp_dir) = create_test_scanner().await;
        assert_eq!(scanner.config.scan_timeout_seconds, 60);
        assert!(scanner.config.full_scan);
    }
    
    #[tokio::test]
    async fn test_scan_availability_check() {
        let (scanner, _temp_dir) = create_test_scanner().await;
        let available = scanner.check_scan_availability().await.unwrap();
        // Mock adapter should return true for availability
        assert!(available);
    }
    
    #[tokio::test]
    async fn test_scan_operators() {
        let (scanner, _temp_dir) = create_test_scanner().await;
        
        let result = scanner.scan_operators().await.unwrap();
        assert_eq!(result.operators.len(), 1);
        assert_eq!(result.operators[0].name, "Amiya");
        assert_eq!(result.operators[0].profession, "Caster");
        assert_eq!(result.operators[0].rarity, 5);
        assert_eq!(result.operators[0].elite, 2);
        assert_eq!(result.operators[0].level, 80);
    }
    
    #[tokio::test]
    async fn test_parse_single_operator() {
        let (scanner, _temp_dir) = create_test_scanner().await;
        
        let raw_operator = RawOperatorData {
            name: "SilverAsh".to_string(),
            profession: Some("Guard".to_string()),
            rarity: Some(6),
            elite: Some(2),
            level: Some(90),
            skills: Some(vec![
                RawSkillData { index: 0, level: 7, name: None },
                RawSkillData { index: 1, level: 7, name: None },
                RawSkillData { index: 2, level: 7, name: None }, // Max level 7
            ]),
            modules: Some(vec![
                RawModuleData {
                    name: "GUA-Y".to_string(),
                    level: 3,
                    unlocked: true,
                    stage: Some("Y".to_string()),
                }
            ]),
            potential: Some(1),
            trust: Some(180),
            raw_data: HashMap::new(),
        };
        
        let operator = scanner.parse_single_operator(raw_operator).await.unwrap();
        assert_eq!(operator.name, "SilverAsh");
        assert_eq!(operator.skill_levels, vec![7, 7, 7]);
        assert_eq!(operator.modules.len(), 1);
        assert_eq!(operator.modules[0].module_type, "GUA-Y");
    }
    
    #[tokio::test]
    async fn test_validate_operator() {
        let (scanner, _temp_dir) = create_test_scanner().await;
        
        // Valid operator
        let valid_operator = Operator::new("Test".to_string(), "Guard".to_string(), 6);
        assert!(scanner.validate_operator(&valid_operator).is_ok());
        
        // Invalid rarity
        let mut invalid_operator = valid_operator.clone();
        invalid_operator.rarity = 7;
        assert!(scanner.validate_operator(&invalid_operator).is_err());
        
        // Invalid elite level
        let mut invalid_operator = valid_operator.clone();
        invalid_operator.elite = 3;
        assert!(scanner.validate_operator(&invalid_operator).is_err());
        
        // Invalid level for elite
        let mut invalid_operator = valid_operator.clone();
        invalid_operator.elite = 0;
        invalid_operator.level = 80; // Too high for Elite 0
        assert!(scanner.validate_operator(&invalid_operator).is_err());
        
        // Invalid skill level
        let mut invalid_operator = valid_operator.clone();
        invalid_operator.skill_levels = vec![7, 8, 5]; // 8 is invalid
        assert!(scanner.validate_operator(&invalid_operator).is_err());
        
        // Invalid potential
        let mut invalid_operator = valid_operator.clone();
        invalid_operator.potential = 7;
        assert!(scanner.validate_operator(&invalid_operator).is_err());
        
        // Invalid trust
        let mut invalid_operator = valid_operator.clone();
        invalid_operator.trust = 201;
        assert!(scanner.validate_operator(&invalid_operator).is_err());
    }
    
    #[tokio::test]
    async fn test_scan_task_params() {
        let (scanner, _temp_dir) = create_test_scanner().await;
        
        let params = scanner.create_scan_params().unwrap();
        
        let parsed_params: serde_json::Value = serde_json::from_str(&params.raw).unwrap();
        assert_eq!(parsed_params["type"], "operator_scan");
        assert_eq!(parsed_params["full_scan"], true);
        assert_eq!(parsed_params["include_details"], true);
    }
    
    #[tokio::test]
    async fn test_targeted_scan_params() {
        let (scanner, _temp_dir) = create_test_scanner().await;
        
        let params = scanner.create_targeted_scan_params("Amiya").unwrap();
        
        let parsed_params: serde_json::Value = serde_json::from_str(&params.raw).unwrap();
        assert_eq!(parsed_params["target_operator"], "Amiya");
    }
}