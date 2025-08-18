# MAA 智能控制中间层

基于 Function Calling 协议的 MaaAssistantArknights 自动化控制接口。

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## 功能特性

- **16个 MAA 功能工具**: 覆盖 MaaAssistantArknights 完整操作
- **Function Calling 协议**: 标准化 AI 模型集成接口  
- **三层架构**: HTTP API → Function Tools → MAA Core
- **双运行模式**: 开发模式（stub）+ 生产模式（真实 MAA 集成）
- **多 AI 提供商**: OpenAI、Azure OpenAI、通义千问、Kimi、Ollama
- **Web 聊天界面**: React + Vite 构建的用户界面

## 快速开始

### 环境要求

- Rust 1.70+ 
- Node.js 18+（前端界面）
- MAA.app（真实模式集成）

### 安装部署

```bash
# 安装 Rust 工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆项目（包含子模块）
git clone --recursive <项目地址>
cd maa-remote-server
git submodule update --init --recursive
```

### 配置环境

```bash
./scripts/setup-env.sh
```

脚本将引导配置：
- AI 提供商选择（通义千问、OpenAI 等）
- API 密钥配置
- MAA 运行模式（stub/real）
- 设备连接设置

### 部署方式

#### 本地部署（推荐）

完整部署后端和前端：

```bash
./scripts/deploy-local.sh
```

启动服务：
- 后端服务: http://localhost:8080
- 前端界面: http://localhost:3000

#### Docker 部署（开发环境）

仅开发测试，stub 模式：

```bash
docker-compose up --build -d
```

### 验证部署

```bash
# 健康检查
curl http://localhost:8080/health

# 查看可用工具
curl http://localhost:8080/tools

# 执行功能调用
curl -X POST http://localhost:8080/call \
  -H "Content-Type: application/json" \
  -d '{
    "function_call": {
      "name": "maa_startup",
      "arguments": {"client_type": "Official", "start_app": true}
    }
  }'
```

## 功能工具

### 核心游戏操作
| 工具 | 功能 | 用途 |
|------|------|------|
| `maa_startup` | 游戏启动和账号管理 | 启动游戏、切换账号 |
| `maa_combat_enhanced` | 自动战斗操作 | 刷关卡、资源收集 |
| `maa_recruit_enhanced` | 招募自动化 | 自动公招、标签优化 |
| `maa_infrastructure_enhanced` | 基建管理 | 设施管理、资源收取 |

### 高级自动化
- **集成战略**: `maa_roguelike_enhanced` - 自动肉鸽模式
- **作业操作**: `maa_copilot_enhanced` - 执行战斗策略  
- **保全派驻**: `maa_sss_copilot` - 保全作业操作
- **生息演算**: `maa_reclamation` - 资源回收模式

### 辅助功能  
- **奖励收集**: `maa_rewards_enhanced` - 日常/周常奖励收集
- **信用商店**: `maa_credit_store_enhanced` - 自动商店购买
- **仓库管理**: `maa_depot_management` - 库存管理
- **干员管理**: `maa_operator_box` - 干员数据管理

## 设备支持

### macOS
PlayCover iOS 应用模拟：
```bash
MAA_DEVICE_ADDRESS=localhost:1717
```

### Windows/Linux  
Android 模拟器通过 ADB：
```bash
MAA_DEVICE_ADDRESS=127.0.0.1:5555

# 验证 ADB 连接
adb devices
```

## API 接口

### 端点列表

| 方法 | 路径 | 说明 |
|------|------|------|
| `GET` | `/health` | 服务器健康检查 |
| `GET` | `/tools` | 列出可用功能工具 |
| `POST` | `/call` | 执行功能调用 |
| `POST` | `/chat` | AI 聊天与功能调用 |
| `GET` | `/status` | MAA 状态信息 |

### 功能调用格式

```json
{
  "function_call": {
    "name": "maa_combat_enhanced", 
    "arguments": {
      "stage": "1-7",
      "strategy": {
        "target_value": 10,
        "target_type": "times"
      }
    }
  }
}
```

### 响应格式

```json
{
  "success": true,
  "data": {
    "operation": "combat",
    "stage": "1-7", 
    "result": "completed",
    "statistics": {
      "battles": 10,
      "duration": 300
    }
  }
}
```

## AI 集成配置

### 通义千问（阿里云）

```bash
AI_PROVIDER=qwen
AI_API_KEY=sk-xxx
AI_BASE_URL=https://dashscope.aliyuncs.com/compatible-mode/v1
AI_MODEL=qwen-plus-2025-04-28
```

### OpenAI

```bash
AI_PROVIDER=openai  
AI_API_KEY=sk-xxx
AI_BASE_URL=https://api.openai.com/v1
AI_MODEL=gpt-4-turbo-preview
```

### 本地 Ollama

```bash
AI_PROVIDER=ollama
AI_API_KEY=ollama
AI_BASE_URL=http://localhost:11434/v1  
AI_MODEL=llama2
```

## 故障排除

### 常见问题

**MAA Core 库文件未找到**
```bash
# 检查 MAA 安装
ls -la /Applications/MAA.app/Contents/Frameworks/

# 重新配置环境
source ./setup_maa_env.sh
```

**设备连接失败**
```bash
# PlayCover: 确保明日方舟正在运行
ps aux | grep "Arknights"

# Android: 检查 ADB 连接  
adb devices
```

**AI API 调用失败**
```bash
# 检查 API 密钥配置
echo $AI_API_KEY

# 测试网络连接
curl -H "Authorization: Bearer $AI_API_KEY" $AI_BASE_URL/models
```

### 调试命令

```bash
# 查看详细日志
RUST_LOG=debug ./target/release/maa-server

# 检查端口占用
lsof -i :8080 && lsof -i :3000

# 重建项目
cargo clean && cargo build --release --features with-maa-core
```

## 系统要求

### 硬件要求
- **内存**: 4GB+ RAM
- **CPU**: 2核心以上推荐  
- **存储**: 2GB+ 可用空间

### 软件要求
- **Rust**: 1.70+（推荐 1.75+）
- **Node.js**: 18+（前端界面）
- **MAA**: 最新版 MAA.app（真实模式）
- **平台**: macOS 12+ / Windows 10+ / Linux

### 游戏设备
- **PlayCover**（macOS 推荐）
- **Android 模拟器**（BlueStacks、MuMu、雷电等）
- **真机设备**（通过 ADB 连接）

## 开发指南

### 项目结构

```
src/
├── bin/maa-server-singleton.rs     # 服务器入口点
├── maa_core/                       # MAA Core 单例模块
│   ├── mod.rs                      # thread_local! 单例管理
│   └── basic_ops.rs                # 7个核心 MAA 操作
├── function_tools/                 # Function Calling 工具集
│   ├── core_game.rs                # 核心游戏功能（4个工具）
│   ├── advanced_automation.rs      # 高级自动化（4个工具）  
│   ├── support_features.rs         # 辅助功能（4个工具）
│   ├── system_features.rs          # 系统功能（4个工具）
│   └── server.rs                   # HTTP 服务器和路由
├── ai_client/                      # AI 客户端集成
│   ├── client.rs                   # 多提供商客户端
│   └── config.rs                   # 配置管理
└── maa-chat-ui/                    # React 前端界面
    ├── main.jsx                    # 主聊天组件
    ├── package.json                # 前端依赖
    └── vite.config.js              # Vite 构建配置
```

### 运行模式

| 模式 | 用途 | MAA 集成 | 部署方式 |
|------|------|----------|----------|
| **Stub** | 开发/测试 | 模拟调用 | Docker/本地 |
| **Real** | 生产环境 | 真实 MAA Core | 仅本地 |

### 开发命令

```bash
# 开发模式（热重载）
cargo watch -x 'run --bin maa-server'

# 类型检查
cargo check --features with-maa-core

# 运行测试
cargo test

# 查看日志
tail -f logs/maa-server.log

# 前端开发
cd maa-chat-ui && npm run dev
```

## 贡献指南

欢迎通过 Issue 和 Pull Request 贡献代码。

### 开发环境设置

1. Fork 项目
2. 创建功能分支: `git checkout -b feature/功能名称`  
3. 提交更改: `git commit -m '添加功能'`
4. 推送分支: `git push origin feature/功能名称`
5. 创建 Pull Request

### 代码规范

- 格式化代码: `cargo fmt`
- 检查警告: `cargo clippy`
- 为新功能添加单元测试
- 更新相关文档

## 开源协议

本项目采用 MIT 协议 - 详见 [LICENSE](LICENSE) 文件。

## 致谢

- [MAA Team](https://github.com/MaaAssistantArknights) - MAA 自动化框架
- [明日方舟](https://ak.hypergryph.com/) - 鹰角网络开发的游戏  
- Rust 社区 - 优秀的工具链和生态系统

---

## 快速上手

1. **安装 Rust**: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. **配置环境**: `./scripts/setup-env.sh` 
3. **一键部署**: `./scripts/deploy-local.sh`
4. **测试接口**: `curl http://localhost:8080/health`

访问 http://localhost:3000 使用 Web 聊天界面进行 MAA 控制。