//! Operator Cache Management
//!
//! This module provides persistent caching functionality for operator data using sled database.
//! It handles serialization, storage, retrieval, and cache invalidation with efficient indexing.

use std::path::Path;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sled::{Db, Tree};
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::operator_manager::{
    types::{Operator, OperatorFilter, ScanResult},
    errors::{OperatorError, OperatorResult},
};

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Database path
    pub db_path: String,
    
    /// Maximum cache size in bytes (0 = unlimited)
    pub max_size_bytes: u64,
    
    /// Default TTL for cached entries in seconds
    pub default_ttl_seconds: u64,
    
    /// Enable compression for stored data
    pub enable_compression: bool,
    
    /// Periodic cleanup interval in seconds
    pub cleanup_interval_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            db_path: "data/operator_cache".to_string(),
            max_size_bytes: 100 * 1024 * 1024, // 100MB
            default_ttl_seconds: 24 * 60 * 60,  // 24 hours
            enable_compression: true,
            cleanup_interval_seconds: 60 * 60,  // 1 hour
        }
    }
}

/// Cache entry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry<T> {
    /// The cached data
    data: T,
    
    /// When this entry was created
    created_at: DateTime<Utc>,
    
    /// When this entry was last accessed
    last_accessed: DateTime<Utc>,
    
    /// When this entry expires (None = never expires)
    expires_at: Option<DateTime<Utc>>,
    
    /// Entry version for conflict detection
    version: u64,
    
    /// Size of the entry in bytes
    size_bytes: u64,
}

impl<T> CacheEntry<T> {
    fn new(data: T, ttl_seconds: Option<u64>) -> Self 
    where 
        T: Serialize,
    {
        let now = Utc::now();
        let expires_at = ttl_seconds.map(|ttl| now + chrono::Duration::seconds(ttl as i64));
        
        // Estimate size (not exact, but good enough for cache management)
        let size_bytes = bincode::serde::encode_to_vec(&data, bincode::config::standard())
            .map(|bytes| bytes.len() as u64)
            .unwrap_or(0);
        
        Self {
            data,
            created_at: now,
            last_accessed: now,
            expires_at,
            version: 1,
            size_bytes,
        }
    }
    
    fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires_at) => Utc::now() > expires_at,
            None => false,
        }
    }
    
    fn touch(&mut self) {
        self.last_accessed = Utc::now();
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total number of entries
    pub total_entries: u64,
    
    /// Total cache size in bytes
    pub total_size_bytes: u64,
    
    /// Cache hit count
    pub hit_count: u64,
    
    /// Cache miss count
    pub miss_count: u64,
    
    /// Number of expired entries
    pub expired_entries: u64,
    
    /// Last cleanup time
    pub last_cleanup: Option<DateTime<Utc>>,
    
    /// Hit ratio (0.0 - 1.0)
    pub hit_ratio: f64,
}

impl CacheStats {
    fn new() -> Self {
        Self {
            total_entries: 0,
            total_size_bytes: 0,
            hit_count: 0,
            miss_count: 0,
            expired_entries: 0,
            last_cleanup: None,
            hit_ratio: 0.0,
        }
    }
    
    fn calculate_hit_ratio(&mut self) {
        let total_requests = self.hit_count + self.miss_count;
        self.hit_ratio = if total_requests > 0 {
            self.hit_count as f64 / total_requests as f64
        } else {
            0.0
        };
    }
}

/// Operator cache manager
pub struct OperatorCache {
    /// Main database
    db: Db,
    
    /// Operators tree (key: operator_name, value: CacheEntry<Operator>)
    operators: Tree,
    
    /// Summary tree (key: filter_hash, value: CacheEntry<OperatorSummary>)
    summaries: Tree,
    
    /// Scan results tree (key: timestamp, value: CacheEntry<ScanResult>)
    scan_results: Tree,
    
    /// Metadata tree (key: meta_key, value: various metadata)
    _metadata: Tree,
    
    /// Cache configuration
    config: CacheConfig,
    
    /// Cache statistics
    stats: Arc<RwLock<CacheStats>>,
}

impl OperatorCache {
    /// Create a new operator cache
    pub async fn new(config: CacheConfig) -> OperatorResult<Self> {
        // Ensure the directory exists
        if let Some(parent) = Path::new(&config.db_path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| OperatorError::resource_access("cache_directory", e.to_string()))?;
        }
        
        // Open the database
        let db = sled::open(&config.db_path)
            .map_err(|e| OperatorError::cache("open", e.to_string()))?;
        
        // Open trees
        let operators = db.open_tree("operators")
            .map_err(|e| OperatorError::cache("open_tree", e.to_string()))?;
        
        let summaries = db.open_tree("summaries")
            .map_err(|e| OperatorError::cache("open_tree", e.to_string()))?;
        
        let scan_results = db.open_tree("scan_results")
            .map_err(|e| OperatorError::cache("open_tree", e.to_string()))?;
        
        let metadata = db.open_tree("metadata")
            .map_err(|e| OperatorError::cache("open_tree", e.to_string()))?;
        
        // Load or initialize stats
        let stats = Arc::new(RwLock::new(CacheStats::new()));
        
        let cache = Self {
            db,
            operators,
            summaries,
            scan_results,
            _metadata: metadata,
            config,
            stats,
        };
        
        // Update stats
        cache.update_stats().await?;
        
        info!("Operator cache initialized at: {}", cache.config.db_path);
        Ok(cache)
    }
    
    /// Store an operator in the cache
    pub async fn store_operator(&self, operator: Operator) -> OperatorResult<()> {
        let key = operator.name.clone().into_bytes();
        let entry = CacheEntry::new(operator.clone(), Some(self.config.default_ttl_seconds));
        
        let data = bincode::serde::encode_to_vec(&entry, bincode::config::standard())
            .map_err(|e| OperatorError::serialization("encode", e.to_string()))?;
        
        self.operators.insert(key, data)
            .map_err(|e| OperatorError::cache_with_key("store", e.to_string(), operator.name.clone()))?;
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_entries += 1;
        stats.total_size_bytes += entry.size_bytes;
        
        debug!("Stored operator in cache: {}", operator.name);
        Ok(())
    }
    
    /// Retrieve an operator from the cache
    pub async fn get_operator(&self, name: &str) -> OperatorResult<Option<Operator>> {
        let key = name.as_bytes();
        
        match self.operators.get(key)
            .map_err(|e| OperatorError::cache_with_key("get", e.to_string(), name.to_string()))? 
        {
            Some(data) => {
                let (mut entry, _): (CacheEntry<Operator>, _) = bincode::serde::decode_from_slice(&data, bincode::config::standard())
                    .map_err(|e| OperatorError::serialization("decode", e.to_string()))?;
                
                // Check if expired
                if entry.is_expired() {
                    self.operators.remove(key)
                        .map_err(|e| OperatorError::cache_with_key("remove_expired", e.to_string(), name.to_string()))?;
                    
                    let mut stats = self.stats.write().await;
                    stats.miss_count += 1;
                    stats.expired_entries += 1;
                    stats.calculate_hit_ratio();
                    
                    debug!("Operator cache entry expired: {}", name);
                    return Ok(None);
                }
                
                // Update access time
                entry.touch();
                let updated_data = bincode::serde::encode_to_vec(&entry, bincode::config::standard())
                    .map_err(|e| OperatorError::serialization("encode", e.to_string()))?;
                
                self.operators.insert(key, updated_data)
                    .map_err(|e| OperatorError::cache_with_key("update_access", e.to_string(), name.to_string()))?;
                
                // Update stats
                let mut stats = self.stats.write().await;
                stats.hit_count += 1;
                stats.calculate_hit_ratio();
                
                debug!("Retrieved operator from cache: {}", name);
                Ok(Some(entry.data))
            },
            None => {
                let mut stats = self.stats.write().await;
                stats.miss_count += 1;
                stats.calculate_hit_ratio();
                
                debug!("Operator not found in cache: {}", name);
                Ok(None)
            }
        }
    }
    
    /// Store multiple operators in batch
    pub async fn store_operators_batch(&self, operators: Vec<Operator>) -> OperatorResult<()> {
        let mut batch = sled::Batch::default();
        let mut total_size = 0u64;
        let operators_count = operators.len();
        
        for operator in &operators {
            let key = operator.name.clone().into_bytes();
            let entry = CacheEntry::new(operator.clone(), Some(self.config.default_ttl_seconds));
            total_size += entry.size_bytes;
            
            let data = bincode::serde::encode_to_vec(&entry, bincode::config::standard())
                .map_err(|e| OperatorError::serialization("encode", e.to_string()))?;
            
            batch.insert(key, data);
        }
        
        self.operators.apply_batch(batch)
            .map_err(|e| OperatorError::cache("batch_store", e.to_string()))?;
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_entries += operators_count as u64;
        stats.total_size_bytes += total_size;
        
        info!("Stored {} operators in cache batch", operators_count);
        Ok(())
    }
    
    /// Get all operators matching a filter
    pub async fn get_operators_filtered(&self, filter: &OperatorFilter) -> OperatorResult<Vec<Operator>> {
        let mut operators = Vec::new();
        
        for result in self.operators.iter() {
            let (key, data) = result.map_err(|e| OperatorError::cache("iterate", e.to_string()))?;
            
            let (entry, _): (CacheEntry<Operator>, _) = bincode::serde::decode_from_slice(&data, bincode::config::standard())
                .map_err(|e| OperatorError::serialization("decode", e.to_string()))?;
            
            // Skip expired entries
            if entry.is_expired() {
                self.operators.remove(&key)
                    .map_err(|e| OperatorError::cache("remove_expired", e.to_string()))?;
                continue;
            }
            
            // Apply filter
            if filter.matches(&entry.data) {
                operators.push(entry.data);
            }
        }
        
        debug!("Retrieved {} operators matching filter", operators.len());
        Ok(operators)
    }
    
    /// Store scan result
    pub async fn store_scan_result(&self, result: ScanResult) -> OperatorResult<()> {
        let key = result.completed_at.timestamp().to_be_bytes();
        let entry = CacheEntry::new(result.clone(), Some(self.config.default_ttl_seconds));
        
        let data = bincode::serde::encode_to_vec(&entry, bincode::config::standard())
            .map_err(|e| OperatorError::serialization("encode", e.to_string()))?;
        
        self.scan_results.insert(key, data)
            .map_err(|e| OperatorError::cache("store_scan_result", e.to_string()))?;
        
        debug!("Stored scan result: {} operators", result.operators.len());
        Ok(())
    }
    
    /// Get the latest scan result
    pub async fn get_latest_scan_result(&self) -> OperatorResult<Option<ScanResult>> {
        let mut latest_entry: Option<CacheEntry<ScanResult>> = None;
        let mut latest_key: Option<sled::IVec> = None;
        
        for result in self.scan_results.iter() {
            let (key, data) = result.map_err(|e| OperatorError::cache("iterate", e.to_string()))?;
            
            let (entry, _): (CacheEntry<ScanResult>, _) = bincode::serde::decode_from_slice(&data, bincode::config::standard())
                .map_err(|e| OperatorError::serialization("decode", e.to_string()))?;
            
            // Skip expired entries
            if entry.is_expired() {
                self.scan_results.remove(&key)
                    .map_err(|e| OperatorError::cache("remove_expired", e.to_string()))?;
                continue;
            }
            
            // Check if this is the latest
            match &latest_entry {
                None => {
                    latest_entry = Some(entry);
                    latest_key = Some(key);
                },
                Some(current_latest) => {
                    if entry.data.completed_at > current_latest.data.completed_at {
                        latest_entry = Some(entry);
                        latest_key = Some(key);
                    }
                }
            }
        }
        
        if let Some(mut entry) = latest_entry {
            // Update access time
            entry.touch();
            let updated_data = bincode::serde::encode_to_vec(&entry, bincode::config::standard())
                .map_err(|e| OperatorError::serialization("encode", e.to_string()))?;
            
            if let Some(key) = latest_key {
                self.scan_results.insert(key, updated_data)
                    .map_err(|e| OperatorError::cache("update_access", e.to_string()))?;
            }
            
            debug!("Retrieved latest scan result");
            Ok(Some(entry.data))
        } else {
            debug!("No scan results found in cache");
            Ok(None)
        }
    }
    
    /// Remove an operator from the cache
    pub async fn remove_operator(&self, name: &str) -> OperatorResult<bool> {
        let key = name.as_bytes();
        
        match self.operators.remove(key)
            .map_err(|e| OperatorError::cache_with_key("remove", e.to_string(), name.to_string()))?
        {
            Some(_) => {
                debug!("Removed operator from cache: {}", name);
                Ok(true)
            },
            None => {
                debug!("Operator not found in cache for removal: {}", name);
                Ok(false)
            }
        }
    }
    
    /// Clear all cached data
    pub async fn clear_all(&self) -> OperatorResult<()> {
        self.operators.clear()
            .map_err(|e| OperatorError::cache("clear_operators", e.to_string()))?;
        
        self.summaries.clear()
            .map_err(|e| OperatorError::cache("clear_summaries", e.to_string()))?;
        
        self.scan_results.clear()
            .map_err(|e| OperatorError::cache("clear_scan_results", e.to_string()))?;
        
        // Reset stats
        let mut stats = self.stats.write().await;
        *stats = CacheStats::new();
        
        info!("Cleared all cache data");
        Ok(())
    }
    
    /// Perform cache cleanup (remove expired entries)
    pub async fn cleanup(&self) -> OperatorResult<()> {
        let mut removed_count = 0u64;
        let mut reclaimed_bytes = 0u64;
        
        // Cleanup operators
        let mut to_remove = Vec::new();
        for result in self.operators.iter() {
            let (key, data) = result.map_err(|e| OperatorError::cache("iterate", e.to_string()))?;
            
            let (entry, _): (CacheEntry<Operator>, _) = bincode::serde::decode_from_slice(&data, bincode::config::standard())
                .map_err(|e| OperatorError::serialization("decode", e.to_string()))?;
            
            if entry.is_expired() {
                to_remove.push((key, entry.size_bytes));
            }
        }
        
        for (key, size) in to_remove {
            self.operators.remove(&key)
                .map_err(|e| OperatorError::cache("remove_expired", e.to_string()))?;
            removed_count += 1;
            reclaimed_bytes += size;
        }
        
        // Cleanup scan results
        let mut to_remove = Vec::new();
        for result in self.scan_results.iter() {
            let (key, data) = result.map_err(|e| OperatorError::cache("iterate", e.to_string()))?;
            
            let (entry, _): (CacheEntry<ScanResult>, _) = bincode::serde::decode_from_slice(&data, bincode::config::standard())
                .map_err(|e| OperatorError::serialization("decode", e.to_string()))?;
            
            if entry.is_expired() {
                to_remove.push((key, entry.size_bytes));
            }
        }
        
        for (key, size) in to_remove {
            self.scan_results.remove(&key)
                .map_err(|e| OperatorError::cache("remove_expired", e.to_string()))?;
            removed_count += 1;
            reclaimed_bytes += size;
        }
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_entries = stats.total_entries.saturating_sub(removed_count);
        stats.total_size_bytes = stats.total_size_bytes.saturating_sub(reclaimed_bytes);
        stats.expired_entries += removed_count;
        stats.last_cleanup = Some(Utc::now());
        
        info!("Cache cleanup completed: removed {} entries, reclaimed {} bytes", removed_count, reclaimed_bytes);
        Ok(())
    }
    
    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }
    
    /// Update cache statistics
    async fn update_stats(&self) -> OperatorResult<()> {
        let mut total_entries = 0u64;
        let mut total_size = 0u64;
        
        // Count operators
        for result in self.operators.iter() {
            let (_, data) = result.map_err(|e| OperatorError::cache("iterate", e.to_string()))?;
            
            let (entry, _): (CacheEntry<Operator>, _) = bincode::serde::decode_from_slice(&data, bincode::config::standard())
                .map_err(|e| OperatorError::serialization("decode", e.to_string()))?;
            
            if !entry.is_expired() {
                total_entries += 1;
                total_size += entry.size_bytes;
            }
        }
        
        // Count scan results
        for result in self.scan_results.iter() {
            let (_, data) = result.map_err(|e| OperatorError::cache("iterate", e.to_string()))?;
            
            let (entry, _): (CacheEntry<ScanResult>, _) = bincode::serde::decode_from_slice(&data, bincode::config::standard())
                .map_err(|e| OperatorError::serialization("decode", e.to_string()))?;
            
            if !entry.is_expired() {
                total_entries += 1;
                total_size += entry.size_bytes;
            }
        }
        
        let mut stats = self.stats.write().await;
        stats.total_entries = total_entries;
        stats.total_size_bytes = total_size;
        stats.calculate_hit_ratio();
        
        Ok(())
    }
    
    /// Get all cached operator names
    pub async fn get_cached_operator_names(&self) -> OperatorResult<Vec<String>> {
        let mut names = Vec::new();
        
        for result in self.operators.iter() {
            let (key, data) = result.map_err(|e| OperatorError::cache("iterate", e.to_string()))?;
            
            let (entry, _): (CacheEntry<Operator>, _) = bincode::serde::decode_from_slice(&data, bincode::config::standard())
                .map_err(|e| OperatorError::serialization("decode", e.to_string()))?;
            
            // Skip expired entries
            if entry.is_expired() {
                continue;
            }
            
            let name = String::from_utf8(key.to_vec())
                .map_err(|e| OperatorError::serialization("utf8", e.to_string()))?;
            
            names.push(name);
        }
        
        names.sort();
        Ok(names)
    }
    
    /// Flush all pending writes to disk
    pub async fn flush(&self) -> OperatorResult<()> {
        self.db.flush()
            .map_err(|e| OperatorError::cache("flush", e.to_string()))?;
        
        debug!("Cache flushed to disk");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::operator_manager::types::{Operator, OperatorFilter};
    
    async fn create_test_cache() -> (OperatorCache, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = CacheConfig {
            db_path: temp_dir.path().join("test_cache").to_string_lossy().to_string(),
            ..CacheConfig::default()
        };
        
        let cache = OperatorCache::new(config).await.unwrap();
        (cache, temp_dir)
    }
    
    #[tokio::test]
    async fn test_cache_creation() {
        let (cache, _temp_dir) = create_test_cache().await;
        let stats = cache.get_stats().await;
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.hit_count, 0);
        assert_eq!(stats.miss_count, 0);
    }
    
    #[tokio::test]
    async fn test_store_and_get_operator() {
        let (cache, _temp_dir) = create_test_cache().await;
        
        let operator = Operator::new("Amiya".to_string(), "Caster".to_string(), 5);
        cache.store_operator(operator.clone()).await.unwrap();
        
        let retrieved = cache.get_operator("Amiya").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Amiya");
        
        let stats = cache.get_stats().await;
        assert_eq!(stats.hit_count, 1);
        assert_eq!(stats.miss_count, 0);
    }
    
    #[tokio::test]
    async fn test_cache_miss() {
        let (cache, _temp_dir) = create_test_cache().await;
        
        let result = cache.get_operator("NonExistent").await.unwrap();
        assert!(result.is_none());
        
        let stats = cache.get_stats().await;
        assert_eq!(stats.hit_count, 0);
        assert_eq!(stats.miss_count, 1);
    }
    
    #[tokio::test]
    async fn test_batch_store() {
        let (cache, _temp_dir) = create_test_cache().await;
        
        let operators = vec![
            Operator::new("Amiya".to_string(), "Caster".to_string(), 5),
            Operator::new("SilverAsh".to_string(), "Guard".to_string(), 6),
            Operator::new("Exusiai".to_string(), "Sniper".to_string(), 6),
        ];
        
        cache.store_operators_batch(operators).await.unwrap();
        
        let amiya = cache.get_operator("Amiya").await.unwrap();
        assert!(amiya.is_some());
        
        let silverash = cache.get_operator("SilverAsh").await.unwrap();
        assert!(silverash.is_some());
        
        let stats = cache.get_stats().await;
        assert_eq!(stats.total_entries, 3);
    }
    
    #[tokio::test]
    async fn test_filtered_retrieval() {
        let (cache, _temp_dir) = create_test_cache().await;
        
        let operators = vec![
            Operator::new("Amiya".to_string(), "Caster".to_string(), 5),
            Operator::new("SilverAsh".to_string(), "Guard".to_string(), 6),
            Operator::new("Melantha".to_string(), "Guard".to_string(), 3),
        ];
        
        cache.store_operators_batch(operators).await.unwrap();
        
        let filter = OperatorFilter::new().with_profession("Guard".to_string());
        let guards = cache.get_operators_filtered(&filter).await.unwrap();
        assert_eq!(guards.len(), 2);
        
        let filter = OperatorFilter::new().with_min_rarity(6);
        let high_rarity = cache.get_operators_filtered(&filter).await.unwrap();
        assert_eq!(high_rarity.len(), 1);
        assert_eq!(high_rarity[0].name, "SilverAsh");
    }
    
    #[tokio::test]
    async fn test_remove_operator() {
        let (cache, _temp_dir) = create_test_cache().await;
        
        let operator = Operator::new("Amiya".to_string(), "Caster".to_string(), 5);
        cache.store_operator(operator).await.unwrap();
        
        let removed = cache.remove_operator("Amiya").await.unwrap();
        assert!(removed);
        
        let result = cache.get_operator("Amiya").await.unwrap();
        assert!(result.is_none());
        
        let removed_again = cache.remove_operator("Amiya").await.unwrap();
        assert!(!removed_again);
    }
    
    #[tokio::test]
    async fn test_clear_all() {
        let (cache, _temp_dir) = create_test_cache().await;
        
        let operators = vec![
            Operator::new("Amiya".to_string(), "Caster".to_string(), 5),
            Operator::new("SilverAsh".to_string(), "Guard".to_string(), 6),
        ];
        
        cache.store_operators_batch(operators).await.unwrap();
        
        let stats_before = cache.get_stats().await;
        assert_eq!(stats_before.total_entries, 2);
        
        cache.clear_all().await.unwrap();
        
        let stats_after = cache.get_stats().await;
        assert_eq!(stats_after.total_entries, 0);
        
        let result = cache.get_operator("Amiya").await.unwrap();
        assert!(result.is_none());
    }
    
    #[tokio::test]
    async fn test_scan_result_storage() {
        let (cache, _temp_dir) = create_test_cache().await;
        
        let scan_result = ScanResult::new(vec![
            Operator::new("Amiya".to_string(), "Caster".to_string(), 5),
        ], 1500);
        
        cache.store_scan_result(scan_result.clone()).await.unwrap();
        
        let retrieved = cache.get_latest_scan_result().await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().operators.len(), 1);
    }
    
    #[tokio::test]
    async fn test_get_cached_operator_names() {
        let (cache, _temp_dir) = create_test_cache().await;
        
        let operators = vec![
            Operator::new("Amiya".to_string(), "Caster".to_string(), 5),
            Operator::new("SilverAsh".to_string(), "Guard".to_string(), 6),
            Operator::new("Exusiai".to_string(), "Sniper".to_string(), 6),
        ];
        
        cache.store_operators_batch(operators).await.unwrap();
        
        let names = cache.get_cached_operator_names().await.unwrap();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"Amiya".to_string()));
        assert!(names.contains(&"SilverAsh".to_string()));
        assert!(names.contains(&"Exusiai".to_string()));
    }
    
    #[tokio::test]
    async fn test_flush() {
        let (cache, _temp_dir) = create_test_cache().await;
        
        let operator = Operator::new("Amiya".to_string(), "Caster".to_string(), 5);
        cache.store_operator(operator).await.unwrap();
        
        // This should not panic
        cache.flush().await.unwrap();
    }
}