# 开发工作流

## 快速启动
```bash
# 1. 启动后端服务 (两种选择)
cargo run --bin maa-optimized-server     # V2优化版本(推荐)
cargo run --bin maa-intelligent-server   # V1稳定版本

# 2. 开发模式(无需真实MAA环境)
cargo run --bin maa-optimized-server --no-default-features --features stub-mode

# 3. 启动前端(可选)
cd maa-chat-ui && npm run dev

# 4. 健康检查
curl localhost:8080/health
```

## 调试和测试
```bash
# 编译检查
cargo check

# 运行测试
cargo test

# 格式化代码
cargo fmt

# 代码检查
cargo clippy

# 查看日志
tail -f logs/maa-server.log
```

## API测试
```bash
# 获取工具列表
curl localhost:8080/tools | jq

# 执行Function Call
curl -X POST localhost:8080/call \
  -H "Content-Type: application/json" \
  -d '{
    "function_call": {
      "name": "maa_take_screenshot",
      "arguments": {}
    }
  }'

# 查看任务状态
curl localhost:8080/status | jq

# SSE实时更新 (V2版本)
curl -N -H "Accept: text/event-stream" localhost:8080/sse/tasks

# AI聊天测试
curl -X POST localhost:8080/chat \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [{"role": "user", "content": "帮我刷1-7"}]
  }'
```

## Git工作流
```bash
# 创建功能分支
git checkout -b feature/python-integration

# 提交更改
git add .
git commit -m "feat: 添加Python决策层基础架构"

# 推送分支  
git push origin feature/python-integration

# 创建PR (使用gh cli)
gh pr create --title "Python决策层集成" --body "实现PyO3桥接和智能决策引擎"
```

## 代码组织
```
src/
├── bin/                    # 可执行文件入口
│   ├── maa-optimized-server.rs
│   └── maa-intelligent-server.rs
├── maa_core/              # MAA核心模块
├── function_tools/        # Function Calling工具
├── sse/                   # Server-Sent Events
├── ai_client/            # AI客户端集成
└── lib.rs                # 库入口
```

## 添加新功能
### 1. 新增Function Tool
```rust
// 在对应模块添加工具定义
// src/function_tools/core_game.rs

pub fn new_game_feature() -> ToolDefinition {
    ToolDefinition {
        name: "maa_new_feature".to_string(),
        description: "新游戏功能".to_string(),
        parameters: // ...
    }
}
```

### 2. 新增AI Provider
```rust
// src/ai_client/providers/new_provider.rs
pub struct NewProvider {
    // provider implementation
}
```

### 3. 扩展MAA操作
```rust
// src/maa_core/basic_ops.rs
pub fn new_maa_operation() -> Result<String> {
    // implementation
}
```

## 调试技巧
```bash
# 启用详细日志
RUST_LOG=debug cargo run --bin maa-optimized-server

# 使用调试器
RUST_BACKTRACE=1 cargo run --bin maa-optimized-server

# 性能分析
cargo flamegraph --bin maa-optimized-server

# 内存分析  
valgrind --tool=massif target/release/maa-optimized-server
```

## 部署流程
```bash
# 生产构建
cargo build --release

# 运行优化版本
./target/release/maa-optimized-server

# Docker构建
docker build -t maa-server .
docker run -p 8080:8080 maa-server
```

## 环境配置
```bash
# 必需的环境变量
export MAA_CORE_LIB=/path/to/libMaaCore.dylib
export MAA_RESOURCE_PATH=/path/to/resource
export MAA_DEVICE_ADDRESS=localhost:1717

# 可选配置
export RUST_LOG=info
export MAA_SERVER_PORT=8080
export MAA_ENABLE_SSE=true
```

## 故障排除
- **编译错误**: 检查Rust版本和依赖
- **FFI错误**: 确认MAA_CORE_LIB路径正确
- **连接失败**: 验证设备地址和ADB连接
- **内存泄漏**: 使用Valgrind检查C++绑定
- **性能问题**: 启用火焰图分析瓶颈