//! 缓存管理模块
//! 
//! 提供基于sled的高性能缓存实现，支持TTL机制和自动过期清理。
//! 用于缓存作业数据和匹配结果，提高系统响应速度。

use super::types::{CopilotData, MatchResult, CopilotError, CopilotResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::path::Path;
use std::sync::Arc;
use tokio::time::{interval, Duration as TokioDuration};

/// 缓存条目结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    /// 缓存的数据
    pub data: T,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 过期时间
    pub expires_at: DateTime<Utc>,
    /// 访问次数
    pub access_count: u64,
    /// 最后访问时间
    pub last_accessed: DateTime<Utc>,
}

impl<T> CacheEntry<T> {
    /// 创建新的缓存条目
    pub fn new(data: T, ttl: Duration) -> Self {
        let now = Utc::now();
        Self {
            data,
            created_at: now,
            expires_at: now + ttl,
            access_count: 0,
            last_accessed: now,
        }
    }

    /// 检查是否已过期
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// 更新访问统计
    pub fn touch(&mut self) {
        self.access_count += 1;
        self.last_accessed = Utc::now();
    }

    /// 刷新过期时间
    pub fn refresh(&mut self, ttl: Duration) {
        self.expires_at = Utc::now() + ttl;
    }
}

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// 数据库路径
    pub db_path: String,
    /// 默认TTL（秒）
    pub default_ttl: i64,
    /// 作业数据TTL（秒）
    pub copilot_data_ttl: i64,
    /// 匹配结果TTL（秒）
    pub match_result_ttl: i64,
    /// 最大缓存大小（条目数）
    pub max_entries: u64,
    /// 清理间隔（秒）
    pub cleanup_interval: u64,
    /// 是否启用压缩
    pub enable_compression: bool,
    /// 统计采样率（0.0 - 1.0）
    pub stats_sample_rate: f32,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            db_path: "./cache/copilot_matcher".to_string(),
            default_ttl: 3600,        // 1小时
            copilot_data_ttl: 7200,   // 2小时
            match_result_ttl: 1800,   // 30分钟
            max_entries: 10000,
            cleanup_interval: 300,     // 5分钟
            enable_compression: true,
            stats_sample_rate: 0.1,   // 10%采样率
        }
    }
}

impl CacheConfig {
    /// 创建新的缓存配置
    pub fn new(db_path: String) -> Self {
        Self {
            db_path,
            ..Default::default()
        }
    }

    /// 设置TTL
    pub fn with_ttl(mut self, default: i64, copilot: i64, match_result: i64) -> Self {
        self.default_ttl = default;
        self.copilot_data_ttl = copilot;
        self.match_result_ttl = match_result;
        self
    }

    /// 设置最大条目数
    pub fn with_max_entries(mut self, max_entries: u64) -> Self {
        self.max_entries = max_entries;
        self
    }

    /// 验证配置
    pub fn validate(&self) -> CopilotResult<()> {
        if self.db_path.is_empty() {
            return Err(CopilotError::ConfigError("Database path cannot be empty".to_string()));
        }

        if self.default_ttl <= 0 {
            return Err(CopilotError::ConfigError("TTL must be positive".to_string()));
        }

        if self.max_entries == 0 {
            return Err(CopilotError::ConfigError("Max entries must be positive".to_string()));
        }

        Ok(())
    }
}

/// 缓存统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// 总访问次数
    pub total_requests: u64,
    /// 缓存命中次数
    pub cache_hits: u64,
    /// 缓存未命中次数
    pub cache_misses: u64,
    /// 当前条目数
    pub current_entries: u64,
    /// 过期清理次数
    pub cleanup_runs: u64,
    /// 最后清理时间
    pub last_cleanup: Option<DateTime<Utc>>,
    /// 启动时间
    pub started_at: DateTime<Utc>,
}

impl CacheStats {
    /// 创建新的统计信息
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            cache_hits: 0,
            cache_misses: 0,
            current_entries: 0,
            cleanup_runs: 0,
            last_cleanup: None,
            started_at: Utc::now(),
        }
    }

    /// 计算命中率
    pub fn hit_rate(&self) -> f32 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.cache_hits as f32 / self.total_requests as f32
        }
    }

    /// 记录缓存命中
    pub fn record_hit(&mut self) {
        self.total_requests += 1;
        self.cache_hits += 1;
    }

    /// 记录缓存未命中
    pub fn record_miss(&mut self) {
        self.total_requests += 1;
        self.cache_misses += 1;
    }

    /// 记录清理操作
    pub fn record_cleanup(&mut self) {
        self.cleanup_runs += 1;
        self.last_cleanup = Some(Utc::now());
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self::new()
    }
}

/// 缓存管理器特征
#[async_trait]
pub trait CacheManagerTrait: Send + Sync {
    /// 存储作业数据
    async fn store_copilot_data(&self, key: &str, data: &CopilotData) -> CopilotResult<()>;

    /// 获取作业数据
    async fn get_copilot_data(&self, key: &str) -> CopilotResult<Option<CopilotData>>;

    /// 存储匹配结果
    async fn store_match_result(&self, key: &str, result: &MatchResult) -> CopilotResult<()>;

    /// 获取匹配结果
    async fn get_match_result(&self, key: &str) -> CopilotResult<Option<MatchResult>>;

    /// 存储匹配结果列表
    async fn store_match_results(&self, key: &str, results: &Vec<MatchResult>) -> CopilotResult<()>;

    /// 获取匹配结果列表
    async fn get_match_results(&self, key: &str) -> CopilotResult<Option<Vec<MatchResult>>>;

    /// 删除缓存条目
    async fn remove(&self, key: &str) -> CopilotResult<bool>;

    /// 清理过期条目
    async fn cleanup_expired(&self) -> CopilotResult<u64>;

    /// 清空所有缓存
    async fn clear_all(&self) -> CopilotResult<()>;

    /// 获取缓存统计
    async fn get_stats(&self) -> CopilotResult<CacheStats>;

    /// 检查健康状态
    async fn health_check(&self) -> CopilotResult<bool>;
}

/// Sled缓存管理器实现
#[derive(Debug)]
pub struct CacheManager {
    config: CacheConfig,
    db: Arc<Db>,
    stats: Arc<tokio::sync::Mutex<CacheStats>>,
}

impl CacheManager {
    /// 创建新的缓存管理器
    pub async fn new(config: CacheConfig) -> CopilotResult<Self> {
        config.validate()?;

        // 确保数据库目录存在
        if let Some(parent) = Path::new(&config.db_path).parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| CopilotError::CacheError(format!("Failed to create cache directory: {}", e)))?;
        }

        // 打开数据库
        let db = sled::open(&config.db_path)
            .map_err(|e| CopilotError::CacheError(format!("Failed to open cache database: {}", e)))?;

        let cache_manager = Self {
            config: config.clone(),
            db: Arc::new(db),
            stats: Arc::new(tokio::sync::Mutex::new(CacheStats::new())),
        };

        // 启动清理任务
        cache_manager.start_cleanup_task().await;

        Ok(cache_manager)
    }

    /// 启动清理任务
    async fn start_cleanup_task(&self) {
        let db = self.db.clone();
        let stats = self.stats.clone();
        let cleanup_interval = self.config.cleanup_interval;

        tokio::spawn(async move {
            let mut interval = interval(TokioDuration::from_secs(cleanup_interval));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = Self::cleanup_expired_internal(&db, &stats).await {
                    tracing::warn!("Cache cleanup failed: {}", e);
                }
            }
        });
    }

    /// 内部清理过期条目
    async fn cleanup_expired_internal(
        db: &Arc<Db>,
        stats: &Arc<tokio::sync::Mutex<CacheStats>>,
    ) -> CopilotResult<u64> {
        let mut removed_count = 0;
        let now = Utc::now();

        for result in db.iter() {
            let (key, value) = result?;
            
            // 尝试解析为 CacheEntry<serde_json::Value>
            if let Ok(entry) = serde_json::from_slice::<CacheEntry<serde_json::Value>>(&value) {
                if entry.expires_at < now {
                    db.remove(&key)?;
                    removed_count += 1;
                }
            }
        }

        // 更新统计信息
        let mut stats_guard = stats.lock().await;
        stats_guard.record_cleanup();
        stats_guard.current_entries = stats_guard.current_entries.saturating_sub(removed_count);

        Ok(removed_count)
    }

    /// 存储数据到缓存
    async fn store_data<T: Serialize>(
        &self,
        key: &str,
        data: &T,
        ttl: Duration,
    ) -> CopilotResult<()> {
        let entry = CacheEntry::new(data, ttl);
        let serialized = serde_json::to_vec(&entry)
            .map_err(|e| CopilotError::SerializationError(format!("Failed to serialize cache entry: {}", e)))?;

        self.db.insert(key, serialized)?;

        // 更新统计信息
        let mut stats = self.stats.lock().await;
        stats.current_entries += 1;

        Ok(())
    }

    /// 从缓存获取数据
    async fn get_data<T: for<'de> Deserialize<'de> + Serialize>(&self, key: &str) -> CopilotResult<Option<T>> {
        let mut stats = self.stats.lock().await;

        match self.db.get(key)? {
            Some(serialized) => {
                let mut entry: CacheEntry<T> = serde_json::from_slice(&serialized)
                    .map_err(|e| CopilotError::SerializationError(format!("Failed to deserialize cache entry: {}", e)))?;

                if entry.is_expired() {
                    // 过期条目，删除并返回None
                    drop(stats); // 释放锁
                    self.db.remove(key)?;
                    let mut stats = self.stats.lock().await;
                    stats.record_miss();
                    stats.current_entries = stats.current_entries.saturating_sub(1);
                    Ok(None)
                } else {
                    // 更新访问统计
                    entry.touch();
                    
                    // 重新序列化更新后的条目
                    let updated_serialized = serde_json::to_vec(&entry)
                        .map_err(|e| CopilotError::SerializationError(format!("Failed to serialize updated entry: {}", e)))?;
                    
                    drop(stats); // 释放锁以避免死锁
                    self.db.insert(key, updated_serialized)?;
                    
                    let mut stats = self.stats.lock().await;
                    stats.record_hit();
                    Ok(Some(entry.data))
                }
            }
            None => {
                stats.record_miss();
                Ok(None)
            }
        }
    }

    /// 生成作业数据缓存键
    fn copilot_data_key(&self, id: &str) -> String {
        format!("copilot_data:{}", id)
    }

    /// 生成匹配结果缓存键
    fn match_result_key(&self, query_hash: &str) -> String {
        format!("match_result:{}", query_hash)
    }
}

#[async_trait]
impl CacheManagerTrait for CacheManager {
    async fn store_copilot_data(&self, key: &str, data: &CopilotData) -> CopilotResult<()> {
        let cache_key = self.copilot_data_key(key);
        let ttl = Duration::seconds(self.config.copilot_data_ttl);
        self.store_data(&cache_key, data, ttl).await
    }

    async fn get_copilot_data(&self, key: &str) -> CopilotResult<Option<CopilotData>> {
        let cache_key = self.copilot_data_key(key);
        self.get_data(&cache_key).await
    }

    async fn store_match_result(&self, key: &str, result: &MatchResult) -> CopilotResult<()> {
        let cache_key = self.match_result_key(key);
        let ttl = Duration::seconds(self.config.match_result_ttl);
        self.store_data(&cache_key, result, ttl).await
    }

    async fn get_match_result(&self, key: &str) -> CopilotResult<Option<MatchResult>> {
        let cache_key = self.match_result_key(key);
        self.get_data(&cache_key).await
    }

    async fn store_match_results(&self, key: &str, results: &Vec<MatchResult>) -> CopilotResult<()> {
        let cache_key = format!("match_results:{}", key);
        let ttl = Duration::seconds(self.config.match_result_ttl);
        self.store_data(&cache_key, results, ttl).await
    }

    async fn get_match_results(&self, key: &str) -> CopilotResult<Option<Vec<MatchResult>>> {
        let cache_key = format!("match_results:{}", key);
        self.get_data(&cache_key).await
    }

    async fn remove(&self, key: &str) -> CopilotResult<bool> {
        let removed = self.db.remove(key)?.is_some();
        
        if removed {
            let mut stats = self.stats.lock().await;
            stats.current_entries = stats.current_entries.saturating_sub(1);
        }
        
        Ok(removed)
    }

    async fn cleanup_expired(&self) -> CopilotResult<u64> {
        Self::cleanup_expired_internal(&self.db, &self.stats).await
    }

    async fn clear_all(&self) -> CopilotResult<()> {
        self.db.clear()?;
        
        let mut stats = self.stats.lock().await;
        stats.current_entries = 0;
        
        Ok(())
    }

    async fn get_stats(&self) -> CopilotResult<CacheStats> {
        let stats = self.stats.lock().await;
        Ok(stats.clone())
    }

    async fn health_check(&self) -> CopilotResult<bool> {
        // 简单的健康检查：尝试写入和读取一个测试键
        let test_key = "health_check_test";
        let test_data = "test_value";
        
        match self.db.insert(test_key, test_data) {
            Ok(_) => {
                match self.db.get(test_key) {
                    Ok(Some(_)) => {
                        let _ = self.db.remove(test_key);
                        Ok(true)
                    }
                    _ => Ok(false),
                }
            }
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::copilot_matcher::types::StageOperator;
    use tempfile::TempDir;

    async fn create_test_cache_manager() -> (CacheManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = CacheConfig::new(temp_dir.path().join("test_cache").to_string_lossy().to_string())
            .with_ttl(60, 120, 30)
            .with_max_entries(100);
        
        let manager = CacheManager::new(config).await.unwrap();
        (manager, temp_dir)
    }

    fn create_test_copilot_data() -> CopilotData {
        CopilotData::new(
            "test_001".to_string(),
            "测试作业".to_string(),
            "1-7".to_string(),
            vec![StageOperator::new("夏".to_string(), 1)],
        )
    }

    fn create_test_match_result() -> MatchResult {
        use crate::copilot_matcher::types::{MatchScore, MatchStage};
        
        MatchResult::new(
            create_test_copilot_data(),
            MatchScore::new(),
            MatchStage::Simple,
        )
    }

    #[test]
    fn test_cache_entry_creation() {
        let data = "test_data";
        let ttl = Duration::hours(1);
        let entry = CacheEntry::new(data, ttl);

        assert_eq!(entry.data, "test_data");
        assert!(!entry.is_expired());
        assert_eq!(entry.access_count, 0);
    }

    #[test]
    fn test_cache_entry_expiration() {
        let data = "test_data";
        let ttl = Duration::seconds(-1); // 已过期
        let entry = CacheEntry::new(data, ttl);

        assert!(entry.is_expired());
    }

    #[test]
    fn test_cache_entry_touch() {
        let data = "test_data";
        let ttl = Duration::hours(1);
        let mut entry = CacheEntry::new(data, ttl);

        let initial_access_time = entry.last_accessed;
        entry.touch();

        assert_eq!(entry.access_count, 1);
        assert!(entry.last_accessed >= initial_access_time);
    }

    #[test]
    fn test_cache_config_validation() {
        let config = CacheConfig::default();
        assert!(config.validate().is_ok());

        let invalid_config = CacheConfig {
            db_path: "".to_string(),
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());

        let invalid_ttl = CacheConfig {
            default_ttl: -1,
            ..Default::default()
        };
        assert!(invalid_ttl.validate().is_err());

        let invalid_max_entries = CacheConfig {
            max_entries: 0,
            ..Default::default()
        };
        assert!(invalid_max_entries.validate().is_err());
    }

    #[test]
    fn test_cache_stats() {
        let mut stats = CacheStats::new();
        
        assert_eq!(stats.hit_rate(), 0.0);
        
        stats.record_hit();
        stats.record_hit();
        stats.record_miss();
        
        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.cache_hits, 2);
        assert_eq!(stats.cache_misses, 1);
        assert!((stats.hit_rate() - 2.0/3.0).abs() < f32::EPSILON);
        
        stats.record_cleanup();
        assert_eq!(stats.cleanup_runs, 1);
        assert!(stats.last_cleanup.is_some());
    }

    #[tokio::test]
    async fn test_cache_manager_creation() {
        let (manager, _temp_dir) = create_test_cache_manager().await;
        
        assert!(manager.health_check().await.unwrap());
    }

    #[tokio::test]
    async fn test_copilot_data_caching() {
        let (manager, _temp_dir) = create_test_cache_manager().await;
        let test_data = create_test_copilot_data();
        
        // 存储数据
        assert!(manager.store_copilot_data("test_001", &test_data).await.is_ok());
        
        // 获取数据
        let retrieved = manager.get_copilot_data("test_001").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "test_001");
        
        // 获取不存在的数据
        let not_found = manager.get_copilot_data("not_exist").await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_match_result_caching() {
        let (manager, _temp_dir) = create_test_cache_manager().await;
        let test_result = create_test_match_result();
        
        // 存储匹配结果
        assert!(manager.store_match_result("query_hash", &test_result).await.is_ok());
        
        // 获取匹配结果
        let retrieved = manager.get_match_result("query_hash").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().copilot.id, "test_001");
    }

    #[tokio::test]
    async fn test_cache_removal() {
        let (manager, _temp_dir) = create_test_cache_manager().await;
        let test_data = create_test_copilot_data();
        
        // 存储数据
        manager.store_copilot_data("test_001", &test_data).await.unwrap();
        
        // 验证数据存在
        assert!(manager.get_copilot_data("test_001").await.unwrap().is_some());
        
        // 删除数据
        let removed = manager.remove(&manager.copilot_data_key("test_001")).await.unwrap();
        assert!(removed);
        
        // 验证数据已删除
        assert!(manager.get_copilot_data("test_001").await.unwrap().is_none());
        
        // 删除不存在的数据
        let not_removed = manager.remove("not_exist").await.unwrap();
        assert!(!not_removed);
    }

    #[tokio::test]
    async fn test_cache_clear_all() {
        let (manager, _temp_dir) = create_test_cache_manager().await;
        let test_data = create_test_copilot_data();
        
        // 存储多个数据
        manager.store_copilot_data("test_001", &test_data).await.unwrap();
        manager.store_copilot_data("test_002", &test_data).await.unwrap();
        
        // 清空所有缓存
        manager.clear_all().await.unwrap();
        
        // 验证所有数据都已删除
        assert!(manager.get_copilot_data("test_001").await.unwrap().is_none());
        assert!(manager.get_copilot_data("test_002").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_cache_stats_tracking() {
        let (manager, _temp_dir) = create_test_cache_manager().await;
        let test_data = create_test_copilot_data();
        
        // 初始统计
        let initial_stats = manager.get_stats().await.unwrap();
        assert_eq!(initial_stats.total_requests, 0);
        
        // 存储数据
        manager.store_copilot_data("test_001", &test_data).await.unwrap();
        
        // 缓存命中
        manager.get_copilot_data("test_001").await.unwrap();
        
        // 缓存未命中
        manager.get_copilot_data("not_exist").await.unwrap();
        
        let final_stats = manager.get_stats().await.unwrap();
        assert_eq!(final_stats.total_requests, 2);
        assert_eq!(final_stats.cache_hits, 1);
        assert_eq!(final_stats.cache_misses, 1);
        assert_eq!(final_stats.hit_rate(), 0.5);
    }

    #[tokio::test]
    async fn test_cache_key_generation() {
        let (manager, _temp_dir) = create_test_cache_manager().await;
        
        let copilot_key = manager.copilot_data_key("test_001");
        assert_eq!(copilot_key, "copilot_data:test_001");
        
        let match_key = manager.match_result_key("query_hash");
        assert_eq!(match_key, "match_result:query_hash");
    }

    #[tokio::test]
    async fn test_cache_health_check() {
        let (manager, _temp_dir) = create_test_cache_manager().await;
        
        let health = manager.health_check().await.unwrap();
        assert!(health);
    }
}