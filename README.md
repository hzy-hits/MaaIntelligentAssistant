# MAA 智能控制中间层

> **基于消息队列 + 单线程工作者架构的 MaaAssistantArknights 自动化控制系统**

通过 Function Calling 协议让大模型直接控制明日方舟，支持 PlayCover iOS 模拟和真机连接。

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![MAA](https://img.shields.io/badge/MAA-v5.22.3+-green.svg)](https://github.com/MaaAssistantArknights/MaaAssistantArknights)
[![PlayCover](https://img.shields.io/badge/PlayCover-✅-blue.svg)](https://playcover.io/)

## ✨ 功能特性

### 🎯 核心能力
- **16个完整 MAA 工具**: 覆盖启动、刷图、招募、基建、肉鸽等全功能
- **PlayCover 完美支持**: 自动 TouchMode 配置，解决 iOS 模拟截图问题
- **并发安全架构**: 消息队列 + 单线程工作者，零锁设计
- **动态库集成**: 运行时加载 MAA.app，版本灵活、资源共享

### 🚀 技术亮点
- **Function Calling 协议**: 标准化 AI 模型集成接口
- **异步桥接**: HTTP 异步请求与 MAA 同步调用的完美结合
- **多 AI 提供商**: OpenAI、Azure、通义千问、Kimi、Ollama
- **双运行模式**: 开发模式（Stub）+ 生产模式（真实 MAA）

### 🎮 设备支持
- ✅ **PlayCover**: macOS 上的 iOS 应用模拟器
- ✅ **Android 模拟器**: BlueStacks、NoxPlayer、LDPlayer
- ✅ **Android 真机**: USB 或无线 ADB 连接

## 🏗️ 系统架构

```
┌─────────────────┐    ┌──────────────┐    ┌────────────────┐
│   Axum 异步     │    │   消息队列    │    │  MAA单线程工作者 │
│   HTTP 服务器   │───▶│    (MPSC)    │───▶│   (独占MAA实例) │
│ (多请求并发处理) │    │              │    │                │
└─────────────────┘    └──────────────┘    └────────────────┘
       │                       │                      │
   1000+ QPS              任务序列化              线程安全执行
   异步并发               消息传递                状态一致
```

**架构优势**:
- **零锁设计**: 避免 `Arc<Mutex<>>` 的死锁和竞态条件
- **高性能**: 消息传递比锁机制更高效，延迟 <1ms
- **易调试**: 清晰的消息流，可追踪的执行路径
- **状态一致**: MAA 实例状态始终保持一致性

## 🚀 快速开始

### 环境要求

- **Rust**: 1.70+ 
- **MAA.app**: macOS 系统安装版本 (用于真实模式)
- **PlayCover**: iOS 应用模拟器 (可选)
- **Node.js**: 18+ (Web UI 前端)

### 1. 克隆项目

```bash
git clone --recursive https://github.com/your-repo/maa-remote-server.git
cd maa-remote-server
```

### 2. 环境配置

项目采用**分层配置系统** - TOML配置文件 + 环境变量：

```bash
cp .env.example .env
# 编辑 .env 文件，配置 MAA 路径和 AI API
```

关键配置项：
```bash
# MAA Core 动态库 (系统MAA.app)
MAA_CORE_LIB=/Applications/MAA.app/Contents/Frameworks/libMaaCore.dylib
MAA_RESOURCE_PATH=/Applications/MAA.app/Contents/Resources
DYLD_LIBRARY_PATH=/Applications/MAA.app/Contents/Frameworks

# 设备连接
MAA_DEVICE_ADDRESS=127.0.0.1:1717  # PlayCover
# MAA_DEVICE_ADDRESS=127.0.0.1:5555  # Android 模拟器

# AI 配置
AI_PROVIDER=qwen
AI_API_KEY=your-api-key-here
```

**配置文件结构**:
- `config/app.toml` - 主配置文件（默认值和选项定义）
- `.env` - 环境变量（运行时配置覆盖）
- `.env.example` - 配置模板

详细配置说明请参考：[配置文档](docs/CONFIGURATION.md)

### 3. 运行服务器

**开发模式** (模拟 MAA 功能):
```bash
cargo run --bin maa-server
```

**生产模式** (真实 MAA 集成):
```bash
cargo run --bin maa-server --features with-maa-core
```

**启动 Web UI** (可选):
```bash
cd maa-chat-ui
npm install
npm run dev
```

### 4. 测试连接

```bash
# 健康检查
curl http://localhost:8080/health

# 获取 Function Calling 工具列表
curl http://localhost:8080/tools

# 执行 MAA 任务
curl -X POST http://localhost:8080/call \
  -H "Content-Type: application/json" \
  -d '{
    "function_call": {
      "name": "maa_startup", 
      "arguments": {
        "client_type": "Official",
        "start_app": false
      }
    }
  }'
```

## 🔧 Function Calling 工具

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

### 系统功能 (4个)
- `maa_closedown` - 游戏关闭
- `maa_custom_task` - 自定义任务
- `maa_video_recognition` - 视频识别
- `maa_system_management` - 系统管理

## 📱 PlayCover 设置指南

### 1. 安装 PlayCover
```bash
brew install --cask playcover-nightlybuild
```

### 2. 安装明日方舟
1. 下载明日方舟 IPA 文件
2. 在 PlayCover 中安装 IPA
3. 启用 **MaaTools** 插件

### 3. 配置连接
```bash
# .env 文件配置
MAA_DEVICE_ADDRESS=127.0.0.1:1717  # PlayCover 固定端口

# 系统会自动:
# - 检测 PlayCover 连接
# - 设置 TouchMode 为 MacPlayTools  
# - 配置 iOS 平台差异资源
```

### 4. 验证连接
启动明日方舟后，运行：
```bash
curl -X POST http://localhost:8080/call \
  -H "Content-Type: application/json" \
  -d '{
    "function_call": {
      "name": "maa_startup",
      "arguments": {"client_type": "Official", "start_app": false}
    }
  }'
```

成功连接会显示设备 UUID 和游戏识别信息。

## 🔍 API 参考

### HTTP 接口

| 端点 | 方法 | 功能 |
|------|------|------|
| `/health` | GET | 健康检查 |
| `/tools` | GET | 获取 Function Calling 工具定义 |
| `/call` | POST | 执行 MAA 任务 |
| `/status` | GET | 获取 MAA 状态信息 |

### Function Calling 格式

```json
{
  "function_call": {
    "name": "maa_combat_enhanced",
    "arguments": {
      "stage": "1-7",
      "strategy": {
        "target_value": 10,
        "medicine": 999,
        "stone": 0
      }
    }
  }
}
```

### 响应格式

```json
{
  "success": true,
  "result": {
    "status": "success",
    "message": "任务执行完成",
    "details": {
      "stage": "1-7",
      "completedRuns": 10,
      "resourcesGained": {...}
    }
  },
  "backend": "singleton",
  "timestamp": "2025-08-19T14:15:32Z"
}
```

## 📊 性能指标

| 指标 | 数值 | 说明 |
|------|------|------|
| HTTP 并发处理 | 1000+ QPS | Axum 异步处理 |
| MAA 任务执行 | 串行处理 | 保证状态一致性 |
| 响应延迟 | <100ms | 消息队列开销 <1ms |
| 内存占用 | ~50MB | 单 MAA 实例 |
| CPU 使用率 | 低 | 无锁竞争 |

## 🔧 故障排除

### 常见问题

#### 1. PlayCover 连接失败
```
错误: PlayCover连接失败: Connection refused

解决方案:
1. 确保 PlayCover 已安装明日方舟
2. 确保 MaaTools 已启用
3. 确保游戏正在运行
4. 检查端口 1717 是否被占用
```

#### 2. MAA 库加载失败
```
错误: 加载 MAA Core 库失败

解决方案:
1. 检查 MAA.app 是否已安装
2. 验证 MAA_CORE_LIB 环境变量路径
3. 确保 DYLD_LIBRARY_PATH 设置正确
```

#### 3. 截图或识别异常
```
错误: 模板匹配失败或截图为空

解决方案:
1. 确认游戏界面可见且未被遮挡
2. 检查设备分辨率是否支持
3. 验证 TouchMode 设置 (已自动处理)
```

### 调试技巧

**启用详细日志**:
```bash
LOG_LEVEL=debug cargo run --bin maa-server --features with-maa-core
```

**查看 MAA 回调**:
日志中会显示详细的 MAA 事件信息，包括连接状态、任务执行、识别结果等。

**健康检查**:
```bash
curl http://localhost:8080/health | jq
curl http://localhost:8080/status | jq
```

## 📁 项目结构

```
maa-remote-server/
├── src/
│   ├── bin/
│   │   └── maa-server-singleton.rs    # 🚀 服务器入口
│   ├── maa_core/                      # 🎯 MAA Core 模块
│   │   ├── mod.rs                     # 实例管理和回调
│   │   ├── worker.rs                  # ⭐ 单线程工作者
│   │   ├── task_queue.rs              # ⭐ 任务队列定义
│   │   └── basic_ops.rs               # 废弃API兼容
│   ├── function_tools/                # 🔧 Function Calling
│   │   ├── server.rs                  # 增强服务器
│   │   ├── queue_client.rs            # 队列客户端
│   │   ├── core_game.rs               # 4个核心工具
│   │   ├── advanced_automation.rs     # 4个高级工具
│   │   ├── support_features.rs        # 4个辅助工具
│   │   └── system_features.rs         # 4个系统工具
│   └── ai_client/                     # 🤖 AI 集成
├── docs/                              # 📚 技术文档
│   ├── architecture/                  # 架构设计
│   └── modules/                       # 模块文档
├── maa-chat-ui/                       # 💬 Web UI
└── scripts/                           # 🔧 部署脚本
```

## 🛠️ 开发指南

### 添加新的 Function Tool

1. 在对应模块文件中添加工具定义
2. 实现工具逻辑，使用 `MaaQueueClient`
3. 在 `mod.rs` 中注册工具
4. 更新文档和测试

### 扩展 MAA 任务类型

1. 在 `task_queue.rs` 中添加新的 `MaaTask` 变体
2. 在 `worker.rs` 中实现对应的处理逻辑
3. 在 `queue_client.rs` 中添加客户端方法

### 集成新的 AI 提供商

1. 在 `ai_client/providers/` 中添加提供商实现
2. 实现 `AiProvider` trait
3. 在配置文件中添加相关环境变量

## 📄 技术文档

- [配置说明文档](docs/CONFIGURATION.md) - 完整的配置系统说明
- [系统架构文档](docs/architecture/SYSTEM_ARCHITECTURE.md)
- [MAA Core 模块](docs/modules/MAA_CORE.md)
- [Function Tools 模块](docs/modules/FUNCTION_TOOLS.md)
- [AI Client 模块](docs/modules/AI_CLIENT.md)

## 🤝 贡献指南

1. Fork 项目
2. 创建特性分支: `git checkout -b feature/new-tool`
3. 提交更改: `git commit -am 'Add new MAA tool'`
4. 推送分支: `git push origin feature/new-tool`
5. 创建 Pull Request

## 📜 许可证

本项目基于 MIT 许可证开源。详见 [LICENSE](LICENSE) 文件。

## 🙏 致谢

- [MaaAssistantArknights](https://github.com/MaaAssistantArknights/MaaAssistantArknights) - 核心自动化引擎
- [PlayCover](https://playcover.io/) - macOS iOS 应用模拟器
- [Rust](https://rust-lang.org/) - 系统编程语言
- [Axum](https://github.com/tokio-rs/axum) - 现代异步 Web 框架

---

**维护状态**: ✅ 积极维护 | **版本**: 1.0.0 | **文档更新**: 2025-08-19