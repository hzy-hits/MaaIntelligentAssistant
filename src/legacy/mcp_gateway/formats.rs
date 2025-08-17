//! AI Format Handlers
//!
//! 实现各种AI服务的Function Calling格式转换

use async_trait::async_trait;
use serde_json::Value;
use tracing::debug;

use super::UnifiedToolCall;

/// AI格式处理器trait
#[async_trait]
pub trait FormatHandler {
    /// 将AI的Function Calling请求转换为统一格式
    fn convert_function_call(&self, ai_request: Value) -> Result<UnifiedToolCall, String>;
    
    /// 将统一格式响应转换为AI格式
    fn convert_response(&self, unified_response: Value) -> Result<Value, String>;
    
    /// 获取格式名称
    fn get_format_name(&self) -> &'static str;
}

/// OpenAI格式处理器
pub struct OpenAIHandler;

impl OpenAIHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl FormatHandler for OpenAIHandler {
    fn convert_function_call(&self, ai_request: Value) -> Result<UnifiedToolCall, String> {
        debug!("Converting OpenAI function call");
        
        // OpenAI Function Calling格式
        // {
        //   "name": "function_name",
        //   "arguments": "{\"param1\": \"value1\"}"
        // }
        
        let name = ai_request.get("name")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'name' field in OpenAI request")?
            .to_string();
            
        let arguments_str = ai_request.get("arguments")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'arguments' field in OpenAI request")?;
            
        let arguments: Value = serde_json::from_str(arguments_str)
            .map_err(|e| format!("Failed to parse OpenAI arguments: {}", e))?;
            
        let id = ai_request.get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(UnifiedToolCall {
            name,
            arguments,
            id,
        })
    }

    fn convert_response(&self, unified_response: Value) -> Result<Value, String> {
        debug!("Converting response to OpenAI format");
        
        // OpenAI Function Response格式
        // {
        //   "role": "function",
        //   "name": "function_name", 
        //   "content": "response_content"
        // }
        
        let content = if unified_response["success"].as_bool().unwrap_or(false) {
            serde_json::to_string(&unified_response["data"]).unwrap_or_default()
        } else {
            format!("Error: {}", unified_response["error"].as_str().unwrap_or("Unknown error"))
        };

        Ok(serde_json::json!({
            "role": "function",
            "content": content
        }))
    }

    fn get_format_name(&self) -> &'static str {
        "openai"
    }
}

/// Claude格式处理器
pub struct ClaudeHandler;

impl ClaudeHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl FormatHandler for ClaudeHandler {
    fn convert_function_call(&self, ai_request: Value) -> Result<UnifiedToolCall, String> {
        debug!("Converting Claude function call");
        
        // Claude Tool Use格式
        // {
        //   "type": "tool_use",
        //   "id": "toolu_123",
        //   "name": "tool_name",
        //   "input": {"param1": "value1"}
        // }
        
        if ai_request.get("type").and_then(|v| v.as_str()) != Some("tool_use") {
            return Err("Not a Claude tool_use request".to_string());
        }

        let name = ai_request.get("name")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'name' field in Claude request")?
            .to_string();
            
        let arguments = ai_request.get("input")
            .ok_or("Missing 'input' field in Claude request")?
            .clone();
            
        let id = ai_request.get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(UnifiedToolCall {
            name,
            arguments,
            id,
        })
    }

    fn convert_response(&self, unified_response: Value) -> Result<Value, String> {
        debug!("Converting response to Claude format");
        
        // Claude Tool Result格式
        // {
        //   "type": "tool_result",
        //   "tool_use_id": "toolu_123",
        //   "content": [{"type": "text", "text": "result"}],
        //   "is_error": false
        // }
        
        let is_error = !unified_response["success"].as_bool().unwrap_or(false);
        let content = if is_error {
            unified_response["error"].as_str().unwrap_or("Unknown error").to_string()
        } else {
            serde_json::to_string(&unified_response["data"]).unwrap_or_default()
        };
        
        let tool_use_id = unified_response.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        Ok(serde_json::json!({
            "type": "tool_result",
            "tool_use_id": tool_use_id,
            "content": [{"type": "text", "text": content}],
            "is_error": is_error
        }))
    }

    fn get_format_name(&self) -> &'static str {
        "claude"
    }
}

/// Qwen格式处理器
pub struct QwenHandler;

impl QwenHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl FormatHandler for QwenHandler {
    fn convert_function_call(&self, ai_request: Value) -> Result<UnifiedToolCall, String> {
        debug!("Converting Qwen function call");
        
        // Qwen Function Call格式（类似OpenAI）
        // {
        //   "name": "function_name",
        //   "parameters": {"param1": "value1"}
        // }
        
        let name = ai_request.get("name")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'name' field in Qwen request")?
            .to_string();
            
        let arguments = ai_request.get("parameters")
            .or_else(|| ai_request.get("arguments"))
            .ok_or("Missing 'parameters' field in Qwen request")?
            .clone();
            
        let id = ai_request.get("call_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(UnifiedToolCall {
            name,
            arguments,
            id,
        })
    }

    fn convert_response(&self, unified_response: Value) -> Result<Value, String> {
        debug!("Converting response to Qwen format");
        
        // Qwen Function Response格式
        // {
        //   "name": "function_name",
        //   "content": "response_content"
        // }
        
        let content = if unified_response["success"].as_bool().unwrap_or(false) {
            serde_json::to_string(&unified_response["data"]).unwrap_or_default()
        } else {
            format!("Error: {}", unified_response["error"].as_str().unwrap_or("Unknown error"))
        };

        Ok(serde_json::json!({
            "content": content
        }))
    }

    fn get_format_name(&self) -> &'static str {
        "qwen"
    }
}

/// Kimi格式处理器
pub struct KimiHandler;

impl KimiHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl FormatHandler for KimiHandler {
    fn convert_function_call(&self, ai_request: Value) -> Result<UnifiedToolCall, String> {
        debug!("Converting Kimi function call");
        
        // Kimi Function Call格式（基于OpenAI标准）
        // {
        //   "function": {
        //     "name": "function_name",
        //     "arguments": "{\"param1\": \"value1\"}"
        //   },
        //   "id": "call_123"
        // }
        
        let function = ai_request.get("function")
            .ok_or("Missing 'function' field in Kimi request")?;
            
        let name = function.get("name")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'name' field in Kimi function")?
            .to_string();
            
        let arguments_str = function.get("arguments")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'arguments' field in Kimi function")?;
            
        let arguments: Value = serde_json::from_str(arguments_str)
            .map_err(|e| format!("Failed to parse Kimi arguments: {}", e))?;
            
        let id = ai_request.get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(UnifiedToolCall {
            name,
            arguments,
            id,
        })
    }

    fn convert_response(&self, unified_response: Value) -> Result<Value, String> {
        debug!("Converting response to Kimi format");
        
        // Kimi Function Response格式
        // {
        //   "role": "tool",
        //   "content": "response_content",
        //   "tool_call_id": "call_123"
        // }
        
        let content = if unified_response["success"].as_bool().unwrap_or(false) {
            serde_json::to_string(&unified_response["data"]).unwrap_or_default()
        } else {
            format!("Error: {}", unified_response["error"].as_str().unwrap_or("Unknown error"))
        };
        
        let tool_call_id = unified_response.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        Ok(serde_json::json!({
            "role": "tool",
            "content": content,
            "tool_call_id": tool_call_id
        }))
    }

    fn get_format_name(&self) -> &'static str {
        "kimi"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_conversion() {
        let handler = OpenAIHandler::new();
        
        let ai_request = serde_json::json!({
            "name": "maa_status",
            "arguments": "{\"verbose\": true}",
            "id": "call_123"
        });
        
        let unified = handler.convert_function_call(ai_request).unwrap();
        assert_eq!(unified.name, "maa_status");
        assert_eq!(unified.id, Some("call_123".to_string()));
        assert_eq!(unified.arguments["verbose"], true);
    }

    #[test]
    fn test_claude_conversion() {
        let handler = ClaudeHandler::new();
        
        let ai_request = serde_json::json!({
            "type": "tool_use",
            "id": "toolu_123",
            "name": "maa_command",
            "input": {"command": "start fight"}
        });
        
        let unified = handler.convert_function_call(ai_request).unwrap();
        assert_eq!(unified.name, "maa_command");
        assert_eq!(unified.id, Some("toolu_123".to_string()));
        assert_eq!(unified.arguments["command"], "start fight");
    }

    #[test]
    fn test_response_conversion() {
        let handler = OpenAIHandler::new();
        
        let unified_response = serde_json::json!({
            "id": "test_123",
            "success": true,
            "data": {"status": "ok"},
            "error": null
        });
        
        let ai_response = handler.convert_response(unified_response).unwrap();
        assert_eq!(ai_response["role"], "function");
        assert!(ai_response["content"].as_str().unwrap().contains("status"));
    }
}