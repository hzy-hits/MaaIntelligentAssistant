# MAA 智能控制中间层

基于 Function Calling 协议的明日方舟自动化控制系统。通过 Rust + MAA Core 集成，提供 HTTP API 和 WebUI 界面，支持通过自然语言或 API 调用控制 MAA 执行游戏任务。

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![MAA](https://img.shields.io/badge/MAA-v5.22.3+-green.svg)](https://github.com/MaaAssistantArknights/MaaAssistantArknights)
[![PlayCover](https://img.shields.io/badge/PlayCover-supported-blue.svg)](https://playcover.io/)

## 技术架构

### 核心设计原则

本项目采用消息队列 + 单线程工作者架构，避免传统多线程同步问题：

```
┌─────────────────────┐    ┌──────────────────┐    ┌─────────────────────┐
│   HTTP 服务器       │    │   任务队列       │    │   MAA 工作线程      │
│   (Axum 异步)       │───▶│   (MPSC 通道)    │───▶│   (单线程 MAA 实例) │
│ - 并发请求处理      │    │ - 消息传递       │    │ - thread_local!     │
│ - Function Calling  │    │ - 任务分发       │    │ - 状态管理          │
└─────────────────────┘    └──────────────────┘    └─────────────────────┘
```

### 架构优势

1. **并发安全**: HTTP 层支持高并发，MAA 层保证线程安全
2. **状态一致**: MAA Assistant 实例始终在单线程中执行，无状态竞争
3. **性能优化**: 消息传递避免锁竞争，异步处理提升响应速度
4. **易于调试**: 清晰的数据流向，可追踪的执行路径

### V2 优化架构

最新的 V2 版本进一步优化了架构设计：

```
┌─────────────────────┐    ┌──────────────────┐    ┌─────────────────────┐
│   HTTP API          │    │   Enhanced Tools │    │   Task Queue V2     │
│   (端口 8080)       │───▶│   V2             │───▶│   (单队列+优先级)   │
└─────────────────────┘    └──────────────────┘    └─────────────────────┘
          │                          │                          │
          ▼                          ▼                          ▼
┌─────────────────────┐    ┌──────────────────┐    ┌─────────────────────┐
│   AI Chat API       │    │   SSE 实时更新   │    │   MAA Worker V2     │
│   (智能对话)        │    │   (任务进度)     │    │   (内部状态管理)    │
└─────────────────────┘    └──────────────────┘    └─────────────────────┘
```

**V2 改进特性**:
- 单队列 + 任务优先级系统
- Server-Sent Events (SSE) 实时任务更新
- 同步/异步任务智能分离
- Worker 内部状态管理
- 减少 JSON 序列化开销

## Function Calling 工具集

系统提供 20 个 MAA 功能工具，按用途分类：

### 核心游戏功能 (4个)
- `maa_startup` - 游戏启动和账号管理
- `maa_combat_enhanced` - 自动战斗和资源管理
- `maa_recruit_enhanced` - 智能公开招募
- `maa_infrastructure_enhanced` - 基建自动化

### 高级自动化 (4个)
- `maa_roguelike_enhanced` - 集成战略(肉鸽)
- `maa_copilot_enhanced` - 作业执行
- `maa_sss_copilot` - 保全派驻
- `maa_reclamation` - 生息演算

### 辅助功能 (4个)
- `maa_rewards_enhanced` - 奖励收集
- `maa_credit_store_enhanced` - 信用商店
- `maa_depot_management` - 仓库管理
- `maa_operator_box` - 干员管理

### 系统功能 (8个)
- `maa_closedown` - 游戏关闭
- `maa_custom_task` - 自定义任务
- `maa_video_recognition` - 视频识别
- `maa_system_management` - 系统管理
- `maa_take_screenshot` - 截图功能
- `maa_get_task_list` - 获取任务列表
- `maa_adjust_task_params` - 动态调整任务参数
- `maa_emergency_home` - 紧急返回主界面

## 快速开始

### 环境要求

**必需组件**:
- Rust 1.70+
- MAA.app (macOS 系统安装版本，用于真实模式)

**可选组件**:
- PlayCover (iOS 应用模拟器)
- Android 模拟器或真机
- Node.js 18+ (前端开发)

### 1. 项目设置

```bash
git clone --recursive https://github.com/your-repo/maa-remote-server.git
cd maa-remote-server
```

### 2. 环境配置

创建配置文件:
```bash
cp .env.example .env
```

编辑 `.env` 文件，设置关键配置项：

```bash
# MAA Core 路径 (macOS)
MAA_CORE_LIB=/Applications/MAA.app/Contents/Frameworks/libMaaCore.dylib
MAA_RESOURCE_PATH=/Applications/MAA.app/Contents/Resources
DYLD_LIBRARY_PATH=/Applications/MAA.app/Contents/Frameworks

# 设备连接配置
MAA_DEVICE_ADDRESS=127.0.0.1:1717  # PlayCover
# MAA_DEVICE_ADDRESS=127.0.0.1:5555  # Android 模拟器

# AI 配置 (用于聊天接口)
AI_PROVIDER=qwen
AI_API_KEY=your-api-key-here
AI_BASE_URL=https://dashscope.aliyuncs.com/compatible-mode/v1
```

### 3. 启动服务

**开发模式** (模拟 MAA 功能):
```bash
cargo run --bin maa-optimized-server --no-default-features --features stub-mode
```

**生产模式** (真实 MAA Core):
```bash
cargo run --bin maa-optimized-server
```

### 4. API 测试

基本功能测试:
```bash
# 健康检查
curl http://localhost:8080/health

# 获取工具列表
curl http://localhost:8080/tools

# 执行截图任务 (同步)
curl -X POST http://localhost:8080/call \
  -H "Content-Type: application/json" \
  -d '{
    "function_call": {
      "name": "maa_take_screenshot",
      "arguments": {"format": "png"}
    }
  }'

# 执行战斗任务 (异步)
curl -X POST http://localhost:8080/call \
  -H "Content-Type: application/json" \
  -d '{
    "function_call": {
      "name": "maa_combat_enhanced", 
      "arguments": {"stage": "1-7", "times": 5}
    }
  }'
```

### 5. Web UI (可选)

```bash
cd maa-chat-ui
npm install
npm run dev
# 访问 http://localhost:3000
```

## API 接口

### HTTP 端点

| 端点 | 方法 | 功能 | 类型 |
|------|------|------|------|
| `/health` | GET | 健康检查 | 系统状态 |
| `/tools` | GET | 获取工具定义 | 开发调试 |
| `/call` | POST | 直接执行工具 | Function Calling |
| `/chat` | POST | 智能对话接口 | AI 集成 |
| `/status` | GET | MAA 状态查询 | 状态监控 |
| `/sse/tasks` | GET | SSE 任务流 | 实时更新 |
| `/optimization/stats` | GET | 性能统计 | 系统监控 |

### Function Calling 格式

**请求格式**:
```json
{
  "function_call": {
    "name": "maa_combat_enhanced",
    "arguments": {
      "stage": "1-7",
      "times": 10,
      "use_medicine": true,
      "use_stone": false
    }
  }
}
```

**响应格式**:
```json
{
  "success": true,
  "backend": "optimized-v2",
  "execution_mode": "asynchronous",
  "result": {
    "task_id": 123,
    "status": "running",
    "check_status_url": "/task/123/status"
  },
  "sse_info": {
    "message": "异步任务已启动，进度将通过SSE推送",
    "sse_endpoint": "/sse/tasks"
  },
  "timestamp": "2025-08-22T13:51:25Z"
}
```

### 任务执行模式

系统支持两种执行模式：

**同步任务** (立即返回结果):
- `maa_take_screenshot`
- `maa_startup`
- `maa_closedown`

**异步任务** (后台执行 + SSE 推送):
- `maa_combat_enhanced`
- `maa_infrastructure_enhanced`
- `maa_roguelike_enhanced`
- 其他所有游戏操作任务

### Server-Sent Events (SSE)

V2 架构支持实时任务进度更新：

```bash
# 监听所有任务进度
curl -N -H "Accept: text/event-stream" http://localhost:8080/sse/tasks

# 示例输出
event: heartbeat
data: {"message":"连接正常","timestamp":"2025-08-22 13:43:12 UTC"}

event: task_progress
data: {"task_id":123,"status":"running","progress":45,"message":"正在执行关卡 1-7"}

event: task_completed
data: {"task_id":123,"status":"completed","result":{"stage":"1-7","runs":10}}
```

## 设备支持

### PlayCover (推荐)

**安装配置**:
```bash
# 安装 PlayCover
brew install --cask playcover-nightlybuild

# 安装明日方舟 IPA
# 在 PlayCover 中启用 MaaTools 插件
```

**连接配置**:
```bash
MAA_DEVICE_ADDRESS=127.0.0.1:1717
```

系统会自动：
- 检测 PlayCover 连接
- 设置 TouchMode 为 MacPlayTools
- 配置 iOS 平台资源差异

### Android 设备

**模拟器**:
```bash
MAA_DEVICE_ADDRESS=127.0.0.1:5555  # 标准 ADB 端口
```

**真机**:
```bash
# USB 连接
adb devices
MAA_DEVICE_ADDRESS=<device_ip>:5555

# 无线连接
adb connect <device_ip>:5555
```

## 项目结构

```
maa-remote-server/
├── src/
│   ├── bin/
│   │   └── maa-optimized-server.rs      # V2 优化服务器入口
│   ├── maa_core/                        # MAA Core 模块
│   │   ├── mod.rs                       # MAA 实例管理
│   │   ├── basic_ops.rs                 # MAA 基础操作
│   │   ├── worker_v2.rs                 # V2 工作线程
│   │   ├── task_queue_v2.rs             # V2 任务队列
│   │   └── task_classification_v2.rs    # 任务分类系统
│   ├── function_tools/                  # Function Calling 工具集
│   │   ├── handler_v2.rs                # V2 工具处理器
│   │   ├── core_game.rs                 # 核心游戏功能
│   │   ├── advanced_automation.rs       # 高级自动化
│   │   ├── support_features.rs          # 辅助功能
│   │   └── system_features.rs           # 系统功能
│   ├── sse/                             # Server-Sent Events
│   │   └── mod.rs                       # SSE 实时更新模块
│   └── ai_client/                       # AI 客户端集成
├── maa-chat-ui/                         # React Web UI
├── docs/                                # 技术文档
├── deprecated/                          # 已废弃的旧版代码
└── scripts/                             # 部署脚本
```

## 开发指南

### 添加新的 Function Tool

1. **选择合适的模块文件** (core_game.rs / advanced_automation.rs / support_features.rs / system_features.rs)

2. **实现工具定义函数**:
```rust
pub fn create_new_tool_definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "maa_new_tool".to_string(),
        description: "工具功能描述".to_string(),
        parameters: // JSON Schema 定义参数
    }
}
```

3. **在 handler_v2.rs 中添加工具处理逻辑**

4. **更新 get_function_definitions() 方法**

### 扩展 MAA 任务类型

1. **在 task_classification_v2.rs 中添加任务分类**
2. **在 worker_v2.rs 中实现具体的 MAA Core 调用**
3. **更新任务队列优先级策略**

### 性能优化要点

1. **避免频繁的 JSON 序列化**: V2 架构直接传递参数
2. **合理设置任务优先级**: 同步任务优先于异步任务
3. **利用 SSE 减少轮询**: 实时推送替代 HTTP 轮询
4. **MAA Core 连接复用**: 单实例避免重复连接开销

## 故障排除

### 常见错误解决

**MAA Core 连接失败**:
```bash
# 检查 MAA 库文件
ls -la /Applications/MAA.app/Contents/Frameworks/libMaaCore.dylib

# 验证资源路径
ls -la /Applications/MAA.app/Contents/Resources/resource

# 检查环境变量
echo $DYLD_LIBRARY_PATH
```

**PlayCover 连接问题**:
```bash
# 检查端口监听
netstat -an | grep 1717

# 验证 MaaTools 插件
# 在 PlayCover 设置中确认 MaaTools 已启用
```

**截图或识别异常**:
- 确认游戏界面可见且未被遮挡
- 检查设备分辨率支持 (系统会自动处理 TouchMode)
- 验证 MAA 资源文件完整性

### 调试模式

**启用详细日志**:
```bash
RUST_LOG=debug cargo run --bin maa-optimized-server
```

**实时监控 MAA 事件**:
服务器日志会显示详细的 MAA 回调信息，包括：
- 连接状态和设备识别
- 任务执行进度和结果
- 图像识别和模板匹配得分
- 错误信息和异常处理

## 技术文档

### 核心文档
- [系统架构文档](docs/architecture/SYSTEM_ARCHITECTURE.md)
- [MAA Core 模块文档](docs/modules/MAA_CORE.md)
- [Function Tools 模块文档](docs/modules/FUNCTION_TOOLS.md)

### 深度技术分析
- [MAA 任务系统研究](docs/maa-research/TASK_SYSTEM.md)
- [图像识别算法分析](docs/maa-research/IMAGE_RECOGNITION.md)
- [基建调度算法研究](docs/maa-research/INFRAST_SCHEDULING.md)
- [战斗策略系统分析](docs/maa-research/BATTLE_STRATEGY.md)
- [FFI 集成技术文档](docs/maa-research/FFI_INTEGRATION.md)

## 贡献指南

1. Fork 项目仓库
2. 创建功能分支: `git checkout -b feature/new-feature`
3. 提交代码更改: `git commit -am 'Add new feature'`
4. 推送到分支: `git push origin feature/new-feature`
5. 创建 Pull Request

### 代码规范

- 遵循 Rust 官方代码风格
- 添加必要的文档注释
- 编写相应的单元测试
- 更新相关技术文档

## 许可证

本项目基于 MIT 许可证开源。详见 [LICENSE](LICENSE) 文件。

## 致谢

- [MaaAssistantArknights](https://github.com/MaaAssistantArknights/MaaAssistantArknights) - MAA 自动化引擎
- [PlayCover](https://playcover.io/) - macOS iOS 应用模拟器
- [Rust](https://rust-lang.org/) - 系统编程语言
- [Axum](https://github.com/tokio-rs/axum) - 异步 Web 框架

---

**项目状态**: 积极维护 | **版本**: 2.0.0 | **最后更新**: 2025-08-22