# MCP 工具模块交付文档

## 1. 实现总结

### 完成的功能
1. **重构 MCP 工具模块** - 将自定义的 McpTool trait 重构为标准实现
2. **简化的 rmcp 兼容实现** - 由于 rmcp 0.5.0 SDK 的复杂性（版本冲突问题），实现了兼容的包装器
3. **四个核心工具** - 实现了 maa_status、maa_command、maa_copilot、maa_operators 四个工具
4. **统一的参数和响应结构** - 使用 serde 进行序列化，确保与不同 AI 提供商兼容
5. **错误处理优化** - 简化错误类型，使用 String 而非复杂的错误枚举

### 设计的架构
```rust
/// MAA 工具服务器
#[derive(Clone)]
pub struct MaaToolsServer {
    maa_adapter: Arc<dyn MaaAdapterTrait + Send + Sync>,
}

// 四个核心工具方法
impl MaaToolsServer {
    pub async fn maa_status(&self, params: MaaStatusParams) -> Result<MaaStatusResponse, String>
    pub async fn maa_command(&self, params: MaaCommandParams) -> Result<MaaCommandResponse, String>
    pub async fn maa_copilot(&self, params: MaaCopilotParams) -> Result<MaaCopilotResponse, String>
    pub async fn maa_operators(&self, params: MaaOperatorsParams) -> Result<MaaOperatorsResponse, String>
}
```

### 关键技术决策
1. **暂时移除 rmcp 0.5.0 依赖** - 由于 schemars 版本冲突和复杂的宏系统
2. **保持类型别名向后兼容** - `pub type MaaToolsHandler = MaaToolsServer;`
3. **统一的 JSON 序列化** - 所有参数和响应都支持 serde 序列化
4. **类型转换处理** - 将 i32 任务ID转换为 u32 以保持API一致性

## 2. 与架构对比

### 与架构师任务的差异
- **原计划**: 使用官方 rmcp 0.5.0 SDK 和 #[tool_router] 宏
- **实际实现**: 简化的兼容包装器，避免复杂的宏和版本冲突

### 偏离原设计的原因
1. **schemars 版本冲突** - rmcp 0.5.0 需要 schemars 1.0，项目使用 0.8
2. **复杂的宏系统** - #[tool_router] 宏需要特定的类型约束和 Future trait
3. **编译兼容性** - 优先保证项目能成功编译和集成

### 新增的功能
1. **统一的响应结构** - 每个工具都有对应的响应类型
2. **时间戳支持** - 所有响应都包含 UTC 时间戳
3. **详细的参数文档** - 每个参数都有清晰的文档说明

## 3. 测试覆盖

### 测试代码覆盖内容
```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_rmcp_tools_creation() // 测试工具服务器创建
    
    #[tokio::test] 
    async fn test_command_parsing() // 测试命令解析功能
    
    #[tokio::test]
    async fn test_parameter_schemas() // 测试参数序列化和反序列化
}
```

### 测试场景说明
1. **工具创建测试** - 验证 MaaToolsServer 能够正确创建
2. **命令解析测试** - 验证自然语言命令能够转换为 MAA 任务类型
3. **参数验证测试** - 验证所有参数结构的 JSON 序列化/反序列化

### 限制说明
- 由于 MAA Core FFI 链接问题，无法运行完整的集成测试
- 当前测试依赖模拟的 MAA 适配器，未实际调用 MAA 功能

## 4. 集成指导

### 为集成测试提供场景构造指导

#### 场景 1: MAA 状态查询
```rust
let params = MaaStatusParams { verbose: true };
let response = tools.maa_status(params).await.unwrap();
assert!(response.status.contains("Ready"));
```

#### 场景 2: 自然语言命令
```rust
let params = MaaCommandParams {
    command: "帮我做日常".to_string(),
    context: Some("明日方舟".to_string()),
};
let response = tools.maa_command(params).await.unwrap();
assert!(response.parsed_tasks > 0);
```

#### 场景 3: 作业执行
```rust
let params = MaaCopilotParams {
    copilot_config: serde_json::json!({"stage": "1-7", "formations": []}),
    name: Some("测试作业".to_string()),
};
let response = tools.maa_copilot(params).await.unwrap();
assert!(response.task_id > 0);
```

### 依赖关系说明
- **maa_adapter** - 必须正确初始化 MAA 适配器
- **chrono** - 用于时间戳生成
- **serde_json** - 用于参数和响应的序列化

### 潜在风险点
1. **MAA Core 链接问题** - FFI 函数未找到，需要正确配置 MAA Core 库
2. **类型转换** - i32 到 u32 的转换可能在极端情况下失败
3. **错误处理** - 当前使用简单的 String 错误，可能需要更详细的错误信息

## 5. 使用示例

### 代码使用示例
```rust
use maa_intelligent_server::mcp_tools::{
    MaaToolsServer, register_rmcp_tools, 
    MaaStatusParams, MaaCommandParams
};

// 创建工具服务器
let maa_adapter = Arc::new(MaaAdapter::new(config).await?);
let tools = register_rmcp_tools(maa_adapter);

// 查询状态
let status_params = MaaStatusParams { verbose: false };
let status_response = tools.maa_status(status_params).await?;
println!("MAA Status: {}", status_response.status);

// 执行命令
let command_params = MaaCommandParams {
    command: "截图".to_string(),
    context: None,
};
let command_response = tools.maa_command(command_params).await?;
println!("Created {} tasks", command_response.parsed_tasks);
```

### 配置示例
```rust
// 在 MCP 网关中使用
let gateway = McpGateway::new(
    maa_adapter.clone(),
    register_rmcp_tools(maa_adapter),
    gateway_config
).await?;
```

### 常见问题解答

**Q: 为什么不使用官方 rmcp 0.5.0 SDK？**
A: 由于 schemars 版本冲突和复杂的宏系统，选择实现简化的兼容包装器确保项目编译成功。

**Q: 如何添加新的工具？**
A: 在 MaaToolsServer 中添加新方法，定义对应的参数和响应结构，并在 MCP 网关中添加路由。

**Q: 类型转换安全吗？**
A: 当前使用 `as u32` 进行转换，在正常情况下安全，但建议后续使用 `try_into()` 进行安全转换。

## 6. 后续改进建议

1. **恢复 rmcp 0.5.0 集成** - 解决版本冲突问题后，迁移到官方 SDK
2. **增强错误处理** - 使用结构化错误类型替代 String
3. **添加更多工具** - 实现基建管理、招募优化等专业工具
4. **性能优化** - 添加工具调用的缓存和批处理机制
5. **测试完善** - 解决 FFI 链接问题，添加完整的集成测试

---

**交付状态**: ✅ 完成
**编译状态**: ✅ 成功
**测试状态**: ⚠️  单元测试通过，集成测试因 FFI 问题暂无法运行
**文档状态**: ✅ 完成
