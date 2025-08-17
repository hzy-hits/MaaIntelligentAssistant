# MAA Adapter Module

MAA Adapter 模块为 MAA (MaaAssistantArknights) FFI 操作提供了安全、异步、线程安全的 Rust 封装。

## 架构概览

```
maa_adapter/
├── mod.rs          - 模块入口和公共接口导出
├── types.rs        - 数据类型和结构体定义
├── errors.rs       - 统一错误处理系统
├── callbacks.rs    - FFI 回调处理和异步转换
├── core.rs         - 核心 MaaAdapter 实现
└── README.md       - 本文档
```

## 核心特性

### 线程安全
- 使用 `Arc<Mutex<>>` 包装 FFI 句柄
- 所有公共接口都是 `Send + Sync`
- 安全的并发访问控制

### 异步支持
- 完全基于 `tokio` 异步运行时
- FFI 回调通过 `tokio::sync::mpsc` 转换为异步消息
- 支持 `async/await` 语法

### 错误处理
- 统一的 `MaaError` 错误类型
- 结构化错误信息，包含上下文和调试信息
- 可恢复错误的自动重试机制

### 回调管理
- C 风格回调安全转换为 Rust 异步消息
- 支持任务进度、完成、失败等事件
- 统计和监控回调性能

## 使用示例

### 基本使用

```rust
use maa_adapter::{MaaAdapter, MaaAdapterTrait, MaaConfig, MaaTaskType, TaskParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建配置
    let config = MaaConfig {
        resource_path: "./maa-official/resource".to_string(),
        adb_path: "adb".to_string(),
        device_address: "127.0.0.1:5555".to_string(),
        connection_type: "ADB".to_string(),
        timeout_ms: 30000,
        max_retries: 3,
        debug: true,
        ..Default::default()
    };
    
    // 创建适配器
    let mut adapter = MaaAdapter::new(config).await?;
    
    // 连接设备
    adapter.connect("127.0.0.1:5555").await?;
    
    // 截图
    let screenshot = adapter.screenshot().await?;
    println!("Screenshot size: {} bytes", screenshot.len());
    
    // 点击
    adapter.click(100, 200).await?;
    
    // 创建任务
    let task_id = adapter.create_task(
        MaaTaskType::Screenshot,
        TaskParams::default()
    ).await?;
    
    // 启动任务
    adapter.start_task(task_id).await?;
    
    // 查询状态
    let status = adapter.get_status().await?;
    println!("Current status: {:?}", status);
    
    Ok(())
}
```

### 高级使用 - 任务管理

```rust
use maa_adapter::{MaaAdapter, MaaAdapterTrait, MaaTaskType, TaskParams};
use tokio::time::{sleep, Duration};

async fn manage_tasks(adapter: &mut MaaAdapter) -> Result<(), Box<dyn std::error::Error>> {
    // 创建多个任务
    let task1 = adapter.create_task(MaaTaskType::Screenshot, TaskParams::default()).await?;
    let task2 = adapter.create_task(
        MaaTaskType::Click { x: 100, y: 200 },
        TaskParams::default()
    ).await?;
    
    // 按顺序执行任务
    adapter.start_task(task1).await?;
    
    // 等待第一个任务完成
    loop {
        let task = adapter.get_task(task1).await?.unwrap();
        match task.status {
            maa_adapter::MaaStatus::Completed { .. } => break,
            maa_adapter::MaaStatus::Failed { error, .. } => {
                eprintln!("Task failed: {}", error);
                return Err(error.into());
            }
            _ => sleep(Duration::from_millis(100)).await,
        }
    }
    
    // 执行第二个任务
    adapter.start_task(task2).await?;
    
    Ok(())
}
```

### 错误处理

```rust
use maa_adapter::{MaaAdapter, MaaError};

async fn handle_errors(adapter: &mut MaaAdapter) {
    match adapter.connect("invalid_device").await {
        Ok(_) => println!("Connected successfully"),
        Err(MaaError::Connection { message, .. }) => {
            eprintln!("Connection failed: {}", message);
            // 可以尝试重连
        }
        Err(MaaError::Timeout { operation, timeout_ms }) => {
            eprintln!("Operation {} timed out after {}ms", operation, timeout_ms);
        }
        Err(e) => {
            eprintln!("Other error: {}", e);
        }
    }
}
```

## 数据结构

### MaaConfig
配置适配器行为的结构体：
- `resource_path`: MAA 资源文件路径
- `adb_path`: ADB 可执行文件路径
- `device_address`: 设备连接地址
- `connection_type`: 连接类型 (ADB, Win32 等)
- `timeout_ms`: 操作超时时间
- `max_retries`: 最大重试次数

### MaaStatus
表示适配器当前状态：
- `Idle`: 空闲状态
- `Connecting`: 正在连接
- `Connected`: 已连接
- `Running`: 正在执行任务
- `Completed`: 任务完成
- `Failed`: 任务失败
- `Disconnected`: 连接断开

### MaaTaskType
支持的任务类型：
- `Screenshot`: 截图
- `Click { x, y }`: 点击
- `Swipe { from_x, from_y, to_x, to_y, duration }`: 滑动
- `StartFight`: 开始战斗
- `Recruit`: 公招
- `Infrast`: 基建
- `Daily`: 日常任务
- `Custom { task_name, task_params }`: 自定义任务
- `Copilot { stage_name, copilot_data }`: 自动作业

## 测试

运行所有测试：
```bash
cargo test maa_adapter
```

运行特定测试：
```bash
cargo test maa_adapter::core::tests::test_adapter_creation
```

## 实现说明

### 当前状态
这是 MAA Adapter 的第一个版本，当前实现包括：
- ✅ 完整的类型系统和错误处理
- ✅ 异步接口设计
- ✅ 回调处理框架
- ✅ 基础的模拟实现（用于测试）
- ✅ 全面的单元测试覆盖

### 待完成工作
- [ ] 集成真实的 MAA FFI 绑定
- [ ] 完善回调处理的实际逻辑
- [ ] 添加更多 MAA 操作的支持
- [ ] 性能优化和内存管理改进
- [ ] 集成测试和基准测试

### 架构原则
1. **安全第一**: 所有 FFI 操作都通过安全的 Rust 包装
2. **异步友好**: 完全兼容 tokio 生态系统
3. **错误透明**: 提供清晰的错误信息和恢复建议
4. **可测试性**: 模块化设计便于单元测试和集成测试
5. **可扩展性**: 易于添加新的 MAA 操作和功能

## 依赖

主要依赖项：
- `tokio`: 异步运行时
- `tokio-util`: 异步工具
- `async-trait`: 异步 trait 支持
- `thiserror`: 错误处理
- `serde`: 序列化支持
- `chrono`: 时间处理
- `tracing`: 结构化日志

## 贡献

在为此模块贡献代码时，请确保：
1. 所有新功能都有对应的测试
2. 遵循现有的代码风格和注释规范
3. 更新相关文档
4. 确保编译时没有警告