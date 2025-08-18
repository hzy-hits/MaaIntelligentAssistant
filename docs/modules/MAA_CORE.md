# MAA Core 模块技术文档

## 模块概述

MAA Core 是 MAA 智能控制系统的底层核心模块，负责直接与 MaaAssistantArknights (MAA) 进行交互。该模块采用 `thread_local!` 单例模式解决了 `maa_sys::Assistant` 不是 `Send` 的线程安全问题，提供了7个基础 MAA 操作和完整的异步接口。

## 架构设计

### 模块结构
```
src/maa_core/
├── mod.rs        # 单例管理和核心结构
└── basic_ops.rs  # 7个基础 MAA 操作函数
```

### 设计原则

1. **单例模式**: 使用 `thread_local!` 确保每个线程独立的 MAA 实例
2. **异步优先**: 所有操作都提供异步接口
3. **错误透明**: 统一的错误处理和传播机制
4. **资源管理**: 自动化的连接和资源生命周期管理

## 核心单例实现 (mod.rs)

### 技术实现

#### Thread Local 单例模式
```rust
// 位置: src/maa_core/mod.rs:25
thread_local! {
    static MAA_CORE: RefCell<Option<MaaCore>> = RefCell::new(None);
}

pub fn with_maa_core<F, R>(f: F) -> Result<R>
where
    F: FnOnce(&mut MaaCore) -> Result<R>,
{
    MAA_CORE.with(|core_ref| {
        let mut core_opt = core_ref.borrow_mut();
        if core_opt.is_none() {
            *core_opt = Some(MaaCore::new());
        }
        let core = core_opt.as_mut().unwrap();
        f(core)
    })
}
```

#### 设计思路
- **线程隔离**: 每个 HTTP 请求在独立线程中处理，MAA 实例互不干扰
- **延迟初始化**: 首次使用时才创建 MAA 实例，避免启动开销
- **生命周期管理**: 实例随线程结束自动清理

### MAA Core 结构定义
```rust
// 位置: src/maa_core/mod.rs:79
pub struct MaaCore {
    #[cfg(feature = "with-maa-core")]
    assistant: Option<maa_sys::Assistant>,
    
    controller: Option<Controller>,
    resource: Option<Resource>,
    connection_id: Option<i32>,
    task_counter: i32,
}
```

#### 字段说明
- `assistant`: MAA 官方绑定的核心对象
- `controller`: 设备控制器（ADB/PlayCover）
- `resource`: MAA 资源管理器
- `connection_id`: 当前连接ID
- `task_counter`: 任务计数器（用于生成唯一ID）

### 初始化流程

#### 开发模式 (Stub)
```rust
// 位置: src/maa_core/mod.rs:99
#[cfg(not(feature = "with-maa-core"))]
impl MaaCore {
    pub fn new() -> Self {
        info!("🚧 创建 MAA Core (Stub模式)");
        Self {
            controller: None,
            resource: None,
            connection_id: None,
            task_counter: 0,
        }
    }
}
```

#### 生产模式 (Real)
```rust
// 位置: src/maa_core/mod.rs:118
#[cfg(feature = "with-maa-core")]
impl MaaCore {
    pub fn new() -> Self {
        info!("🎯 创建真实 MAA Core");
        
        // 初始化资源
        let resource = Resource::new();
        resource.load_resources(&resource_path);
        
        // 初始化控制器
        let controller = Controller::new();
        controller.set_option(ControllerOption::ScreenshotTargetLongSide, 720);
        
        // 创建 Assistant
        let assistant = Assistant::new();
        assistant.bind_controller(&controller);
        assistant.bind_resource(&resource);
        
        Self {
            assistant: Some(assistant),
            controller: Some(controller),
            resource: Some(resource),
            connection_id: None,
            task_counter: 0,
        }
    }
}
```

### 核心操作接口

#### 设备连接
```rust
// 位置: src/maa_core/mod.rs:197
pub fn connect(&mut self, address: &str) -> Result<i32> {
    info!("🔌 连接设备: {}", address);
    
    #[cfg(feature = "with-maa-core")]
    {
        if let Some(controller) = &self.controller {
            let connection_id = controller.post_connection(address)?;
            controller.wait_connection_complete(connection_id)?;
            self.connection_id = Some(connection_id);
            Ok(connection_id)
        } else {
            Err(anyhow!("Controller 未初始化"))
        }
    }
    
    #[cfg(not(feature = "with-maa-core"))]
    {
        let mock_id = 1;
        self.connection_id = Some(mock_id);
        Ok(mock_id)
    }
}
```

#### 任务执行
```rust
// 位置: src/maa_core/mod.rs:225
pub fn execute_task(&mut self, task_type: &str, params: &str) -> Result<i32> {
    self.task_counter += 1;
    let task_id = self.task_counter;
    
    info!("🎮 执行任务: {} (ID: {})", task_type, task_id);
    debug!("任务参数: {}", params);
    
    #[cfg(feature = "with-maa-core")]
    {
        if let Some(assistant) = &self.assistant {
            assistant.post_task(task_type, params)?;
            assistant.wait_task_complete(task_id)?;
            Ok(task_id)
        } else {
            Err(anyhow!("Assistant 未初始化"))
        }
    }
    
    #[cfg(not(feature = "with-maa-core"))]
    {
        // Stub 模式返回模拟 ID
        Ok(task_id)
    }
}
```

## 基础操作实现 (basic_ops.rs)

### 7个核心操作

#### 1. 设备连接 (`connect_device`)
```rust
// 位置: src/maa_core/basic_ops.rs:23
pub fn connect_device(address: &str) -> Result<i32> {
    info!("连接设备: {}", address);
    
    with_maa_core(|core| {
        core.connect(address)
    })
}
```

#### 2. 战斗任务 (`execute_fight`)
```rust
// 位置: src/maa_core/basic_ops.rs:41
pub async fn execute_fight(stage: &str, medicine: i32, stone: i32, times: i32) -> Result<Value> {
    info!("执行刷图任务: {} x {}, medicine={}, stone={}", stage, times, medicine, stone);
    
    // 模拟异步操作
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    
    with_maa_core(|core| {
        let params = json!({
            "stage": stage,
            "medicine": medicine,
            "stone": stone,
            "times": if times > 0 { times } else { 1 }
        });
        
        let params_str = serde_json::to_string(&params)?;
        let task_id = core.execute_task("Fight", &params_str)?;
        
        Ok(json!({
            "task_id": task_id,
            "stage": stage,
            "status": "started"
        }))
    })
}
```

#### 3. 状态查询 (`get_maa_status`)
```rust
// 位置: src/maa_core/basic_ops.rs:79
pub async fn get_maa_status() -> Result<Value> {
    debug!("获取MAA状态");
    
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    
    with_maa_core(|core| {
        let status = core.get_status();
        Ok(json!({
            "maa_status": status,
            "timestamp": Utc::now(),
            "connected": true,
            "running": false
        }))
    })
}
```

#### 4. 招募任务 (`execute_recruit`)
```rust
// 位置: src/maa_core/basic_ops.rs:177
pub async fn execute_recruit(times: i32, expedite: bool, skip_robot: bool) -> Result<Value> {
    info!("执行招募任务: times={}, expedite={}, skip_robot={}", times, expedite, skip_robot);
    
    with_maa_core(|core| {
        let params = json!({
            "enable": true,
            "select": [4, 5, 6],
            "confirm": [3, 4, 5, 6],
            "times": times,
            "expedite": expedite,
            "skip_robot": skip_robot
        });
        
        let task_id = core.execute_task("Recruit", &params_str)?;
        
        Ok(json!({
            "task_id": task_id,
            "times": times,
            "status": "started"
        }))
    })
}
```

#### 5. 基建任务 (`execute_infrastructure`)
```rust
// 位置: src/maa_core/basic_ops.rs:216
pub async fn execute_infrastructure(facility: Value, dorm_trust_enabled: bool, filename: &str) -> Result<Value> {
    info!("执行基建任务: facility={:?}, dorm_trust={}, filename={}", facility, dorm_trust_enabled, filename);
    
    with_maa_core(|core| {
        let params = json!({
            "facility": facility,
            "dorm_trust_enabled": dorm_trust_enabled,
            "filename": filename,
            "plan_index": 0
        });
        
        let task_id = core.execute_task("Infrast", &params_str)?;
        
        Ok(json!({
            "task_id": task_id,
            "facility": facility,
            "status": "started"
        }))
    })
}
```

#### 6. 肉鸽任务 (`execute_roguelike`)
```rust
// 位置: src/maa_core/basic_ops.rs:272
pub async fn execute_roguelike(theme: &str, mode: i32, starts_count: i32) -> Result<Value> {
    info!("执行肉鸽任务: theme={}, mode={}, starts_count={}", theme, mode, starts_count);
    
    with_maa_core(|core| {
        let params = json!({
            "theme": theme,
            "mode": mode,
            "starts_count": starts_count,
            "investment_enabled": true,
            "investments_count": 999
        });
        
        let task_id = core.execute_task("Roguelike", &params_str)?;
        
        Ok(json!({
            "task_id": task_id,
            "theme": theme,
            "status": "started"
        }))
    })
}
```

#### 7. 其他操作
- `execute_copilot` - 作业执行 (位置: basic_ops.rs:311)
- `execute_startup` - 游戏启动 (位置: basic_ops.rs:362)
- `execute_awards` - 奖励收集 (位置: basic_ops.rs:404)

### 异步设计模式

#### 异步包装策略
```rust
// 所有操作都包含短暂的异步延迟，模拟真实操作时间
tokio::time::sleep(std::time::Duration::from_millis(100)).await;

// 然后调用同步的 with_maa_core 函数
with_maa_core(|core| {
    // 执行具体操作
})
```

#### 好处
- 与 Function Tools 层的异步接口保持一致
- 为真实 MAA 操作的异步特性做好准备
- 提供更好的并发处理能力

## 配置管理

### 环境变量支持
```rust
// 位置: src/maa_core/basic_ops.rs:445
#[cfg(feature = "with-maa-core")]
fn find_maa_core_library() -> Result<std::path::PathBuf> {
    // 从环境变量获取
    if let Ok(path) = std::env::var("MAA_CORE_LIB") {
        let path_buf = PathBuf::from(path);
        if path_buf.exists() {
            return Ok(path_buf);
        }
    }
    
    // 已知路径列表 (按平台)
    #[cfg(target_os = "macos")]
    let known_paths = vec![
        "/Applications/MAA.app/Contents/Frameworks/libMaaCore.dylib",
        "/usr/local/lib/libMaaCore.dylib",
    ];
    
    // 搜索逻辑...
}
```

### 支持的环境变量
- `MAA_CORE_LIB`: MAA Core 库文件路径
- `MAA_RESOURCE_PATH`: MAA 资源文件路径
- `MAA_DEVICE_ADDRESS`: 默认设备地址
- `MAA_ADB_PATH`: ADB 可执行文件路径

## 自然语言解析

### 智能刷图命令解析
```rust
// 位置: src/maa_core/basic_ops.rs:159
pub async fn smart_fight(command: &str) -> Result<Value> {
    info!("智能刷图命令: {}", command);
    
    // 解析自然语言命令
    let (stage, times) = parse_fight_command(command)?;
    
    // 执行任务
    let result = execute_fight(&stage, 0, 0, times).await?;
    
    Ok(json!({
        "result": result,
        "stage": stage,
        "command": command,
        "status": "completed"
    }))
}
```

### 解析规则
```rust
// 位置: src/maa_core/basic_ops.rs:476
fn parse_fight_command(command: &str) -> Result<(String, i32)> {
    let cmd_lower = command.to_lowercase();
    
    // 常见关卡映射
    let stage = if cmd_lower.contains("龙门币") || cmd_lower.contains("ce-5") {
        "CE-5"
    } else if cmd_lower.contains("狗粮") || cmd_lower.contains("1-7") {
        "1-7"
    } else if cmd_lower.contains("技能书") || cmd_lower.contains("ca-5") {
        "CA-5"
    } else if cmd_lower.contains("日常") {
        "1-7"  // 日常任务默认刷狗粮
    } else {
        extract_stage_name(command)?
    };
    
    // 解析次数
    let times = if cmd_lower.contains("用完") || cmd_lower.contains("理智") {
        0  // 0表示用完理智
    } else if let Some(times) = extract_number(&cmd_lower) {
        times
    } else {
        1  // 默认1次
    };
    
    Ok((stage.to_string(), times))
}
```

## 错误处理模式

### 统一错误类型
```rust
use anyhow::{Result, anyhow};

// 所有函数返回 Result<T> 其中 Error = anyhow::Error
pub async fn execute_fight(...) -> Result<Value> {
    // 实现
}
```

### 错误传播策略
```rust
// 底层错误
maa_sys::Assistant::post_task(...)?;

// 包装业务错误
.map_err(|e| anyhow!("执行 MAA 任务失败: {}", e))?;

// 最终传播到 Function Tools 层进行用户友好化处理
```

## 平台适配

### 跨平台库路径
```rust
#[cfg(target_os = "macos")]
let known_paths = vec![
    "/Applications/MAA.app/Contents/Frameworks/libMaaCore.dylib",
];

#[cfg(target_os = "linux")]  
let known_paths = vec![
    "/usr/local/lib/libMaaCore.so",
];

#[cfg(target_os = "windows")]
let known_paths = vec![
    "C:\\MAA\\MaaCore.dll",
];
```

### 设备类型支持
```rust
// 位置: src/maa_core/mod.rs:337
fn is_playcover_address(&self, address: &str) -> bool {
    address.starts_with("localhost:") || address.starts_with("127.0.0.1:")
}
```

## 性能优化

### 连接复用
- 单例模式确保每个线程只有一个 MAA 连接
- 避免重复初始化的开销

### 资源管理
```rust
// 延迟加载资源
if core_opt.is_none() {
    *core_opt = Some(MaaCore::new());  // 仅在需要时创建
}
```

### 内存优化
- 使用 `RefCell<Option<T>>` 实现可选的所有权
- 避免不必要的 Clone 操作

## 测试支持

### Stub 模式
```rust
#[cfg(not(feature = "with-maa-core"))]
impl MaaCore {
    pub fn execute_task(&mut self, task_type: &str, params: &str) -> Result<i32> {
        // 返回模拟结果，用于开发和测试
        Ok(self.task_counter)
    }
}
```

### 单元测试
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_fight_command() {
        assert_eq!(parse_fight_command("刷龙门币").unwrap(), ("CE-5".to_string(), 1));
        assert_eq!(parse_fight_command("刷狗粮 10次").unwrap(), ("1-7".to_string(), 10));
        assert_eq!(parse_fight_command("1-7 用完理智").unwrap(), ("1-7".to_string(), 0));
    }
}
```

## 上下游交互

### 上游依赖 (生产模式)
1. **maa_sys**: MAA 官方 Rust 绑定
   - `Assistant` - 核心助手对象
   - `Controller` - 设备控制
   - `Resource` - 资源管理

2. **系统依赖**:
   - ADB (Android Debug Bridge)
   - MAA 资源文件
   - 设备连接 (ADB/PlayCover)

### 下游消费者
1. **function_tools**: 16个 Function Calling 工具
   - 调用7个基础操作函数
   - 组合基础操作实现复杂功能

2. **HTTP API**: 通过 function_tools 间接使用
   - 状态查询接口
   - 任务执行接口

## 部署配置

### 开发环境
```bash
# 使用 Stub 模式，无需 MAA Core
cargo run

# 特性: 快速启动，无外部依赖
```

### 生产环境
```bash
# 编译真实 MAA Core 集成
cargo build --features with-maa-core

# 环境变量配置
export MAA_CORE_LIB=/path/to/libMaaCore.dylib
export MAA_RESOURCE_PATH=/path/to/resource
export MAA_DEVICE_ADDRESS=localhost:1717
```

## 维护指南

### 版本同步
- 保持与 MAA 官方版本同步
- 定期更新 maa_sys 依赖
- 适配新的 MAA API 变化

### 性能监控
```rust
// 添加性能日志
let start_time = std::time::Instant::now();
let result = execute_task(task_type, params);
let duration = start_time.elapsed();
debug!("任务 {} 执行耗时: {:?}", task_type, duration);
```

### 故障排除
1. **连接问题**: 检查设备地址和 ADB 状态
2. **资源问题**: 验证资源文件路径和完整性
3. **权限问题**: 确保库文件和资源文件可访问

## 代码对应关系

| 功能 | 文件位置 | 关键函数/结构 |
|-----|----------|--------------|
| 单例管理 | `src/maa_core/mod.rs:25` | `thread_local! MAA_CORE` |
| 核心结构 | `src/maa_core/mod.rs:79` | `struct MaaCore` |
| 设备连接 | `src/maa_core/basic_ops.rs:23` | `connect_device()` |
| 战斗任务 | `src/maa_core/basic_ops.rs:41` | `execute_fight()` |
| 状态查询 | `src/maa_core/basic_ops.rs:79` | `get_maa_status()` |
| 招募任务 | `src/maa_core/basic_ops.rs:177` | `execute_recruit()` |
| 自然语言解析 | `src/maa_core/basic_ops.rs:476` | `parse_fight_command()` |