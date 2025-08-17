# MAA 智能控制中间层 - AI玩明日方舟

## 项目核心理念
**让大模型通过自然语言智能控制 MaaAssistantArknights，实现真正的 "AI 玩明日方舟"**

## 项目识别
```yaml
project:
  type: ai-game-automation-middleware
  domain: intelligent-game-control
  target: arknights-automation
  language: rust + react
  protocols: [function-calling, http, ffi]
  status: production-ready
  purpose: "大模型 ↔ MAA引擎 智能桥接"
```

## 三层系统架构

### 核心设计哲学
通过标准 Function Calling 协议，实现 AI 大模型与游戏自动化引擎的无缝集成。

### 三层架构详解

#### 1. 前端层 - maa-chat-ui
**智能对话界面** (http://localhost:3000)
- **技术栈**: React 19 + Vite 5 + CSS Variables
- **功能**: 自然语言聊天界面，用户通过对话控制游戏
- **特色**: 
  - MAA官方徽标集成
  - 深色/浅色主题支持
  - 实时后端连接状态显示
  - 直接调用 Qwen API 进行智能对话

#### 2. 中间件层 - maa-intelligent-server  
**Function Calling API 服务器** (http://localhost:8080)
- **技术栈**: Rust + Axum + Tokio + Sled
- **核心价值**: 将大模型的自然语言意图转换为MAA可执行操作
- **关键模块**:
  ```
  src/
  ├── function_calling_server.rs  # HTTP API服务器
  ├── maa_adapter/               # MAA FFI安全封装
  │   ├── core.rs                # 核心适配器逻辑
  │   ├── ffi_bindings.rs        # FFI安全包装(支持stub模式)
  │   └── types.rs               # MAA数据类型定义
  ├── mcp_tools/                 # 4个核心Function Calling工具
  │   ├── maa_status.rs          # 系统状态查询
  │   ├── maa_command.rs         # 自然语言命令执行
  │   ├── maa_operators.rs       # 干员数据管理
  │   └── maa_copilot.rs         # 智能作业匹配
  ├── operator_manager/          # 干员扫描与缓存
  └── copilot_matcher/           # 三阶段作业匹配引擎
  ```

#### 3. 引擎层 - maa-official (git submodule)
**MAA自动化引擎** 
- **技术栈**: C++ MaaCore + 多语言绑定(Rust/Python/Java/Golang等)
- **功能**: 图像识别、设备控制、任务执行、战斗逻辑
- **规模**: 40000+ 代码文件，包含完整的明日方舟自动化解决方案
- **关键组件**:
  ```
  maa-official/src/
  ├── MaaCore/              # C++核心引擎
  │   ├── Vision/           # 图像识别模块
  │   ├── Controller/       # 设备控制模块  
  │   ├── Task/            # 任务执行模块
  │   └── Config/          # 配置管理模块
  └── Rust/                # Rust HTTP服务器绑定
      ├── src/             # 我们集成的入口点
      └── Cargo.toml       # 依赖配置
  ```

## 智能控制数据流

### 核心工作流程
```
用户自然语言输入 → 前端聊天界面(3000) 
    ↓
Qwen API 大模型理解意图 → 生成 Function Calling
    ↓  
后端API服务器(8080) → 路由到对应MCP工具
    ↓
MAA适配器 → MAA引擎 → 实际游戏操作
    ↓
执行结果 → 返回用户界面
```

### 示例交互流程
```
用户: "帮我做今天的日常任务"
  ↓
AI: function_call(name="maa_command", args={"command":"帮我做日常"})
  ↓  
HTTP POST /call → mcp_tools::maa_command::execute()
  ↓
maa_adapter.start_task("LinkStart") → MAA引擎执行
  ↓
返回: {"status":"success", "message":"日常任务已开始执行"}
```

## 技术栈详解

### 后端技术栈
```toml
# HTTP服务器
axum = "0.8.4"                    # 现代化异步Web框架
tokio = "1.0"                     # 异步运行时
tower-http = "0.6.6"             # HTTP中间件和CORS

# 数据存储  
sled = "0.34"                     # 嵌入式数据库

# AI集成
async-openai = "0.27"             # OpenAI API客户端
reqwest = "0.11"                  # HTTP客户端

# 序列化
serde = "1.0"                     # JSON序列化
bincode = "2.0.1"                 # 二进制序列化

# 错误处理和日志
anyhow = "1.0"                    # 错误处理
tracing = "0.1"                   # 结构化日志
```

### 前端技术栈
```json
{
  "react": "^19.0.0",
  "vite": "^5.4.19", 
  "dependencies": {
    "聊天界面": "自研React组件",
    "AI集成": "直接调用Qwen API",
    "主题系统": "CSS Variables深色/浅色模式"
  }
}
```

### 系统要求
- **平台支持**: macOS (主要), Linux, Windows  
- **开发环境**: Rust 1.70+, Node.js 18+
- **运行模式**: 
  - **开发模式**: stub模式，模拟MAA功能
  - **生产模式**: 真实MAA引擎集成 (需要 `--features with-maa-core`)

## API 规范

### HTTP 端点
```http
# 核心API
GET  /health         # 系统健康检查
GET  /tools          # 获取Function Calling工具定义
POST /call           # 执行Function Calling

# API版本化路径  
GET  /api/health     # 版本化健康检查
GET  /api/tools      # 版本化工具列表
POST /api/call       # 版本化函数调用

# 静态文件服务(生产模式)
GET  /*              # 前端静态文件服务
```

## 四个核心 Function Calling 工具

### 1. maa_status - 系统状态查询
```typescript
{
  name: "maa_status",
  description: "获取MAA当前状态、设备信息和活动任务",
  parameters: {
    verbose: {
      type: "boolean",
      default: false,
      description: "是否返回详细信息，包括设备信息和活动任务"
    }
  }
}
```
**返回示例**: 
```json
{
  "status": "Idle|Running|Connecting",
  "device": "127.0.0.1:5555", 
  "tasks": ["当前执行的任务列表"],
  "message": "MAA状态获取成功"
}
```

### 2. maa_command - 自然语言命令执行  
```typescript
{
  name: "maa_command", 
  description: "使用自然语言执行MAA命令，如'帮我做日常'、'截图'、'刷1-7'等",
  parameters: {
    command: {
      type: "string",
      description: "自然语言命令，例如：'帮我做日常'、'截图'、'刷1-7关卡'、'基建收菜'"
    },
    context: {
      type: "string", 
      description: "可选的上下文信息，用于更好地理解命令"
    }
  }
}
```
**支持的命令类型**:
- `"帮我做日常"` → 启动 LinkStart 一键日常
- `"截图"` → 执行屏幕截图  
- `"刷1-7"` → 刷指定关卡
- `"基建收菜"` → 基建设施管理

### 3. maa_operators - 干员数据管理
```typescript
{
  name: "maa_operators",
  description: "查询和管理明日方舟干员信息", 
  parameters: {
    query_type: {
      type: "string",
      enum: ["list", "search"],
      description: "查询类型：list（列出所有）或 search（搜索）"
    },
    query: {
      type: "string",
      description: "搜索关键词（当query_type为search时使用）"
    }
  }
}
```
**功能**:
- 基于MAA图像识别扫描干员数据
- sled数据库缓存和查询
- 增量更新和同步功能

### 4. maa_copilot - 智能作业匹配
```typescript
{
  name: "maa_copilot",
  description: "执行MAA作业（自动战斗脚本）",
  parameters: {
    copilot_config: {
      type: "object", 
      description: "作业配置JSON，包含关卡信息和编队配置"
    },
    name: {
      type: "string",
      description: "作业名称（可选）"
    }
  }
}
```
**三阶段智能匹配**:
- **Simple**: 基础干员存在性检查
- **Level**: 等级和精英化验证  
- **Smart**: 综合练度分析和智能替换推荐

## 项目结构

### 完整目录结构
```
maa-remote-server/                    # 项目根目录
├── src/                             # Rust后端源码
│   ├── main.rs                      # 应用程序入口点
│   ├── function_calling_server.rs   # HTTP API服务器实现
│   ├── maa_adapter/                 # MAA引擎FFI安全封装
│   │   ├── core.rs                  # 核心适配器逻辑
│   │   ├── ffi_bindings.rs          # FFI函数安全包装
│   │   ├── ffi_stub.rs             # 开发测试Stub模式
│   │   ├── types.rs                 # MAA数据类型定义
│   │   └── errors.rs                # 错误处理
│   ├── mcp_tools/                   # Function Calling工具集
│   │   ├── function_calling.rs      # 工具注册和管理
│   │   ├── maa_status.rs           # 状态查询工具
│   │   ├── maa_command.rs          # 命令执行工具  
│   │   ├── maa_operators.rs        # 干员管理工具
│   │   └── maa_copilot.rs          # 作业匹配工具
│   ├── operator_manager/            # 干员数据管理
│   │   ├── scanner.rs              # 干员扫描器
│   │   ├── cache.rs                # 数据缓存管理
│   │   └── types.rs                # 干员数据类型
│   ├── copilot_matcher/             # 作业匹配引擎
│   │   ├── matcher.rs              # 三阶段匹配算法
│   │   ├── api_client.rs           # 作业站API客户端
│   │   └── cache.rs                # 作业缓存管理
│   └── ai_client/                   # AI提供商集成
│       ├── client.rs               # 统一AI客户端
│       └── providers/              # 多提供商支持
├── maa-chat-ui/                     # React前端
│   ├── index.html                  # 主界面HTML
│   ├── main.jsx                    # React应用入口
│   ├── package.json                # 前端依赖配置
│   ├── vite.config.js              # Vite构建配置
│   └── public/assets/              # 静态资源(MAA徽标等)
├── maa-official/                    # MAA引擎子模块
│   ├── src/MaaCore/                # C++核心引擎
│   ├── src/Rust/                   # Rust绑定
│   └── resource/                   # MAA资源文件
├── scripts/                         # 部署脚本
│   ├── start-all.sh                # 生产环境启动
│   └── dev.sh                      # 开发环境启动
├── Dockerfile                       # Docker容器配置
├── docker-compose.yml              # Docker编排配置
├── Cargo.toml                      # Rust项目配置
└── CLAUDE.md                       # 项目知识库(本文件)
```

## 开发状态

### 已完成模块 (Production Ready)
- **MAA适配器** - FFI集成完成，支持stub/生产双模式
- **Function Calling服务器** - HTTP API服务器，支持CORS
- **4个核心工具** - 完整的Function Calling工具集
- **干员管理器** - 数据扫描和sled缓存系统  
- **作业匹配器** - 三阶段智能匹配引擎
- **前端界面** - React 19 + Vite 5聊天界面
- **Docker支持** - 多阶段构建和容器化部署

### 当前运行状态 (2025-08-17)
- **后端服务器**: http://localhost:8080 运行中
- **前端开发服务器**: http://localhost:3000 运行中  
- **运行模式**: stub模式 (MAA功能模拟)
- **健康检查**: http://localhost:8080/health 正常
- **API文档**: http://localhost:8080/tools 可访问

### 技术债务
- rmcp集成需要使用官方SDK重构 (暂时忽略)
- ~~部分编译警告需要清理~~ (已完成)
- ~~前端可能有未使用的依赖项~~ (已检查，依赖简洁)

## 明日方舟游戏领域知识

### 目标应用
**MaaAssistantArknights** - 明日方舟自动化操作框架

### 常见游戏操作
- **日常任务自动化**: 一键完成每日任务和签到
- **资源刷取**: 1-7(狗粮), CE-5(龙门币), CA-5(技能书), AP-5(红票)  
- **干员招募管理**: 公开招募和寻访抽取
- **基建设施管理**: 基建收菜、换班、无人机加速

### 游戏专业术语
- **关卡**: 1-7(经验书刷取), CE-5(龙门币本), H12-4(高难度关卡)
- **知名干员**: 陈、银灰、史尔特尔、山、棘刺、凯尔希等
- **游戏资源**: 至纯源石、合成玉、龙门币、经验卡、技能概要
- **作业系统**: 自动战斗脚本，由社区分享的通关策略

### AI智能决策支持
- 根据用户干员配置推荐最优作业
- 分析干员练度和关卡要求匹配度
- 提供资源获取和干员培养建议

## AI助手集成说明

### Function Calling 使用指南
AI助手与本系统交互时应该:
1. **使用标准格式**: 所有MAA操作使用Function Calling协议
2. **理解游戏术语**: 正确理解明日方舟的游戏机制和专业术语  
3. **智能上下文管理**: 提供游戏状态的自然语言解释
4. **策略建议**: 根据用户资源提供最优的自动化策略

### 错误处理机制
- **Stub模式**: MAA Core不可用时使用模拟功能
- **详细错误信息**: HTTP响应包含完整的错误诊断
- **健康状态监控**: 实时检查系统连接状态

## 开发工作流程

### 本地开发环境
```bash
# 快速启动开发环境
./scripts/dev.sh

# 手动分别启动
cargo run                    # 后端服务器 (8080)
cd maa-chat-ui && npm run dev # 前端服务器 (3000)

# Stub模式编译 (默认)
cargo build

# 生产模式编译 (需要MAA Core)
cargo build --features with-maa-core
```

### 生产环境部署
```bash
# Docker容器化部署
docker-compose up

# 直接部署
./scripts/start-all.sh

# 手动Docker构建
docker build -t maa-intelligent-server .
docker run -p 8080:8080 maa-intelligent-server
```

### 功能测试
```bash
# 系统健康检查
curl http://localhost:8080/health

# 获取工具列表
curl http://localhost:8080/tools

# 测试Function Calling
curl -X POST http://localhost:8080/call \
  -H "Content-Type: application/json" \
  -d '{
    "function_call": {
      "name": "maa_status",
      "arguments": {"verbose": false}
    }
  }'

# 测试自然语言命令
curl -X POST http://localhost:8080/call \
  -H "Content-Type: application/json" \
  -d '{
    "function_call": {
      "name": "maa_command", 
      "arguments": {"command": "帮我做日常"}
    }
  }'
```

### 调试和监控
```bash
# 查看详细日志
RUST_LOG=debug cargo run

# 查看特定模块日志  
RUST_LOG=maa_intelligent_server::maa_adapter=trace cargo run

# 检查端口占用
lsof -i :8080
lsof -i :3000
```

---

## 文档编写规则

### 严格禁止项
1. **表情符号**: 任何形式的表情符号都不得出现在技术文档中
   - 禁止: 火箭、勾选、警告、工具等所有emoji符号
   - 替代: 使用简洁的文字描述或标准标点符号

2. **营销话语**: 避免使用煽动性、宣传性语言
   - 禁止: "令人兴奋的"、"革命性的"、"颠覆性的"、"震撼"等
   - 替代: 使用客观、准确的技术描述

3. **夸大表述**: 避免不实的技术声明
   - 禁止: "完美的"、"最佳的"、"史上最强"、"无与伦比"等
   - 替代: 使用具体的技术指标和客观评价

### 推荐写法
- 使用清晰、简洁的技术术语
- 提供具体的代码示例和配置
- 包含准确的版本号和技术规格
- 描述实际功能而非理想愿景

### 文档维护
任何对此文档的修改都必须遵循上述规则。如发现违反规则的内容，应立即修正。
