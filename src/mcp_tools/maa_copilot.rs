//! MAA Copilot Tool
//!
//! This tool finds and analyzes copilot configurations for specific stages
//! with smart recommendations.

use std::sync::Arc;
use async_trait::async_trait;
use serde_json::{json, Value};
use tracing::{debug, info};

use crate::maa_adapter::MaaAdapterTrait;
use super::{McpTool, McpError, McpResult, validation, response};

/// MAA Copilot Tool for finding and managing copilot configurations
pub struct MaaCopilotTool {
    _maa_adapter: Arc<dyn MaaAdapterTrait + Send + Sync>,
}

impl MaaCopilotTool {
    /// Create a new MAA copilot tool
    pub fn new(maa_adapter: Arc<dyn MaaAdapterTrait + Send + Sync>) -> McpResult<Self> {
        Ok(Self { _maa_adapter: maa_adapter })
    }
    
    /// Search for copilot configurations for a specific stage
    async fn search_copilots(&self, stage: &str, mode: &str) -> McpResult<Value> {
        debug!("Searching copilots for stage: {} with mode: {}", stage, mode);
        
        // Mock copilot data - in real implementation this would:
        // 1. Query the copilot database/API
        // 2. Filter by stage and user preferences
        // 3. Sort by success rate and other metrics
        
        let mock_copilots = match stage {
            "1-7" => vec![
                json!({
                    "id": "cop_1_7_001",
                    "name": "1-7 Auto Farm",
                    "author": "MAA_User_123",
                    "stage": "1-7",
                    "success_rate": 99.5,
                    "average_time": 180,
                    "required_operators": [
                        {"name": "Kroos", "elite": 1, "level": 55, "skill_level": 7},
                        {"name": "Melantha", "elite": 1, "level": 55, "skill_level": 7}
                    ],
                    "tags": ["beginner", "farming", "stable"],
                    "description": "Simple and stable auto for 1-7 farming",
                    "views": 15420,
                    "likes": 892
                }),
                json!({
                    "id": "cop_1_7_002",
                    "name": "1-7 Speed Run",
                    "author": "SpeedRunner_Pro",
                    "stage": "1-7",
                    "success_rate": 95.2,
                    "average_time": 120,
                    "required_operators": [
                        {"name": "SilverAsh", "elite": 2, "level": 70, "skill_level": "M3"},
                        {"name": "Exusiai", "elite": 2, "level": 60, "skill_level": "M1"}
                    ],
                    "tags": ["speed", "high-requirement"],
                    "description": "Fast clear with high-level operators",
                    "views": 8500,
                    "likes": 340
                })
            ],
            stage if stage.starts_with("H") => vec![
                json!({
                    "id": format!("cop_{}_hard", stage.replace("-", "_")),
                    "name": format!("{} Hard Mode Clear", stage),
                    "author": "HardMode_Expert",
                    "stage": stage,
                    "success_rate": 87.3,
                    "average_time": 450,
                    "required_operators": [
                        {"name": "SilverAsh", "elite": 2, "level": 90, "skill_level": "M3"},
                        {"name": "Eyjafjalla", "elite": 2, "level": 90, "skill_level": "M3"},
                        {"name": "Saria", "elite": 2, "level": 80, "skill_level": "M1"}
                    ],
                    "tags": ["hard", "high-requirement", "strategy"],
                    "description": "Reliable clear for hard mode with optimal positioning",
                    "views": 5200,
                    "likes": 180
                })
            ],
            _ => vec![
                json!({
                    "id": format!("cop_{}_general", stage.replace("-", "_")),
                    "name": format!("{} General Clear", stage),
                    "author": "General_User",
                    "stage": stage,
                    "success_rate": 92.0,
                    "average_time": 300,
                    "required_operators": [
                        {"name": "Balanced Team", "note": "Various operators depending on stage"}
                    ],
                    "tags": ["general", "stable"],
                    "description": format!("Standard strategy for {}", stage),
                    "views": 2000,
                    "likes": 50
                })
            ]
        };
        
        // Filter based on mode
        let filtered_copilots = match mode {
            "simple" => mock_copilots.into_iter().take(1).collect(),
            "smart" => mock_copilots,
            "advanced" => {
                // In advanced mode, we'd include more detailed analysis
                mock_copilots
            }
            _ => mock_copilots,
        };
        
        Ok(json!({
            "stage": stage,
            "mode": mode,
            "found": filtered_copilots.len(),
            "copilots": filtered_copilots,
            "message": "Mock copilot data. Full implementation would query actual copilot database."
        }))
    }
    
    /// Analyze copilot compatibility with user's operators
    async fn analyze_compatibility(&self, copilot: &Value, user_operators: Option<&[String]>) -> McpResult<Value> {
        debug!("Analyzing copilot compatibility");
        
        let empty_vec = vec![];
        let required_ops = copilot.get("required_operators")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_vec);
        
        let user_ops = user_operators.unwrap_or(&[]);
        
        // Mock compatibility analysis
        let mut compatibility_score = 85.0;
        let mut missing_operators = Vec::new();
        let mut recommendations = Vec::new();
        
        for req_op in required_ops {
            if let Some(op_name) = req_op.get("name").and_then(|v| v.as_str()) {
                if !user_ops.contains(&op_name.to_string()) {
                    missing_operators.push(op_name);
                    compatibility_score -= 15.0;
                    recommendations.push(format!("Consider obtaining operator: {}", op_name));
                }
            }
        }
        
        if compatibility_score < 50.0 {
            recommendations.push("This copilot requires significant investment".to_string());
        } else if compatibility_score > 90.0 {
            recommendations.push("High compatibility - recommended for use".to_string());
        }
        
        Ok(json!({
            "compatibility_score": compatibility_score,
            "missing_operators": missing_operators,
            "recommendations": recommendations,
            "can_use": compatibility_score >= 50.0,
            "difficulty": if compatibility_score >= 90.0 { "Easy" } 
                         else if compatibility_score >= 70.0 { "Medium" } 
                         else { "Hard" }
        }))
    }
    
    /// Get detailed analysis of a copilot
    async fn get_detailed_analysis(&self, copilot: &Value) -> McpResult<Value> {
        debug!("Getting detailed copilot analysis");
        
        let _stage = copilot.get("stage").and_then(|v| v.as_str()).unwrap_or("unknown");
        let success_rate = copilot.get("success_rate").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let avg_time = copilot.get("average_time").and_then(|v| v.as_u64()).unwrap_or(0);
        
        Ok(json!({
            "performance": {
                "success_rate": success_rate,
                "average_time_seconds": avg_time,
                "stability": if success_rate >= 95.0 { "Very Stable" } 
                           else if success_rate >= 85.0 { "Stable" } 
                           else { "Needs Attention" },
                "speed": if avg_time <= 180 { "Fast" } 
                        else if avg_time <= 300 { "Normal" } 
                        else { "Slow" }
            },
            "requirements": {
                "operator_count": copilot.get("required_operators")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.len())
                    .unwrap_or(0),
                "investment_level": "Medium", // Would be calculated based on operator requirements
                "skill_requirements": "Varies" // Would analyze skill levels needed
            },
            "community": {
                "popularity": "High", // Based on views/likes
                "trust_score": if success_rate >= 90.0 { "High" } else { "Medium" },
                "last_updated": "Recently" // Would show actual update time
            },
            "usage_tips": [
                "Practice the timing for skill activations",
                "Make sure operators are properly positioned",
                "Consider the stage's specific mechanics"
            ]
        }))
    }
}

#[async_trait]
impl McpTool for MaaCopilotTool {
    async fn call(&self, params: Value) -> Result<Value, McpError> {
        debug!("MaaCopilotTool::call with params: {}", params);
        
        // Validate required parameters
        let stage: String = validation::validate_non_empty_string(&params, "stage")?;
        
        // Optional parameters
        let mode: String = validation::validate_optional_param(&params, "mode", "string")?
            .unwrap_or_else(|| "simple".to_string());
        let user_operators: Option<Vec<String>> = validation::validate_optional_param(&params, "user_operators", "array")?;
        let include_analysis: bool = validation::validate_optional_param(&params, "include_analysis", "boolean")?
            .unwrap_or(false);
        
        info!("Searching copilots for stage '{}' with mode '{}'", stage, mode);
        
        // Search for copilots
        let search_result = self.search_copilots(&stage, &mode).await?;
        let empty_vec = vec![];
        let copilots = search_result.get("copilots")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_vec);
        
        let mut enhanced_copilots = Vec::new();
        
        for copilot in copilots {
            let mut enhanced_copilot = copilot.clone();
            
            // Add compatibility analysis if user operators provided
            if let Some(ref user_ops) = user_operators {
                let compatibility = self.analyze_compatibility(copilot, Some(user_ops)).await?;
                enhanced_copilot["compatibility"] = compatibility;
            }
            
            // Add detailed analysis if requested
            if include_analysis {
                let detailed_analysis = self.get_detailed_analysis(copilot).await?;
                enhanced_copilot["detailed_analysis"] = detailed_analysis;
            }
            
            enhanced_copilots.push(enhanced_copilot);
        }
        
        Ok(response::success(json!({
            "stage": stage,
            "mode": mode,
            "user_operators": user_operators,
            "include_analysis": include_analysis,
            "search_result": {
                "found": enhanced_copilots.len(),
                "copilots": enhanced_copilots
            }
        })))
    }
    
    fn get_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "stage": {
                    "type": "string",
                    "description": "Stage identifier (e.g., '1-7', 'H5-3')"
                },
                "mode": {
                    "type": "string",
                    "description": "Search mode: simple, smart, or advanced",
                    "enum": ["simple", "smart", "advanced"],
                    "default": "simple"
                },
                "user_operators": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "List of user operators for compatibility matching"
                },
                "include_analysis": {
                    "type": "boolean",
                    "description": "Include detailed analysis of found copilots",
                    "default": false
                }
            },
            "required": ["stage"]
        })
    }
    
    fn get_name(&self) -> &'static str {
        "maa_copilot"
    }
    
    fn get_description(&self) -> &'static str {
        "Find and analyze copilot configurations for specific stages with smart recommendations"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maa_adapter::{MaaAdapter, MaaConfig};
    use std::sync::Arc;
    
    async fn create_test_tool() -> MaaCopilotTool {
        let config = MaaConfig::default();
        let adapter = Arc::new(MaaAdapter::new(config).await.unwrap());
        MaaCopilotTool::new(adapter).unwrap()
    }
    
    #[tokio::test]
    async fn test_search_copilots() {
        let tool = create_test_tool().await;
        
        // Test simple search
        let result = tool.search_copilots("1-7", "simple").await.unwrap();
        assert_eq!(result["stage"], "1-7");
        assert_eq!(result["mode"], "simple");
        assert!(result["copilots"].is_array());
        
        // Test hard stage search
        let result = tool.search_copilots("H5-3", "smart").await.unwrap();
        assert_eq!(result["stage"], "H5-3");
        assert!(result["found"].as_u64().unwrap() > 0);
    }
    
    #[tokio::test]
    async fn test_analyze_compatibility() {
        let tool = create_test_tool().await;
        
        let mock_copilot = json!({
            "required_operators": [
                {"name": "SilverAsh", "elite": 2},
                {"name": "Eyjafjalla", "elite": 2}
            ]
        });
        
        let user_ops = vec!["SilverAsh".to_string(), "Chen".to_string()];
        let result = tool.analyze_compatibility(&mock_copilot, Some(&user_ops)).await.unwrap();
        
        assert!(result["compatibility_score"].is_number());
        assert!(result["missing_operators"].is_array());
        assert!(result["can_use"].is_boolean());
    }
    
    #[tokio::test]
    async fn test_detailed_analysis() {
        let tool = create_test_tool().await;
        
        let mock_copilot = json!({
            "stage": "1-7",
            "success_rate": 95.5,
            "average_time": 180,
            "required_operators": [{"name": "Kroos"}]
        });
        
        let result = tool.get_detailed_analysis(&mock_copilot).await.unwrap();
        
        assert!(result["performance"].is_object());
        assert!(result["requirements"].is_object());
        assert!(result["community"].is_object());
        assert!(result["usage_tips"].is_array());
    }
    
    #[tokio::test]
    async fn test_tool_call() {
        let tool = create_test_tool().await;
        
        // Basic call
        let params = json!({
            "stage": "1-7"
        });
        let result = tool.call(params).await.unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["data"]["stage"], "1-7");
        
        // Call with all options
        let params = json!({
            "stage": "H5-3",
            "mode": "smart",
            "user_operators": ["SilverAsh", "Eyjafjalla"],
            "include_analysis": true
        });
        let result = tool.call(params).await.unwrap();
        assert_eq!(result["success"], true);
        assert!(result["data"]["search_result"]["copilots"].is_array());
    }
    
    #[tokio::test]
    async fn test_invalid_parameters() {
        let tool = create_test_tool().await;
        
        // Missing stage
        let params = json!({
            "mode": "simple"
        });
        let result = tool.call(params).await;
        assert!(result.is_err());
        
        // Empty stage
        let params = json!({
            "stage": ""
        });
        let result = tool.call(params).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_tool_metadata() {
        let tool = create_test_tool().await;
        
        assert_eq!(tool.get_name(), "maa_copilot");
        assert!(!tool.get_description().is_empty());
        
        let schema = tool.get_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["stage"].is_object());
        assert!(schema["required"].as_array().unwrap().contains(&json!("stage")));
    }
}