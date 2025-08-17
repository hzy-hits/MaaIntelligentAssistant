//! MAA Operators Tool
//!
//! This tool manages and analyzes operators including scanning, filtering,
//! and getting recommendations.

use std::sync::Arc;
use async_trait::async_trait;
use serde_json::{json, Value};
use tracing::{debug, info};

use crate::maa_adapter::MaaAdapterTrait;
use super::{McpTool, McpError, McpResult, validation, response};

/// MAA Operators Tool for managing operator data
pub struct MaaOperatorsTool {
    _maa_adapter: Arc<dyn MaaAdapterTrait + Send + Sync>,
}

impl MaaOperatorsTool {
    /// Create a new MAA operators tool
    pub fn new(maa_adapter: Arc<dyn MaaAdapterTrait + Send + Sync>) -> McpResult<Self> {
        Ok(Self { _maa_adapter: maa_adapter })
    }
    
    /// Scan operators from the game
    async fn scan_operators(&self) -> McpResult<Value> {
        debug!("Scanning operators from game");
        
        // In a full implementation, this would:
        // 1. Take a screenshot of the operator roster
        // 2. Use OCR/image recognition to identify operators
        // 3. Update the operator database
        // 4. Return the scan results
        
        // For now, return a mock response
        Ok(json!({
            "scan_started": true,
            "estimated_time": "30 seconds",
            "status": "scanning",
            "message": "Operator scan started. This feature requires full MAA integration.",
            "note": "In full implementation, this would scan the game UI and update operator database"
        }))
    }
    
    /// List operators with optional filtering
    async fn list_operators(&self, filter: Option<&Value>) -> McpResult<Value> {
        debug!("Listing operators with filter: {:?}", filter);
        
        // Mock operator data - in real implementation this would come from operator_manager
        let mock_operators = vec![
            json!({
                "name": "Chen",
                "rarity": 6,
                "elite": 2,
                "level": 90,
                "skill_levels": [7, 7, "M3"],
                "potential": 6,
                "class": "Guard"
            }),
            json!({
                "name": "Silverash",
                "rarity": 6,
                "elite": 2,
                "level": 90,
                "skill_levels": [7, 7, "M3"],
                "potential": 1,
                "class": "Guard"
            }),
            json!({
                "name": "Eyjafjalla",
                "rarity": 6,
                "elite": 2,
                "level": 90,
                "skill_levels": [7, 7, "M3"],
                "potential": 3,
                "class": "Caster"
            })
        ];
        
        // Apply filters if provided
        let mut filtered_operators = mock_operators;
        
        if let Some(filter_obj) = filter {
            if let Some(rarity) = filter_obj.get("rarity").and_then(|v| v.as_u64()) {
                filtered_operators.retain(|op| op["rarity"].as_u64() == Some(rarity));
            }
            
            if let Some(class) = filter_obj.get("class").and_then(|v| v.as_str()) {
                filtered_operators.retain(|op| op["class"].as_str() == Some(class));
            }
            
            if let Some(min_elite) = filter_obj.get("min_elite").and_then(|v| v.as_u64()) {
                filtered_operators.retain(|op| op["elite"].as_u64().unwrap_or(0) >= min_elite);
            }
        }
        
        Ok(json!({
            "operators": filtered_operators,
            "total": filtered_operators.len(),
            "filtered": filter.is_some(),
            "message": "Mock operator data. In full implementation, this would query the operator database."
        }))
    }
    
    /// Analyze operators for team composition recommendations
    async fn analyze_operators(&self, options: Option<&Value>) -> McpResult<Value> {
        debug!("Analyzing operators with options: {:?}", options);
        
        // Mock analysis - in real implementation this would:
        // 1. Analyze current operator roster
        // 2. Identify team composition strengths/weaknesses
        // 3. Suggest improvements or missing operators
        
        Ok(json!({
            "analysis": {
                "total_operators": 150,
                "six_star_count": 25,
                "completion_rate": 0.85,
                "strongest_classes": ["Guard", "Caster", "Medic"],
                "weakest_classes": ["Supporter", "Specialist"],
                "recommendations": [
                    "Consider raising more Supporters for crowd control",
                    "Your Guard roster is very strong",
                    "Missing some key 5-star operators like Specter"
                ]
            },
            "team_suggestions": [
                {
                    "name": "General Clear Team",
                    "operators": ["SilverAsh", "Eyjafjalla", "Saria", "Siege", "Exusiai", "Shining"],
                    "strengths": ["High DPS", "Balanced coverage"],
                    "use_cases": ["Most story stages", "Easy events"]
                }
            ],
            "message": "Mock analysis data. Full implementation would provide detailed roster analysis."
        }))
    }
    
    /// Get operator recommendations for specific content
    async fn recommend_operators(&self, options: Option<&Value>) -> McpResult<Value> {
        debug!("Getting operator recommendations with options: {:?}", options);
        
        let stage = options
            .and_then(|opts| opts.get("stage"))
            .and_then(|v| v.as_str())
            .unwrap_or("general");
        
        // Mock recommendations based on stage type
        let recommendations = match stage {
            stage if stage.starts_with("H") => {
                json!({
                    "stage_type": "Hard Mode",
                    "difficulty": "High",
                    "recommended_operators": [
                        {"name": "SilverAsh", "reason": "Strong physical DPS and range"},
                        {"name": "Saria", "reason": "Tank with healing and arts damage"},
                        {"name": "Eyjafjalla", "reason": "Strong arts DPS"}
                    ],
                    "strategy_tips": [
                        "Focus on high-level operators",
                        "Consider mastery skills",
                        "May require specific tactics"
                    ]
                })
            }
            stage if stage.starts_with("CE") => {
                json!({
                    "stage_type": "LMD Farming",
                    "difficulty": "Low-Medium",
                    "recommended_operators": [
                        {"name": "Any strong DPS", "reason": "Fast clear times"},
                        {"name": "Low-cost operators", "reason": "DP efficiency"}
                    ],
                    "strategy_tips": [
                        "Focus on clear speed",
                        "Use auto-deploy for farming"
                    ]
                })
            }
            _ => {
                json!({
                    "stage_type": "General",
                    "difficulty": "Variable",
                    "recommended_operators": [
                        {"name": "Balanced team", "reason": "Covers most situations"}
                    ],
                    "strategy_tips": [
                        "Adapt based on stage requirements"
                    ]
                })
            }
        };
        
        Ok(json!({
            "stage": stage,
            "recommendations": recommendations,
            "message": "Mock recommendations. Full implementation would analyze stage requirements and user roster."
        }))
    }
}

#[async_trait]
impl McpTool for MaaOperatorsTool {
    async fn call(&self, params: Value) -> Result<Value, McpError> {
        debug!("MaaOperatorsTool::call with params: {}", params);
        
        // Validate required parameters
        let action: String = validation::validate_param(&params, "action", "string")?;
        
        // Optional parameters
        let filter = params.get("filter");
        let options = params.get("options");
        
        info!("Executing operators action: '{}'", action);
        
        let result = match action.as_str() {
            "scan" => {
                self.scan_operators().await?
            }
            "list" => {
                self.list_operators(filter).await?
            }
            "analyze" => {
                self.analyze_operators(options).await?
            }
            "recommend" => {
                self.recommend_operators(options).await?
            }
            _ => {
                return Err(McpError::invalid_param(
                    "action", 
                    "scan|list|analyze|recommend", 
                    format!("Unsupported action: {}", action)
                ));
            }
        };
        
        Ok(response::success(json!({
            "action": action,
            "filter": filter,
            "options": options,
            "result": result
        })))
    }
    
    fn get_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "description": "Action to perform: scan, list, analyze, recommend",
                    "enum": ["scan", "list", "analyze", "recommend"]
                },
                "filter": {
                    "type": "object",
                    "description": "Filter criteria for operators",
                    "properties": {
                        "rarity": {
                            "type": "integer",
                            "description": "Filter by operator rarity (1-6)",
                            "minimum": 1,
                            "maximum": 6
                        },
                        "class": {
                            "type": "string",
                            "description": "Filter by operator class",
                            "enum": ["Guard", "Sniper", "Defender", "Medic", "Supporter", "Caster", "Specialist", "Vanguard"]
                        },
                        "min_elite": {
                            "type": "integer",
                            "description": "Minimum elite level (0-2)",
                            "minimum": 0,
                            "maximum": 2
                        }
                    }
                },
                "options": {
                    "type": "object",
                    "description": "Analysis options",
                    "properties": {
                        "stage": {
                            "type": "string",
                            "description": "Target stage for recommendations"
                        },
                        "team_size": {
                            "type": "integer",
                            "description": "Target team size",
                            "minimum": 1,
                            "maximum": 12
                        },
                        "include_details": {
                            "type": "boolean",
                            "description": "Include detailed analysis",
                            "default": false
                        }
                    }
                }
            },
            "required": ["action"]
        })
    }
    
    fn get_name(&self) -> &'static str {
        "maa_operators"
    }
    
    fn get_description(&self) -> &'static str {
        "Manage and analyze operators including scanning, filtering, and getting recommendations"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maa_adapter::{MaaAdapter, MaaConfig};
    use std::sync::Arc;
    
    async fn create_test_tool() -> MaaOperatorsTool {
        let config = MaaConfig::default();
        let adapter = Arc::new(MaaAdapter::new(config).await.unwrap());
        MaaOperatorsTool::new(adapter).unwrap()
    }
    
    #[tokio::test]
    async fn test_scan_operators() {
        let tool = create_test_tool().await;
        let result = tool.scan_operators().await.unwrap();
        
        assert!(result["scan_started"].as_bool().unwrap_or(false));
        assert!(result["status"].is_string());
    }
    
    #[tokio::test]
    async fn test_list_operators() {
        let tool = create_test_tool().await;
        
        // Test without filter
        let result = tool.list_operators(None).await.unwrap();
        assert!(result["operators"].is_array());
        assert!(result["total"].is_number());
        
        // Test with filter
        let filter = json!({"rarity": 6});
        let result = tool.list_operators(Some(&filter)).await.unwrap();
        assert!(result["operators"].is_array());
        assert_eq!(result["filtered"], true);
    }
    
    #[tokio::test]
    async fn test_analyze_operators() {
        let tool = create_test_tool().await;
        let result = tool.analyze_operators(None).await.unwrap();
        
        assert!(result["analysis"].is_object());
        assert!(result["team_suggestions"].is_array());
    }
    
    #[tokio::test]
    async fn test_recommend_operators() {
        let tool = create_test_tool().await;
        
        let options = json!({"stage": "H5-3"});
        let result = tool.recommend_operators(Some(&options)).await.unwrap();
        
        assert!(result["recommendations"].is_object());
        assert_eq!(result["stage"], "H5-3");
    }
    
    #[tokio::test]
    async fn test_tool_call() {
        let tool = create_test_tool().await;
        
        // Test scan action
        let params = json!({"action": "scan"});
        let result = tool.call(params).await.unwrap();
        assert_eq!(result["success"], true);
        
        // Test list action
        let params = json!({
            "action": "list",
            "filter": {"rarity": 6}
        });
        let result = tool.call(params).await.unwrap();
        assert_eq!(result["success"], true);
        
        // Test invalid action
        let params = json!({"action": "invalid"});
        let result = tool.call(params).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_tool_metadata() {
        let tool = create_test_tool().await;
        
        assert_eq!(tool.get_name(), "maa_operators");
        assert!(!tool.get_description().is_empty());
        
        let schema = tool.get_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["action"].is_object());
        assert!(schema["required"].as_array().unwrap().contains(&json!("action")));
    }
}