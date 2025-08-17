# 开发指南

## 快速开始

### 开发环境准备

```bash
# 1. 克隆项目
git clone --recursive https://github.com/your-org/maa-remote-server.git
cd maa-remote-server

# 2. 安装Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 3. 安装依赖
cargo build

# 4. 启动开发服务器
cargo run
```

### 项目结构

```
maa-remote-server/
├── src/                      # 源代码
│   ├── main.rs              # 主程序入口
│   ├── lib.rs               # 库入口
│   ├── function_calling_server.rs # HTTP服务器
│   ├── maa_adapter/         # MAA适配器
│   ├── mcp_tools/           # 工具集
│   ├── operator_manager/    # 干员管理
│   ├── copilot_matcher/     # 作业匹配
│   └── ai_client/           # AI客户端
├── docs/                    # 文档
├── examples/                # 示例代码
├── tests/                   # 测试
└── maa-official/           # MAA官方子模块
```

## 开发工作流

### 1. 分支管理

```bash
# 主分支
main                    # 稳定版本

# 开发分支
develop                 # 开发主线
feature/module-name     # 功能分支
bugfix/issue-name       # 修复分支
```

### 2. 提交规范

```bash
# 格式
<type>(<scope>): <description>

# 示例
feat(maa_adapter): 添加新的FFI绑定
fix(copilot_matcher): 修复匹配算法bug
docs(api): 更新API文档
test(operator_manager): 添加缓存测试
```

**类型说明**:
- `feat`: 新功能
- `fix`: 修复bug
- `docs`: 文档更新
- `test`: 测试相关
- `refactor`: 重构
- `perf`: 性能优化

### 3. 代码规范

#### Rust代码风格

```bash
# 格式化代码
cargo fmt

# 检查代码
cargo clippy

# 运行测试
cargo test
```

#### 命名约定

```rust
// 模块名: snake_case
mod maa_adapter;

// 结构体: PascalCase
struct MaaAdapter;

// 函数名: snake_case
async fn start_task() -> Result<()>;

// 常量: SCREAMING_SNAKE_CASE
const MAX_RETRY_COUNT: usize = 3;

// 枚举: PascalCase
enum TaskStatus {
    Pending,
    Running,
    Completed,
}
```

#### 错误处理

```rust
// 使用thiserror定义错误
#[derive(Debug, thiserror::Error)]
pub enum MaaError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Task not found: {task_id}")]
    TaskNotFound { task_id: String },
}

// 使用Result类型
pub type MaaResult<T> = Result<T, MaaError>;

// 异步函数错误处理
async fn example_function() -> MaaResult<String> {
    let result = risky_operation()
        .await
        .map_err(|e| MaaError::ConnectionFailed(e.to_string()))?;
    Ok(result)
}
```

## 测试策略

### 1. 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_maa_adapter_initialization() {
        let config = MaaConfig::default();
        let adapter = MaaAdapter::new(config).await;
        assert!(adapter.is_ok());
    }
    
    #[test]
    fn test_operator_parsing() {
        let data = r#"{"name": "山", "rarity": 6}"#;
        let operator: Operator = serde_json::from_str(data).unwrap();
        assert_eq!(operator.name, "山");
    }
}
```

### 2. 集成测试

```rust
// tests/integration_test.rs
use maa_intelligent_server::*;

#[tokio::test]
async fn test_full_workflow() {
    // 初始化服务
    let adapter = MaaAdapter::new(MaaConfig::default()).await?;
    let manager = OperatorManager::new(config).await?;
    
    // 执行完整流程
    let operators = manager.scan_operators().await?;
    assert!(!operators.is_empty());
}
```

### 3. 性能测试

```rust
#[tokio::test]
async fn benchmark_operator_scan() {
    let start = std::time::Instant::now();
    let result = manager.scan_operators().await?;
    let duration = start.elapsed();
    
    assert!(duration < std::time::Duration::from_secs(5));
    assert!(result.operators.len() > 0);
}
```

### 4. 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test maa_adapter

# 运行集成测试
cargo test --test integration_test

# 显示测试输出
cargo test -- --nocapture

# 测试覆盖率
cargo tarpaulin --out Html
```

## 调试指南

### 1. 日志配置

```rust
// 初始化日志
tracing_subscriber::fmt()
    .with_env_filter("debug,maa_intelligent_server=trace")
    .init();

// 使用日志
use tracing::{info, warn, error, debug, trace};

info!("服务器启动，端口: {}", port);
debug!("处理请求: {:?}", request);
error!("操作失败: {}", error);
```

### 2. 环境变量调试

```bash
# 启用详细日志
RUST_LOG=debug cargo run

# 特定模块日志
RUST_LOG=maa_intelligent_server::maa_adapter=trace cargo run

# 启用反向跟踪
RUST_BACKTRACE=1 cargo run

# 完整反向跟踪
RUST_BACKTRACE=full cargo run
```

### 3. 调试技巧

```rust
// 使用dbg!宏调试
let result = dbg!(complex_calculation());

// 条件断点
if debug_mode {
    println!("Debug: {:?}", variable);
}

// 性能监控
let start = std::time::Instant::now();
let result = expensive_operation().await;
println!("耗时: {:?}", start.elapsed());
```

## 代码贡献

### 1. Pull Request流程

```bash
# 1. Fork项目并克隆
git clone https://github.com/your-username/maa-remote-server.git

# 2. 创建功能分支
git checkout -b feature/new-feature

# 3. 提交更改
git add .
git commit -m "feat: 添加新功能"

# 4. 推送分支
git push origin feature/new-feature

# 5. 创建Pull Request
```

### 2. PR检查清单

- [ ] 代码通过 `cargo fmt` 格式化
- [ ] 代码通过 `cargo clippy` 检查
- [ ] 所有测试通过 `cargo test`
- [ ] 添加了适当的测试用例
- [ ] 更新了相关文档
- [ ] 提交信息符合规范

### 3. 代码审查

**审查要点**:
- 代码逻辑正确性
- 错误处理完整性
- 性能影响分析
- 安全性考虑
- 文档完整性

## 性能优化

### 1. 编译优化

```toml
# Cargo.toml
[profile.release]
lto = true              # 链接时优化
codegen-units = 1       # 单个代码生成单元
panic = "abort"         # abort而不是unwind
opt-level = 3           # 最高优化级别
```

### 2. 内存优化

```rust
// 使用Arc避免不必要的克隆
let shared_data = Arc::new(expensive_data);

// 使用Cow避免不必要的分配
fn process_string(s: Cow<str>) -> String {
    match s {
        Cow::Borrowed(borrowed) => borrowed.to_uppercase(),
        Cow::Owned(owned) => owned.to_uppercase(),
    }
}

// 预分配容量
let mut vec = Vec::with_capacity(expected_size);
```

### 3. 异步优化

```rust
// 并发执行独立任务
let (result1, result2) = tokio::join!(
    async_task1(),
    async_task2()
);

// 使用select处理超时
tokio::select! {
    result = long_running_task() => result,
    _ = tokio::time::sleep(Duration::from_secs(30)) => {
        Err("操作超时")
    }
}
```

## 部署指南

### 1. 本地开发

```bash
# 开发模式
cargo run

# 生产模式编译
cargo build --release

# 运行发布版本
./target/release/maa-server
```

### 2. Docker部署

```dockerfile
# Dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/maa-server /usr/local/bin/
EXPOSE 8080
CMD ["maa-server"]
```

```bash
# 构建镜像
docker build -t maa-intelligent-server .

# 运行容器
docker run -p 8080:8080 maa-intelligent-server
```

### 3. 生产环境配置

```yaml
# docker-compose.yml
version: '3.8'
services:
  maa-server:
    build: .
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
      - PORT=8080
    volumes:
      - ./config:/app/config
      - ./cache:/app/cache
    restart: unless-stopped
```

## 故障排除

### 常见问题

#### 1. 编译错误

```bash
# 清理并重新编译
cargo clean && cargo build

# 更新依赖
cargo update

# 检查Rust版本
rustc --version
```

#### 2. MAA Core问题

```bash
# 检查子模块
git submodule status
git submodule update --init --recursive

# 重新构建MAA Core
cd maa-official && ./build.sh
```

#### 3. 运行时错误

```bash
# 启用详细日志
RUST_LOG=debug cargo run

# 检查依赖
ldd target/release/maa-server

# 检查端口占用
netstat -tulpn | grep 8080
```

## 工具推荐

### 开发工具

- **IDE**: VS Code + rust-analyzer插件
- **调试**: VS Code debugger / GDB
- **性能分析**: cargo flamegraph
- **内存检查**: valgrind (Linux)

### 测试工具

- **单元测试**: cargo test
- **集成测试**: docker-compose
- **压力测试**: k6 / Apache Bench
- **API测试**: Postman / curl

### 监控工具

- **日志**: journalctl / Docker logs
- **性能**: htop / Docker stats
- **网络**: netstat / ss
- **文件**: lsof