# Function Tools 模块技术文档

## 模块概述

Function Tools 是 MAA 智能控制系统的核心功能模块，提供 16 个完整的 MAA Function Calling 工具。该模块在 2025-08-18 重构后实现了：

- **增强的工具描述**: 参考 maa-knowledge 添加了详细的使用场景和参数说明
- **智能自然语言解析**: 支持更多中文游戏术语和别名
- **统一响应格式**: 完善的错误处理和资源使用统计
- **上下文感知**: 工具间状态共享和任务链推荐

## 架构设计

### 模块结构
```
src/function_tools/
├── mod.rs              # 模块导出和集成
├── types.rs            # 增强的类型定义（错误处理、响应格式、上下文）
├── core_game.rs        # 核心游戏功能 (4个工具) - 已增强
├── advanced_automation.rs  # 高级自动化 (4个工具)
├── support_features.rs     # 辅助功能 (4个工具)
├── system_features.rs      # 系统功能 (4个工具)
├── context.rs          # 上下文管理和任务链推荐
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

### 增强的类型系统
```rust
// 位置: src/function_tools/types.rs

// 增强的响应类型
pub struct FunctionResponse {
    pub success: bool,
    pub result: Option<Value>,
    pub error: Option<MaaError>,           // 统一错误类型
    pub timestamp: DateTime<Utc>,
    pub execution_time_ms: Option<u64>,    // 执行时间
    pub metadata: ResponseMetadata,        // 元数据
}

// 细化的错误类型
pub struct MaaError {
    pub error_type: ErrorType,      // 错误分类
    pub message: String,            // 错误消息
    pub details: Option<String>,    // 详细信息
    pub suggestion: Option<String>, // 解决建议
    pub error_code: Option<String>, // 错误码
}

// 上下文管理
pub struct TaskContext {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub game_state: GameState,      // 游戏状态
    pub last_operations: Vec<String>, // 历史操作
    pub recommendations: Vec<String>, // 智能推荐
}

// 资源使用统计
pub struct ResourceUsage {
    pub sanity_used: Option<i32>,
    pub medicine_used: Option<i32>,
    pub stone_used: Option<i32>,
    pub recruit_tickets_used: Option<i32>,
    pub items_gained: HashMap<String, i32>,
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

### 新增功能亮点

### 1. 智能自然语言解析

#### 中文游戏术语支持
- **关卡别名**: 狗粮=1-7、龙门币本=CE-5、经验书本=LS-5
- **数字识别**: 支持中文数字（一、二、三等）
- **材料映射**: 固源岩→1-7、糖聚块→S4-1

```rust
// src/maa_core/basic_ops.rs:519
fn parse_fight_command(command: &str) -> Result<(String, i32)> {
    // 支持更多中文别名和数字表达
    if cmd_lower.contains("龙门币") || cmd_lower.contains("金币") {
        "CE-5"
    } else if cmd_lower.contains("狗粮") || cmd_lower.contains("经验") {
        "1-7"
    }
    // ...
}
```

### 2. 统一错误处理系统

#### 分类错误管理
```rust
pub enum ErrorType {
    ParameterError,    // 参数错误
    MaaCoreError,     // MAA核心错误
    DeviceError,      // 设备连接错误
    GameStateError,   // 游戏状态错误
    TimeoutError,     // 超时错误
}

// 使用示例
let error = MaaError::parameter_error(
    "不支持的客户端类型",
    Some("支持: Official, Bilibili, txwy...")
);
FunctionResponse::error("maa_startup", error)
```

### 3. 上下文感知系统

#### 智能任务链推荐
```rust
// src/function_tools/context.rs
fn generate_recommendations(user_id: &str, current_operation: &str) -> Vec<String> {
    match current_operation {
        "maa_startup" => vec![
            "建议接下来执行 maa_rewards_enhanced 收集每日奖励",
            "可以执行 maa_infrastructure_enhanced 进行基建管理",
        ],
        "maa_combat_enhanced" => {
            if context.game_state.current_sanity < 20 {
                vec!["理智不足，建议使用理智药或等待恢复"]
            }
        }
    }
}
```

#### 游戏状态跟踪
```rust
pub struct GameState {
    pub current_sanity: Option<i32>,
    pub medicine_count: Option<i32>,
    pub recruit_tickets: Option<i32>,
    pub last_login: Option<DateTime<Utc>>,
}

// 自动提醒系统
fn check_reminders(user_id: &str) -> Vec<String> {
    if current_sanity >= max_sanity - 10 {
        vec!["理智即将满值，建议及时使用"]
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

## 优化成果总结

### 量化指标
- **工具描述增强**: 16个工具全部增加详细的使用场景和参数说明
- **自然语言支持**: 新增30+中文游戏术语支持
- **错误处理**: 6种错误类型分类 + 智能建议系统
- **上下文管理**: 增加任务链推荐和状态跟踪

### 用户体验提升
- **更容易理解**: 中文命令 “刷龙门币本用完理智” → CE-5 无限战斗
- **更好的错误反馈**: 明确的错误类型和解决建议
- **智能推荐**: 根据当前操作和游戏状态推荐后续任务
- **资源跟踪**: 显示理智、材料使用情况

### 维护指南

#### 日常维护
- 定期更新游戏术语映射表
- 监控错误率和用户反馈
- 优化任务链推荐逻辑

#### 扩展指南
- 新增工具: 在对应类别模块中添加工具定义和处理函数
- 扩展语言支持: 在 `maa_core/basic_ops.rs` 中添加新的解析规则
- 新增上下文: 在 `context.rs` 中扩展推荐逻辑

### 性能监控
- 监控函数执行时间
- 跟踪内存使用情况
- 记录错误率和成功率

### 版本管理
- 保持与 MAA Core 版本同步
- 向后兼容性考虑
- API 变更通知机制