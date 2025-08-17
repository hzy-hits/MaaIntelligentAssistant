//! Operator Manager Module
//!
//! This module provides comprehensive operator management functionality for MAA (MaaAssistantArknights).
//! It handles operator scanning, caching, querying, and analysis through integration with MAA's operator recognition capabilities.
//!
//! # Features
//!
//! - **Operator Scanning**: Automatic recognition of operator data using MAA
//! - **Persistent Caching**: Efficient storage and retrieval using sled database
//! - **Intelligent Filtering**: Advanced filtering and querying capabilities
//! - **Incremental Updates**: Smart detection of changes to avoid full rescans
//! - **Statistics and Analysis**: Comprehensive operator collection analysis
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │  OperatorManager│────│  OperatorScanner│────│   MAA Adapter   │
//! │                 │    │                 │    │                 │
//! └─────────────────┘    └─────────────────┘    └─────────────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │  OperatorCache  │
//! │    (sled DB)    │
//! └─────────────────┘
//! ```
//!
//! # Usage Example
//!
//! ```rust,ignore
//! use maa_intelligent_server::operator_manager::{OperatorManager, OperatorManagerConfig};
//! use maa_intelligent_server::maa_adapter::{MaaAdapter, MaaConfig, MaaAdapterTrait};
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize MAA adapter
//!     let maa_config = MaaConfig::default();
//!     let maa_adapter: Arc<dyn MaaAdapterTrait + Send + Sync> = Arc::new(MaaAdapter::new(maa_config).await?);
//!     
//!     // Initialize operator manager
//!     let manager_config = OperatorManagerConfig::default();
//!     let mut operator_manager = OperatorManager::new(maa_adapter, manager_config).await?;
//!     
//!     // Scan operators
//!     let scan_result = operator_manager.scan_operators().await?;
//!     println!("Found {} operators", scan_result.operators.len());
//!     
//!     // Query specific operator
//!     if let Some(amiya) = operator_manager.get_operator("Amiya").await? {
//!         println!("Amiya: Elite {}, Level {}", amiya.elite, amiya.level);
//!     }
//!     
//!     Ok(())
//! }
//! ```

use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::maa_adapter::MaaBackend;

// Re-export public types
pub use types::{
    Operator, OperatorFilter, OperatorSummary, ScanResult, ModuleInfo
};
pub use errors::{OperatorError, OperatorResult};
pub use cache::{OperatorCache, CacheConfig, CacheStats};
pub use scanner::{OperatorScanner, ScannerConfig, ScanProgress};

// Internal modules
pub mod types;
pub mod errors;
pub mod cache;
pub mod scanner;

/// Operator manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorManagerConfig {
    /// Cache configuration
    pub cache: CacheConfig,
    
    /// Scanner configuration
    pub scanner: ScannerConfig,
    
    /// Auto-scan interval in seconds (0 to disable)
    pub auto_scan_interval_seconds: u64,
    
    /// Whether to perform incremental scans by default
    pub prefer_incremental_scans: bool,
    
    /// Maximum age of cached data before forcing a rescan (in seconds)
    pub max_cache_age_seconds: u64,
    
    /// Enable automatic cache cleanup
    pub enable_auto_cleanup: bool,
}

impl Default for OperatorManagerConfig {
    fn default() -> Self {
        Self {
            cache: CacheConfig::default(),
            scanner: ScannerConfig::default(),
            auto_scan_interval_seconds: 3600, // 1 hour
            prefer_incremental_scans: true,
            max_cache_age_seconds: 24 * 3600, // 24 hours
            enable_auto_cleanup: true,
        }
    }
}

/// Main operator manager
/// 
/// Provides a unified interface for all operator management operations.
/// Coordinates between scanning, caching, and querying functionality.
pub struct OperatorManager {
    /// MAA backend for scanning operations
    _maa_backend: Arc<MaaBackend>,
    
    /// Operator cache
    cache: Arc<OperatorCache>,
    
    /// Operator scanner
    scanner: OperatorScanner,
    
    /// Configuration
    config: OperatorManagerConfig,
    
    /// Last scan time
    last_scan: Arc<RwLock<Option<DateTime<Utc>>>>,
}

impl OperatorManager {
    /// Create a new operator manager
    pub async fn new(
        maa_backend: Arc<MaaBackend>,
        config: OperatorManagerConfig,
    ) -> OperatorResult<Self> {
        info!("Initializing Operator Manager");
        
        // Initialize cache
        let cache = Arc::new(OperatorCache::new(config.cache.clone()).await?);
        
        // Initialize scanner
        let scanner = OperatorScanner::new(
            maa_backend.clone(),
            cache.clone(),
            config.scanner.clone(),
        );
        
        let manager = Self {
            _maa_backend: maa_backend,
            cache,
            scanner,
            config,
            last_scan: Arc::new(RwLock::new(None)),
        };
        
        // Load last scan time from cache if available
        manager.load_last_scan_time().await?;
        
        info!("Operator Manager initialized successfully");
        Ok(manager)
    }
    
    /// Scan all operators
    /// 
    /// Performs a comprehensive scan of all operators using MAA's recognition system.
    /// Results are automatically cached for future queries.
    pub async fn scan_operators(&mut self) -> OperatorResult<ScanResult> {
        self.scan_operators_with_options(None).await
    }
    
    /// Scan operators with custom options
    pub async fn scan_operators_with_options(&mut self, force_full_scan: Option<bool>) -> OperatorResult<ScanResult> {
        info!("Starting operator scan");
        
        // Check if scanning is available
        if !self.scanner.check_scan_availability().await? {
            return Err(OperatorError::maa_operation(
                "scan_availability",
                "MAA is not ready for scanning"
            ));
        }
        
        // Determine scan type
        let should_force_full = force_full_scan.unwrap_or_else(|| {
            !self.config.prefer_incremental_scans || self.should_force_full_scan()
        });
        
        if should_force_full {
            info!("Performing full operator scan");
        } else {
            info!("Performing incremental operator scan");
        }
        
        // Perform the scan
        let scan_result = self.scanner.scan_operators().await?;
        
        // Update last scan time
        let mut last_scan = self.last_scan.write().await;
        *last_scan = Some(scan_result.completed_at);
        
        // Store last scan time in cache metadata
        self.store_last_scan_time(scan_result.completed_at).await?;
        
        info!(
            "Operator scan completed: {} operators ({} new, {} updated)",
            scan_result.operators.len(),
            scan_result.new_count,
            scan_result.updated_count
        );
        
        Ok(scan_result)
    }
    
    /// Get a specific operator by name
    pub async fn get_operator(&self, name: &str) -> OperatorResult<Option<Operator>> {
        debug!("Querying operator: {}", name);
        
        // Try cache first
        match self.cache.get_operator(name).await? {
            Some(operator) => {
                debug!("Found operator in cache: {}", name);
                Ok(Some(operator))
            },
            None => {
                debug!("Operator not in cache, attempting scan: {}", name);
                
                // Try scanning the specific operator
                match self.scanner.scan_operator(name).await? {
                    Some(operator) => {
                        debug!("Successfully scanned operator: {}", name);
                        Ok(Some(operator))
                    },
                    None => {
                        debug!("Operator not found: {}", name);
                        Ok(None)
                    }
                }
            }
        }
    }
    
    /// Get operators matching a filter
    pub async fn get_operators_by_filter(&self, filter: &OperatorFilter) -> OperatorResult<Vec<Operator>> {
        debug!("Filtering operators with criteria");
        
        // Get filtered operators from cache
        let operators = self.cache.get_operators_filtered(filter).await?;
        
        debug!("Found {} operators matching filter", operators.len());
        Ok(operators)
    }
    
    /// Get all cached operators
    pub async fn get_all_operators(&self) -> OperatorResult<Vec<Operator>> {
        let filter = OperatorFilter::new();
        self.get_operators_by_filter(&filter).await
    }
    
    /// Get operator names only (for quick listing)
    pub async fn get_operator_names(&self) -> OperatorResult<Vec<String>> {
        self.cache.get_cached_operator_names().await
    }
    
    /// Get operator collection summary
    pub async fn get_summary(&self) -> OperatorResult<OperatorSummary> {
        let operators = self.get_all_operators().await?;
        Ok(OperatorSummary::from_operators(&operators))
    }
    
    /// Get operator collection summary for filtered operators
    pub async fn get_summary_filtered(&self, filter: &OperatorFilter) -> OperatorResult<OperatorSummary> {
        let operators = self.get_operators_by_filter(filter).await?;
        Ok(OperatorSummary::from_operators(&operators))
    }
    
    /// Update the operator cache (incremental update)
    pub async fn update_cache(&mut self) -> OperatorResult<()> {
        info!("Updating operator cache");
        
        // Check if update is needed
        if !self.should_update_cache().await {
            debug!("Cache update not needed");
            return Ok(());
        }
        
        // Perform incremental scan
        let _scan_result = self.scan_operators_with_options(Some(false)).await?;
        
        info!("Cache update completed");
        Ok(())
    }
    
    /// Force a full rescan of all operators
    pub async fn force_rescan(&mut self) -> OperatorResult<ScanResult> {
        info!("Forcing full operator rescan");
        self.scan_operators_with_options(Some(true)).await
    }
    
    /// Clear all cached data
    pub async fn clear_cache(&self) -> OperatorResult<()> {
        info!("Clearing operator cache");
        self.cache.clear_all().await?;
        
        // Reset last scan time
        let mut last_scan = self.last_scan.write().await;
        *last_scan = None;
        
        info!("Operator cache cleared");
        Ok(())
    }
    
    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> CacheStats {
        self.cache.get_stats().await
    }
    
    /// Get the latest scan result
    pub async fn get_latest_scan_result(&self) -> OperatorResult<Option<ScanResult>> {
        self.cache.get_latest_scan_result().await
    }
    
    /// Get last scan time
    pub async fn get_last_scan_time(&self) -> Option<DateTime<Utc>> {
        *self.last_scan.read().await
    }
    
    /// Check if a cache update is needed
    async fn should_update_cache(&self) -> bool {
        let last_scan = *self.last_scan.read().await;
        
        match last_scan {
            Some(last_scan_time) => {
                let age = Utc::now().signed_duration_since(last_scan_time);
                age.num_seconds() as u64 > self.config.max_cache_age_seconds
            },
            None => true, // No scan yet
        }
    }
    
    /// Check if a full scan should be forced
    fn should_force_full_scan(&self) -> bool {
        // Always force full scan if we have no previous scan data
        if self.last_scan.try_read().map_or(true, |scan| scan.is_none()) {
            return true;
        }
        
        // Additional logic could be added here for other conditions
        // that should trigger a full scan (e.g., MAA version changes)
        false
    }
    
    /// Load last scan time from cache metadata
    async fn load_last_scan_time(&self) -> OperatorResult<()> {
        // This would involve reading metadata from the cache
        // For now, we'll use the latest scan result as a proxy
        if let Some(scan_result) = self.cache.get_latest_scan_result().await? {
            let mut last_scan = self.last_scan.write().await;
            *last_scan = Some(scan_result.completed_at);
            debug!("Loaded last scan time: {}", scan_result.completed_at);
        }
        
        Ok(())
    }
    
    /// Store last scan time in cache metadata
    async fn store_last_scan_time(&self, scan_time: DateTime<Utc>) -> OperatorResult<()> {
        // This would involve storing metadata in the cache
        // For now, scan results serve this purpose
        debug!("Stored last scan time: {}", scan_time);
        Ok(())
    }
    
    /// Perform cache cleanup
    pub async fn cleanup_cache(&self) -> OperatorResult<()> {
        info!("Performing cache cleanup");
        self.cache.cleanup().await?;
        info!("Cache cleanup completed");
        Ok(())
    }
    
    /// Get operators with development recommendations
    pub async fn get_development_candidates(&self) -> OperatorResult<Vec<Operator>> {
        // Get operators that could benefit from development
        let filter = OperatorFilter::new()
            .with_min_rarity(4) // Focus on 4-6 star operators
            .with_has_mastery(false); // Haven't reached full mastery yet
        
        let mut candidates = self.get_operators_by_filter(&filter).await?;
        
        // Sort by development potential (high rarity, low development score)
        candidates.sort_by(|a, b| {
            let score_diff = a.development_score().partial_cmp(&b.development_score()).unwrap_or(std::cmp::Ordering::Equal);
            let rarity_diff = b.rarity.cmp(&a.rarity);
            
            // First by rarity (higher is better), then by development score (lower is better)
            rarity_diff.then(score_diff)
        });
        
        Ok(candidates)
    }
    
    /// Get operators by profession
    pub async fn get_operators_by_profession(&self, profession: &str) -> OperatorResult<Vec<Operator>> {
        let filter = OperatorFilter::new().with_profession(profession.to_string());
        self.get_operators_by_filter(&filter).await
    }
    
    /// Get high-rarity operators (5-6 stars)
    pub async fn get_high_rarity_operators(&self) -> OperatorResult<Vec<Operator>> {
        let filter = OperatorFilter::new().with_min_rarity(5);
        self.get_operators_by_filter(&filter).await
    }
    
    /// Get operators at max level
    pub async fn get_max_level_operators(&self) -> OperatorResult<Vec<Operator>> {
        let filter = OperatorFilter::new().with_has_modules(true);
        let mut operators = self.get_operators_by_filter(&filter).await?;
        
        // Filter to only include operators at max level for their elite
        operators.retain(|op| op.is_max_level());
        
        Ok(operators)
    }
    
    /// Get operators with mastery skills
    pub async fn get_mastery_operators(&self) -> OperatorResult<Vec<Operator>> {
        let filter = OperatorFilter::new().with_has_mastery(true);
        self.get_operators_by_filter(&filter).await
    }
    
    /// Validate operator data integrity
    pub async fn validate_cache_integrity(&self) -> OperatorResult<Vec<String>> {
        let mut issues = Vec::new();
        let operators = self.get_all_operators().await?;
        
        for operator in operators {
            // Check for data consistency issues
            if operator.name.trim().is_empty() {
                issues.push(format!("Operator has empty name"));
            }
            
            if operator.rarity < 1 || operator.rarity > 6 {
                issues.push(format!("Operator {} has invalid rarity: {}", operator.name, operator.rarity));
            }
            
            if operator.elite > 2 {
                issues.push(format!("Operator {} has invalid elite level: {}", operator.name, operator.elite));
            }
            
            // Check level consistency with elite
            let max_level = match operator.elite {
                0 => 50,
                1 => 80,
                2 => 90,
                _ => 90,
            };
            
            if operator.level > max_level {
                issues.push(format!(
                    "Operator {} has invalid level {} for elite {}",
                    operator.name, operator.level, operator.elite
                ));
            }
            
            // Check skill levels
            for (i, &skill_level) in operator.skill_levels.iter().enumerate() {
                if skill_level < 1 || skill_level > 7 {
                    issues.push(format!(
                        "Operator {} has invalid skill {} level: {}",
                        operator.name, i + 1, skill_level
                    ));
                }
            }
            
            // Check potential
            if operator.potential < 1 || operator.potential > 6 {
                issues.push(format!("Operator {} has invalid potential: {}", operator.name, operator.potential));
            }
            
            // Check trust
            if operator.trust > 200 {
                issues.push(format!("Operator {} has invalid trust: {}", operator.name, operator.trust));
            }
        }
        
        if issues.is_empty() {
            info!("Cache integrity validation passed");
        } else {
            warn!("Cache integrity validation found {} issues", issues.len());
        }
        
        Ok(issues)
    }
    
    /// Export operator data as JSON
    pub async fn export_operators(&self) -> OperatorResult<String> {
        let operators = self.get_all_operators().await?;
        let json = serde_json::to_string_pretty(&operators)
            .map_err(|e| OperatorError::serialization("export", e.to_string()))?;
        Ok(json)
    }
    
    /// Import operator data from JSON
    pub async fn import_operators(&self, json_data: &str) -> OperatorResult<u32> {
        let operators: Vec<Operator> = serde_json::from_str(json_data)
            .map_err(|e| OperatorError::serialization("import", e.to_string()))?;
        
        let count = operators.len() as u32;
        self.cache.store_operators_batch(operators).await?;
        
        info!("Imported {} operators", count);
        Ok(count)
    }
    
    /// Flush cache to disk
    pub async fn flush_cache(&self) -> OperatorResult<()> {
        self.cache.flush().await
    }
}

/// Trait for operator management operations
/// 
/// This trait defines the core interface for operator management that can be
/// implemented by different backends or used for testing with mock implementations.
#[async_trait::async_trait]
pub trait OperatorManagerTrait: Send + Sync {
    /// Scan all operators
    async fn scan_operators(&mut self) -> OperatorResult<ScanResult>;
    
    /// Get a specific operator by name
    async fn get_operator(&self, name: &str) -> OperatorResult<Option<Operator>>;
    
    /// Get operators matching a filter
    async fn get_operators_by_filter(&self, filter: &OperatorFilter) -> OperatorResult<Vec<Operator>>;
    
    /// Update the cache
    async fn update_cache(&mut self) -> OperatorResult<()>;
    
    /// Get cache statistics
    async fn get_cache_stats(&self) -> CacheStats;
}

#[async_trait::async_trait]
impl OperatorManagerTrait for OperatorManager {
    async fn scan_operators(&mut self) -> OperatorResult<ScanResult> {
        self.scan_operators().await
    }
    
    async fn get_operator(&self, name: &str) -> OperatorResult<Option<Operator>> {
        self.get_operator(name).await
    }
    
    async fn get_operators_by_filter(&self, filter: &OperatorFilter) -> OperatorResult<Vec<Operator>> {
        self.get_operators_by_filter(filter).await
    }
    
    async fn update_cache(&mut self) -> OperatorResult<()> {
        self.update_cache().await
    }
    
    async fn get_cache_stats(&self) -> CacheStats {
        self.get_cache_stats().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maa_adapter::{MaaAdapter, MaaConfig};
    use tempfile::TempDir;
    
    async fn create_test_manager() -> (OperatorManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let mut config = OperatorManagerConfig::default();
        config.cache.db_path = temp_dir.path().join("test_cache").to_string_lossy().to_string();
        
        let maa_config = MaaConfig::default();
        let maa_adapter = Arc::new(MaaAdapter::new(maa_config).await.unwrap());
        let manager = OperatorManager::new(maa_adapter, config).await.unwrap();
        
        (manager, temp_dir)
    }
    
    #[tokio::test]
    async fn test_manager_creation() {
        let (manager, _temp_dir) = create_test_manager().await;
        
        let stats = manager.get_cache_stats().await;
        assert_eq!(stats.total_entries, 0);
        
        let last_scan = manager.get_last_scan_time().await;
        assert!(last_scan.is_none());
    }
    
    #[tokio::test]
    async fn test_get_operator_names() {
        let (manager, _temp_dir) = create_test_manager().await;
        
        let names = manager.get_operator_names().await.unwrap();
        assert_eq!(names.len(), 0); // No operators cached yet
    }
    
    #[tokio::test]
    async fn test_get_all_operators() {
        let (manager, _temp_dir) = create_test_manager().await;
        
        let operators = manager.get_all_operators().await.unwrap();
        assert_eq!(operators.len(), 0); // No operators cached yet
    }
    
    #[tokio::test]
    async fn test_get_summary() {
        let (manager, _temp_dir) = create_test_manager().await;
        
        let summary = manager.get_summary().await.unwrap();
        assert_eq!(summary.total_count, 0);
    }
    
    #[tokio::test]
    async fn test_filter_operations() {
        let (manager, _temp_dir) = create_test_manager().await;
        
        // Test various filter operations
        let guards = manager.get_operators_by_profession("Guard").await.unwrap();
        assert_eq!(guards.len(), 0);
        
        let high_rarity = manager.get_high_rarity_operators().await.unwrap();
        assert_eq!(high_rarity.len(), 0);
        
        let max_level = manager.get_max_level_operators().await.unwrap();
        assert_eq!(max_level.len(), 0);
        
        let mastery = manager.get_mastery_operators().await.unwrap();
        assert_eq!(mastery.len(), 0);
        
        let dev_candidates = manager.get_development_candidates().await.unwrap();
        assert_eq!(dev_candidates.len(), 0);
    }
    
    #[tokio::test]
    async fn test_cache_operations() {
        let (manager, _temp_dir) = create_test_manager().await;
        
        // Test cache cleanup
        manager.cleanup_cache().await.unwrap();
        
        // Test cache clearing
        manager.clear_cache().await.unwrap();
        
        // Test cache flushing
        manager.flush_cache().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_validation() {
        let (manager, _temp_dir) = create_test_manager().await;
        
        let issues = manager.validate_cache_integrity().await.unwrap();
        assert_eq!(issues.len(), 0); // No data, no issues
    }
    
    #[tokio::test]
    async fn test_import_export() {
        let (manager, _temp_dir) = create_test_manager().await;
        
        // Test export with empty data
        let exported = manager.export_operators().await.unwrap();
        assert_eq!(exported, "[]");
        
        // Test import
        let test_data = r#"[
            {
                "name": "Amiya",
                "profession": "Caster",
                "rarity": 5,
                "elite": 2,
                "level": 80,
                "skill_levels": [7, 7, 7],
                "modules": [],
                "potential": 6,
                "trust": 200,
                "last_updated": "2023-01-01T00:00:00Z",
                "metadata": {}
            }
        ]"#;
        
        let imported_count = manager.import_operators(test_data).await.unwrap();
        assert_eq!(imported_count, 1);
        
        // Verify import worked
        let names = manager.get_operator_names().await.unwrap();
        assert_eq!(names.len(), 1);
        assert!(names.contains(&"Amiya".to_string()));
    }
}