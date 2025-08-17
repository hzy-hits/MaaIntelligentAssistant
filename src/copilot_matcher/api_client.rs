//! API客户端模块
//! 
//! 负责与外部作业数据源进行通信，获取作业信息和相关数据。

use super::types::{CopilotData, CopilotError, CopilotResult};
use async_trait::async_trait;
use reqwest::{Client, ClientBuilder, Response};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// API客户端配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// 基础URL
    pub base_url: String,
    /// API密钥
    pub api_key: Option<String>,
    /// 请求超时时间（秒）
    pub timeout: u64,
    /// 最大重试次数
    pub max_retries: u32,
    /// 重试间隔（毫秒）
    pub retry_interval: u64,
    /// 用户代理
    pub user_agent: String,
    /// 是否启用压缩
    pub enable_compression: bool,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.copilot.maa.plus".to_string(),
            api_key: None,
            timeout: 30,
            max_retries: 3,
            retry_interval: 1000,
            user_agent: "MAA-CopilotMatcher/1.0".to_string(),
            enable_compression: true,
        }
    }
}

impl ApiConfig {
    /// 创建新的API配置
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            ..Default::default()
        }
    }

    /// 设置API密钥
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// 设置超时时间
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }

    /// 设置最大重试次数
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// 验证配置
    pub fn validate(&self) -> CopilotResult<()> {
        if self.base_url.is_empty() {
            return Err(CopilotError::ConfigError("Base URL cannot be empty".to_string()));
        }

        if self.timeout == 0 {
            return Err(CopilotError::ConfigError("Timeout must be greater than 0".to_string()));
        }

        Ok(())
    }
}

/// API响应数据结构
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// 是否成功
    pub success: bool,
    /// 响应数据
    pub data: Option<T>,
    /// 错误信息
    pub error: Option<String>,
    /// 响应消息
    pub message: Option<String>,
    /// 总数（用于分页）
    pub total: Option<u64>,
    /// 当前页（用于分页）
    pub page: Option<u32>,
    /// 每页大小（用于分页）
    pub page_size: Option<u32>,
}

/// 分页参数
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationParams {
    /// 页码（从1开始）
    pub page: u32,
    /// 每页大小
    pub page_size: u32,
    /// 排序字段
    pub sort_by: Option<String>,
    /// 排序方向（asc/desc）
    pub sort_order: Option<String>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
            sort_by: Some("updated_at".to_string()),
            sort_order: Some("desc".to_string()),
        }
    }
}

/// 查询过滤器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryFilter {
    /// 关卡ID
    pub stage_id: Option<String>,
    /// 最低难度
    pub min_difficulty: Option<u32>,
    /// 最高难度
    pub max_difficulty: Option<u32>,
    /// 标签过滤
    pub tags: Option<Vec<String>>,
    /// 是否只返回推荐作业
    pub recommended_only: Option<bool>,
    /// 创建时间范围
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    /// 干员名称过滤
    pub operator_names: Option<Vec<String>>,
}

impl Default for QueryFilter {
    fn default() -> Self {
        Self {
            stage_id: None,
            min_difficulty: None,
            max_difficulty: None,
            tags: None,
            recommended_only: None,
            created_after: None,
            created_before: None,
            operator_names: None,
        }
    }
}

/// API客户端特征
#[async_trait]
pub trait ApiClientTrait: Send + Sync {
    /// 获取作业列表
    async fn get_copilots(
        &self,
        filter: Option<QueryFilter>,
        pagination: Option<PaginationParams>,
    ) -> CopilotResult<Vec<CopilotData>>;

    /// 根据ID获取作业
    async fn get_copilot_by_id(&self, id: &str) -> CopilotResult<CopilotData>;

    /// 根据关卡ID获取作业列表
    async fn get_copilots_by_stage(&self, stage_id: &str) -> CopilotResult<Vec<CopilotData>>;

    /// 搜索作业
    async fn search_copilots(&self, query: &str) -> CopilotResult<Vec<CopilotData>>;

    /// 获取推荐作业
    async fn get_recommended_copilots(&self, limit: Option<u32>) -> CopilotResult<Vec<CopilotData>>;

    /// 检查API健康状态
    async fn health_check(&self) -> CopilotResult<bool>;
}

/// HTTP API客户端实现
#[derive(Debug)]
pub struct ApiClient {
    config: ApiConfig,
    client: Client,
}

impl ApiClient {
    /// 创建新的API客户端
    pub fn new(config: ApiConfig) -> CopilotResult<Self> {
        config.validate()?;

        let mut builder = ClientBuilder::new()
            .timeout(Duration::from_secs(config.timeout))
            .user_agent(&config.user_agent);

        if !config.enable_compression {
            builder = builder.no_gzip();
        }

        let client = builder.build()
            .map_err(|e| CopilotError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// 发送GET请求
    async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
        params: Option<&HashMap<String, String>>,
    ) -> CopilotResult<ApiResponse<T>> {
        let url = format!("{}/{}", self.config.base_url.trim_end_matches('/'), endpoint);
        
        let mut request = self.client.get(&url);

        // 添加API密钥（如果有）
        if let Some(ref api_key) = self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        // 添加查询参数
        if let Some(params) = params {
            request = request.query(params);
        }

        // 重试机制
        let mut last_error = None;
        for attempt in 0..=self.config.max_retries {
            match request.try_clone() {
                Some(req) => {
                    match req.send().await {
                        Ok(response) => {
                            return self.handle_response(response).await;
                        }
                        Err(e) => {
                            last_error = Some(e);
                            if attempt < self.config.max_retries {
                                tokio::time::sleep(Duration::from_millis(self.config.retry_interval)).await;
                            }
                        }
                    }
                }
                None => {
                    return Err(CopilotError::InternalError("Failed to clone request".to_string()));
                }
            }
        }

        Err(CopilotError::NetworkError(
            last_error
                .map(|e| e.to_string())
                .unwrap_or_else(|| "Max retries exceeded".to_string())
        ))
    }

    /// 处理HTTP响应
    async fn handle_response<T: for<'de> Deserialize<'de>>(
        &self,
        response: Response,
    ) -> CopilotResult<ApiResponse<T>> {
        let status = response.status();
        
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| format!("HTTP {}", status));
            return Err(CopilotError::ApiError(error_text));
        }

        let text = response.text().await
            .map_err(|e| CopilotError::NetworkError(format!("Failed to read response: {}", e)))?;

        serde_json::from_str(&text)
            .map_err(|e| CopilotError::SerializationError(format!("Failed to parse response: {}", e)))
    }

    /// 构建查询参数
    fn build_query_params(
        filter: Option<&QueryFilter>,
        pagination: Option<&PaginationParams>,
    ) -> HashMap<String, String> {
        let mut params = HashMap::new();

        // 添加分页参数
        if let Some(page) = pagination {
            params.insert("page".to_string(), page.page.to_string());
            params.insert("page_size".to_string(), page.page_size.to_string());
            
            if let Some(ref sort_by) = page.sort_by {
                params.insert("sort_by".to_string(), sort_by.clone());
            }
            
            if let Some(ref sort_order) = page.sort_order {
                params.insert("sort_order".to_string(), sort_order.clone());
            }
        }

        // 添加过滤参数
        if let Some(filter) = filter {
            if let Some(ref stage_id) = filter.stage_id {
                params.insert("stage_id".to_string(), stage_id.clone());
            }
            
            if let Some(min_difficulty) = filter.min_difficulty {
                params.insert("min_difficulty".to_string(), min_difficulty.to_string());
            }
            
            if let Some(max_difficulty) = filter.max_difficulty {
                params.insert("max_difficulty".to_string(), max_difficulty.to_string());
            }
            
            if let Some(ref tags) = filter.tags {
                params.insert("tags".to_string(), tags.join(","));
            }
            
            if let Some(recommended_only) = filter.recommended_only {
                params.insert("recommended_only".to_string(), recommended_only.to_string());
            }
            
            if let Some(ref operator_names) = filter.operator_names {
                params.insert("operator_names".to_string(), operator_names.join(","));
            }
        }

        params
    }
}

#[async_trait]
impl ApiClientTrait for ApiClient {
    async fn get_copilots(
        &self,
        filter: Option<QueryFilter>,
        pagination: Option<PaginationParams>,
    ) -> CopilotResult<Vec<CopilotData>> {
        let params = Self::build_query_params(filter.as_ref(), pagination.as_ref());
        let response: ApiResponse<Vec<CopilotData>> = self.get("copilots", Some(&params)).await?;

        if response.success {
            Ok(response.data.unwrap_or_default())
        } else {
            Err(CopilotError::ApiError(
                response.error.unwrap_or_else(|| "Unknown API error".to_string())
            ))
        }
    }

    async fn get_copilot_by_id(&self, id: &str) -> CopilotResult<CopilotData> {
        let endpoint = format!("copilots/{}", id);
        let response: ApiResponse<CopilotData> = self.get(&endpoint, None).await?;

        if response.success {
            response.data.ok_or_else(|| CopilotError::CopilotNotFound(id.to_string()))
        } else {
            Err(CopilotError::ApiError(
                response.error.unwrap_or_else(|| "Unknown API error".to_string())
            ))
        }
    }

    async fn get_copilots_by_stage(&self, stage_id: &str) -> CopilotResult<Vec<CopilotData>> {
        let filter = QueryFilter {
            stage_id: Some(stage_id.to_string()),
            ..Default::default()
        };
        
        self.get_copilots(Some(filter), None).await
    }

    async fn search_copilots(&self, query: &str) -> CopilotResult<Vec<CopilotData>> {
        let mut params = HashMap::new();
        params.insert("q".to_string(), query.to_string());
        
        let response: ApiResponse<Vec<CopilotData>> = self.get("copilots/search", Some(&params)).await?;

        if response.success {
            Ok(response.data.unwrap_or_default())
        } else {
            Err(CopilotError::ApiError(
                response.error.unwrap_or_else(|| "Unknown API error".to_string())
            ))
        }
    }

    async fn get_recommended_copilots(&self, limit: Option<u32>) -> CopilotResult<Vec<CopilotData>> {
        let filter = QueryFilter {
            recommended_only: Some(true),
            ..Default::default()
        };
        
        let pagination = PaginationParams {
            page_size: limit.unwrap_or(10),
            ..Default::default()
        };

        self.get_copilots(Some(filter), Some(pagination)).await
    }

    async fn health_check(&self) -> CopilotResult<bool> {
        match self.get::<serde_json::Value>("health", None).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

/// 模拟API客户端（用于测试）
#[cfg(test)]
pub struct MockApiClient {
    pub mock_data: Vec<CopilotData>,
    pub should_fail: bool,
}

#[cfg(test)]
impl MockApiClient {
    pub fn new(mock_data: Vec<CopilotData>) -> Self {
        Self {
            mock_data,
            should_fail: false,
        }
    }

    pub fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

#[cfg(test)]
#[async_trait]
impl ApiClientTrait for MockApiClient {
    async fn get_copilots(
        &self,
        filter: Option<QueryFilter>,
        _pagination: Option<PaginationParams>,
    ) -> CopilotResult<Vec<CopilotData>> {
        if self.should_fail {
            return Err(CopilotError::ApiError("Mock API failure".to_string()));
        }

        let mut result = self.mock_data.clone();

        if let Some(filter) = filter {
            if let Some(stage_id) = filter.stage_id {
                result.retain(|c| c.stage_id == stage_id);
            }
            
            if let Some(min_difficulty) = filter.min_difficulty {
                result.retain(|c| c.difficulty >= min_difficulty);
            }
            
            if let Some(max_difficulty) = filter.max_difficulty {
                result.retain(|c| c.difficulty <= max_difficulty);
            }
            
            if let Some(recommended_only) = filter.recommended_only {
                if recommended_only {
                    result.retain(|c| c.recommended);
                }
            }
        }

        Ok(result)
    }

    async fn get_copilot_by_id(&self, id: &str) -> CopilotResult<CopilotData> {
        if self.should_fail {
            return Err(CopilotError::ApiError("Mock API failure".to_string()));
        }

        self.mock_data.iter()
            .find(|c| c.id == id)
            .cloned()
            .ok_or_else(|| CopilotError::CopilotNotFound(id.to_string()))
    }

    async fn get_copilots_by_stage(&self, stage_id: &str) -> CopilotResult<Vec<CopilotData>> {
        let filter = QueryFilter {
            stage_id: Some(stage_id.to_string()),
            ..Default::default()
        };
        self.get_copilots(Some(filter), None).await
    }

    async fn search_copilots(&self, query: &str) -> CopilotResult<Vec<CopilotData>> {
        if self.should_fail {
            return Err(CopilotError::ApiError("Mock API failure".to_string()));
        }

        let query_lower = query.to_lowercase();
        let result = self.mock_data.iter()
            .filter(|c| {
                c.name.to_lowercase().contains(&query_lower) ||
                c.stage_id.to_lowercase().contains(&query_lower) ||
                c.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect();

        Ok(result)
    }

    async fn get_recommended_copilots(&self, limit: Option<u32>) -> CopilotResult<Vec<CopilotData>> {
        if self.should_fail {
            return Err(CopilotError::ApiError("Mock API failure".to_string()));
        }

        let mut result: Vec<_> = self.mock_data.iter()
            .filter(|c| c.recommended)
            .cloned()
            .collect();

        if let Some(limit) = limit {
            result.truncate(limit as usize);
        }

        Ok(result)
    }

    async fn health_check(&self) -> CopilotResult<bool> {
        Ok(!self.should_fail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::copilot_matcher::types::StageOperator;

    fn create_mock_copilot(id: &str, stage_id: &str, recommended: bool) -> CopilotData {
        CopilotData {
            id: id.to_string(),
            name: format!("Test Copilot {}", id),
            stage_id: stage_id.to_string(),
            description: Some("Test description".to_string()),
            operators: vec![StageOperator::new("夏".to_string(), 1)],
            min_level: 50,
            avg_level: 60.0,
            elite_requirements: HashMap::new(),
            skill_requirements: HashMap::new(),
            mastery_requirements: HashMap::new(),
            difficulty: 5,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: vec!["test".to_string()],
            recommended,
        }
    }

    #[test]
    fn test_api_config_validation() {
        let config = ApiConfig::default();
        assert!(config.validate().is_ok());

        let invalid_config = ApiConfig {
            base_url: "".to_string(),
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());

        let invalid_timeout = ApiConfig {
            timeout: 0,
            ..Default::default()
        };
        assert!(invalid_timeout.validate().is_err());
    }

    #[test]
    fn test_api_config_builder() {
        let config = ApiConfig::new("https://test.api.com".to_string())
            .with_api_key("test-key".to_string())
            .with_timeout(60)
            .with_max_retries(5);

        assert_eq!(config.base_url, "https://test.api.com");
        assert_eq!(config.api_key, Some("test-key".to_string()));
        assert_eq!(config.timeout, 60);
        assert_eq!(config.max_retries, 5);
    }

    #[tokio::test]
    async fn test_mock_api_client_get_copilots() {
        let mock_data = vec![
            create_mock_copilot("1", "1-7", true),
            create_mock_copilot("2", "1-8", false),
            create_mock_copilot("3", "1-7", true),
        ];

        let client = MockApiClient::new(mock_data);

        // 测试获取所有作业
        let all_copilots = client.get_copilots(None, None).await.unwrap();
        assert_eq!(all_copilots.len(), 3);

        // 测试根据关卡过滤
        let filter = QueryFilter {
            stage_id: Some("1-7".to_string()),
            ..Default::default()
        };
        let filtered = client.get_copilots(Some(filter), None).await.unwrap();
        assert_eq!(filtered.len(), 2);

        // 测试推荐作业过滤
        let recommended_filter = QueryFilter {
            recommended_only: Some(true),
            ..Default::default()
        };
        let recommended = client.get_copilots(Some(recommended_filter), None).await.unwrap();
        assert_eq!(recommended.len(), 2);
    }

    #[tokio::test]
    async fn test_mock_api_client_get_by_id() {
        let mock_data = vec![
            create_mock_copilot("1", "1-7", true),
        ];

        let client = MockApiClient::new(mock_data);

        // 测试获取存在的作业
        let copilot = client.get_copilot_by_id("1").await.unwrap();
        assert_eq!(copilot.id, "1");

        // 测试获取不存在的作业
        let result = client.get_copilot_by_id("999").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CopilotError::CopilotNotFound(_) => (),
            _ => panic!("Expected CopilotNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_mock_api_client_search() {
        let mock_data = vec![
            create_mock_copilot("1", "1-7", true),
            create_mock_copilot("2", "2-8", false),
        ];

        let client = MockApiClient::new(mock_data);

        // 测试搜索作业名称
        let results = client.search_copilots("Test Copilot 1").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "1");

        // 测试搜索关卡ID
        let results = client.search_copilots("1-7").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].stage_id, "1-7");

        // 测试搜索不存在的内容
        let results = client.search_copilots("nonexistent").await.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_mock_api_client_error_handling() {
        let client = MockApiClient::new(vec![]).with_failure();

        let result = client.get_copilots(None, None).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CopilotError::ApiError(_) => (),
            _ => panic!("Expected ApiError"),
        }

        let health = client.health_check().await.unwrap();
        assert!(!health);
    }

    #[test]
    fn test_pagination_params() {
        let params = PaginationParams::default();
        assert_eq!(params.page, 1);
        assert_eq!(params.page_size, 20);
        assert_eq!(params.sort_by, Some("updated_at".to_string()));
        assert_eq!(params.sort_order, Some("desc".to_string()));
    }

    #[test]
    fn test_query_filter() {
        let filter = QueryFilter {
            stage_id: Some("1-7".to_string()),
            min_difficulty: Some(3),
            max_difficulty: Some(7),
            ..Default::default()
        };

        assert_eq!(filter.stage_id, Some("1-7".to_string()));
        assert_eq!(filter.min_difficulty, Some(3));
        assert_eq!(filter.max_difficulty, Some(7));
    }
}