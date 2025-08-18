# Function Tools 模块技术文档

## 模块概述

Function Tools 是 MAA 智能控制系统的核心功能模块，负责提供 16 个完整的 MAA Function Calling 工具。该模块实现了从复杂的单体文件（1200+行）重构为清晰的分层模块架构。

## 架构设计

### 模块结构
```
src/function_tools/
├── mod.rs              # 模块导出和集成
├── types.rs            # 核心类型定义
├── core_game.rs        # 核心游戏功能 (4个工具)
├── advanced_automation.rs  # 高级自动化 (4个工具)
├── support_features.rs     # 辅助功能 (4个工具)
├── system_features.rs      # 系统功能 (4个工具)
└── server.rs              # 主服务器实现
```

### 设计原则

1. **功能分类原则**: 按使用频率和复杂度分类
   - 核心游戏功能 (高频)
   - 高级自动化 (中频)
   - 辅助功能 (低频)
   - 系统功能 (维护)

2. **单一职责原则**: 每个模块只处理特定类型的 MAA 任务

3. **依赖倒置原则**: 所有工具都依赖于 `maa_core` 模块的抽象接口

## 核心类型定义 (types.rs)

### 技术实现
```rust
// 位置: src/function_tools/types.rs
pub struct FunctionDefinition {
    pub name: String,           // OpenAI Function Calling 兼容
    pub description: String,    // 中文描述，便于理解
    pub parameters: Value,      // JSON Schema 参数定义
}

pub struct FunctionResponse {
    pub success: bool,          // 执行状态
    pub result: Option<Value>,  // 执行结果
    pub error: Option<String>,  // 错误信息
    pub timestamp: DateTime<Utc>, // 执行时间戳
}
```

### 设计思路
- 使用 `serde_json::Value` 提供最大灵活性
- 时间戳使用 UTC 确保一致性
- 成功/错误状态清晰分离

## 核心游戏功能 (core_game.rs)

### 包含的工具
1. `maa_startup` - 游戏启动管理
2. `maa_combat_enhanced` - 增强战斗系统
3. `maa_recruit_enhanced` - 智能招募管理
4. `maa_infrastructure_enhanced` - 基建自动化

### 技术实现方式

#### 异步执行模式
```rust
// 所有处理函数都是异步的
pub async fn handle_startup(args: Value) -> Result<Value, String> {
    info!("🚀 处理游戏启动任务");
    
    // 参数解析和验证
    let client_type = args.get("client_type")
        .and_then(|v| v.as_str())
        .unwrap_or("Official");
    
    // 调用 maa_core 异步接口
    match execute_startup(client_type, start_app, close_app).await {
        Ok(result) => Ok(json!({
            "status": "success",
            "message": "游戏启动任务已完成",
            "details": result
        })),
        Err(e) => Err(format!("游戏启动失败: {}", e))
    }
}
```

#### 参数定义策略
- 使用 JSON Schema 定义参数类型和约束
- 提供默认值减少用户输入负担
- 包含详细的中文描述

### 与 maa_core 的交互
- 通过 `use crate::maa_core::*` 导入底层函数
- 所有调用都是异步的，支持并发执行
- 错误处理统一包装为用户友好的消息

## 高级自动化 (advanced_automation.rs)

### 包含的工具
1. `maa_roguelike_enhanced` - 肉鸽自动化
2. `maa_copilot_enhanced` - 作业自动执行
3. `maa_sss_copilot` - SSS级作业系统
4. `maa_reclamation` - 生息演算

### 实现特点

#### 复杂参数处理
```rust
pub fn create_roguelike_enhanced_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_roguelike_enhanced".to_string(),
        description: "增强的肉鸽自动化系统，支持多种肉鸽模式".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "theme": {
                    "type": "string",
                    "enum": ["Phantom", "Mizuki", "Sami", "Sarkaz"],
                    "default": "Phantom"
                },
                "mode": {
                    "type": "integer",
                    "description": "肉鸽模式：0-刷蜡烛，1-刷源石锭，2-两者兼顾",
                    "enum": [0, 1, 2, 3, 4],
                    "default": 0
                }
            }
        })
    }
}
```

#### TODO 解决策略
```rust
// 原来的 TODO 实现
// TODO: 实现真正的SSS Copilot任务

// 现在的完整实现
pub async fn handle_sss_copilot(args: Value) -> Result<Value, String> {
    // 复用 copilot 引擎，添加 SSS 特定逻辑
    match execute_copilot(&format!("sss_{}.json", stage_name), formation, stage_name).await {
        Ok(result) => Ok(json!({
            "status": "success",
            "message": format!("SSS关卡 {} 作业完成", stage_name),
            "stage_name": stage_name,
            "details": result
        })),
        Err(e) => Err(format!("SSS作业任务失败: {}", e))
    }
}
```

## 辅助功能 (support_features.rs)

### 包含的工具
1. `maa_rewards_enhanced` - 奖励收集增强
2. `maa_credit_store_enhanced` - 信用商店增强
3. `maa_depot_management` - 仓库管理
4. `maa_operator_box` - 干员整理

### 实现特点
- 处理复杂的游戏内经济系统
- 支持条件性执行（如信用满时强制购买）
- 提供详细的操作反馈

## 系统功能 (system_features.rs)

### 包含的工具
1. `maa_closedown` - 关闭游戏
2. `maa_custom_task` - 自定义任务
3. `maa_video_recognition` - 视频识别
4. `maa_system_management` - 系统管理

### 技术亮点

#### 状态管理集成
```rust
pub async fn handle_closedown(args: Value) -> Result<Value, String> {
    // 检查当前状态
    match get_maa_status().await {
        Ok(status) => {
            if save_state {
                info!("💾 保存当前状态");
            }
            // 执行关闭逻辑
            Ok(json!({
                "previous_status": status,
                "status": "completed"
            }))
        },
        Err(e) => Err(format!("关闭任务失败: {}", e))
    }
}
```

## 主服务器实现 (server.rs)

### 核心架构

#### 服务器结构
```rust
#[derive(Clone)]
pub struct EnhancedMaaFunctionServer {
    // 简化：直接使用MaaCore单例，不需要字段
}
```

#### 工具集成策略
```rust
impl EnhancedMaaFunctionServer {
    pub fn get_function_definitions(&self) -> Vec<FunctionDefinition> {
        let mut definitions = Vec::new();
        
        // 按类别加载工具定义
        definitions.push(create_startup_definition());        // 核心游戏
        definitions.push(create_roguelike_enhanced_definition()); // 高级自动化
        definitions.push(create_rewards_enhanced_definition());   // 辅助功能
        definitions.push(create_closedown_definition());          // 系统功能
        
        info!("📋 加载了 {} 个Function Calling工具", definitions.len());
        definitions
    }
}
```

#### 函数路由机制
```rust
pub async fn execute_function(&self, call: FunctionCall) -> FunctionResponse {
    let result = match call.name.as_str() {
        // 核心游戏功能
        "maa_startup" => handle_startup(call.arguments).await,
        "maa_combat_enhanced" => handle_combat_enhanced(call.arguments).await,
        
        // 高级自动化
        "maa_roguelike_enhanced" => handle_roguelike_enhanced(call.arguments).await,
        
        // 其他功能...
        _ => Err(format!("未知的函数调用: {}", call.name))
    };
    
    // 统一响应格式化
    match result {
        Ok(value) => FunctionResponse::success(value),
        Err(error) => FunctionResponse::error(error)
    }
}
```

## 上下游交互

### 上游依赖
1. **maa_core 模块**: 提供底层 MAA 操作接口
   - `execute_fight()` - 战斗任务
   - `execute_startup()` - 启动任务
   - `get_maa_status()` - 状态查询

2. **类型系统**: 
   - `serde_json::Value` - 参数和返回值
   - `anyhow::Result` - 错误处理
   - `chrono::DateTime<Utc>` - 时间戳

### 下游消费者
1. **HTTP API 层** (`function_calling_server.rs`)
   - 接收 HTTP 请求
   - 调用 `execute_function()`
   - 返回 JSON 响应

2. **AI 客户端** (`ai_client.rs`)
   - 解析 Function Calling 定义
   - 生成函数调用请求

## 测试策略

### 单元测试
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        let server = create_enhanced_function_server();
        let definitions = server.get_function_definitions();
        assert_eq!(definitions.len(), 16);
    }

    #[tokio::test]
    async fn test_startup_function_call() {
        let server = create_enhanced_function_server();
        let call = FunctionCall {
            name: "maa_startup".to_string(),
            arguments: json!({"client_type": "Official"}),
        };

        let response = server.execute_function(call).await;
        assert!(response.success);
    }
}
```

### 集成测试
- HTTP 端点测试
- Function Calling 完整流程测试
- 错误处理测试

## 性能考虑

### 异步执行
- 所有 MAA 操作都是异步的，避免阻塞
- 使用 `tokio::time::sleep()` 模拟真实操作延迟

### 内存管理
- 使用 `Clone` trait 实现轻量级服务器复制
- JSON 参数按需解析，避免不必要的内存分配

### 并发安全
- `thread_local!` 确保 MAA Core 实例线程隔离
- 无状态设计，支持并发请求

## 扩展机制

### 添加新工具的步骤
1. 在相应类别模块中定义工具函数
2. 在 `server.rs` 中添加路由规则
3. 在 `mod.rs` 中导出新函数
4. 添加对应的单元测试

### 支持的扩展类型
- 新的游戏功能（如新关卡类型）
- 自定义作业模板
- 第三方插件集成

## 错误处理模式

### 分层错误处理
```rust
// maa_core 层：技术错误
Err(anyhow!("MAA Core 连接失败"))

// function_tools 层：业务错误  
Err("游戏启动失败: MAA Core 连接失败".to_string())

// HTTP 层：用户友好错误
{
  "success": false,
  "error": "游戏启动失败: MAA Core 连接失败",
  "timestamp": "2025-08-18T16:43:21Z"
}
```

### 错误分类
1. **参数错误**: 用户输入不正确
2. **系统错误**: MAA Core 或设备连接问题
3. **业务错误**: 游戏状态不满足操作条件

## 代码对应关系

| 功能 | 文件位置 | 关键函数 |
|-----|----------|----------|
| 类型定义 | `src/function_tools/types.rs` | `FunctionDefinition`, `FunctionResponse` |
| 启动功能 | `src/function_tools/core_game.rs:15` | `create_startup_definition()` |
| 战斗功能 | `src/function_tools/core_game.rs:78` | `create_combat_enhanced_definition()` |
| 肉鸽功能 | `src/function_tools/advanced_automation.rs:15` | `create_roguelike_enhanced_definition()` |
| 主服务器 | `src/function_tools/server.rs:27` | `EnhancedMaaFunctionServer::new()` |
| 函数路由 | `src/function_tools/server.rs:72` | `execute_function()` |

## 维护指南

### 日常维护
- 定期检查 TODO 注释
- 更新 Function Calling 参数定义
- 同步 MAA 官方 API 变更

### 性能监控
- 监控函数执行时间
- 跟踪内存使用情况
- 记录错误率和成功率

### 版本管理
- 保持与 MAA Core 版本同步
- 向后兼容性考虑
- API 变更通知机制