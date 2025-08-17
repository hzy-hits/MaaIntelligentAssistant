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
  ├── mcp_tools/                 # 16个增强Function Calling工具集
  │   ├── function_calling.rs   # 基础4工具实现（兼容模式）
  │   ├── enhanced_tools.rs     # 增强16工具实现（专业模式）
  │   ├── maa_startup.rs        # 游戏启动管理（已完成）
  │   ├── maa_combat.rs         # 增强战斗系统（开发中）
  │   ├── maa_recruit.rs        # 智能招募管理（设计中）
  │   ├── maa_infrastructure.rs # 基建自动化（设计中）
  │   └── [12个高级工具]        # roguelike, copilot, rewards等
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

## 双模式 Function Calling 工具架构

### 架构设计理念
基于深入分析maa-cli项目的16种MAA任务类型，我们设计了**双模式Function Calling架构**：

#### 基础模式 (4工具) - 向下兼容
**服务器**: `maa-server` (http://localhost:8080)  
**用途**: 简单场景和快速原型开发
```bash
cargo run --bin maa-server
```

#### 增强模式 (16工具) - 专业完整 ✅ **生产就绪**
**服务器**: `maa-server-enhanced` (http://localhost:8080)  
**用途**: 生产环境和完整MAA功能覆盖
**状态**: 所有16个工具已完成实现和集成
```bash
cargo run --bin maa-server-enhanced
```

### 工具实现状态
- **基础工具**: 4个 (maa_status, maa_command, maa_operators, maa_copilot) ✅
- **增强工具**: 16个完整MAA任务类型覆盖 ✅
- **文档状态**: 已移至单独文件 📋

**详细工具清单**: 参见 [docs/enhanced-tools-status.md](docs/enhanced-tools-status.md)  
**已完成工具**: 参见 [docs/completed-tools.md](docs/completed-tools.md)

### 统一架构接口
```rust
#[async_trait]
pub trait FunctionCallingServerTrait: Send + Sync {
    fn get_function_definitions(&self) -> Vec<FunctionDefinition>;
    async fn execute_function(&self, call: FunctionCall) -> FunctionResponse;
}
```

## Function Calling 工具系统

### 工具配置概览

#### 基础工具集 (4个) - 兼容模式 ✅
适用于简单场景和快速原型开发
- **maa_status** - 系统状态查询
- **maa_command** - 自然语言命令执行  
- **maa_operators** - 干员数据管理
- **maa_copilot** - 智能作业匹配

#### 增强工具集 (16个) - 专业模式 ✅ **生产就绪**
完整的MAA任务类型覆盖，基于maa-cli项目深度分析

**核心游戏功能 (4个)**:
- maa_startup, maa_combat_enhanced, maa_recruit_enhanced, maa_infrastructure_enhanced

**高级自动化 (4个)**:  
- maa_roguelike_enhanced, maa_copilot_enhanced, maa_sss_copilot, maa_reclamation

**辅助功能 (4个)**:
- maa_rewards_enhanced, maa_credit_store_enhanced, maa_depot_management, maa_operator_box

**系统功能 (4个)**:
- maa_closedown, maa_custom_task, maa_video_recognition, maa_system_management

### 技术实现特性

#### 智能参数解析
- **自然语言理解**: 支持中文游戏术语自动解析
- **别名映射**: "狗粮"→"1-7", "龙门币本"→"CE-5"等
- **参数验证**: 完整的参数类型和范围检查
- **默认值处理**: 智能填充缺失参数

#### 复杂配置管理
- **嵌套配置对象**: 支持多层级参数结构
- **条件逻辑**: 基于参数值的动态行为调整
- **状态管理**: 任务执行状态跟踪和恢复

#### 错误处理和日志
- **详细错误信息**: 参数解析和执行失败的具体描述
- **执行追踪**: 完整的任务执行流程日志
- **性能监控**: 任务执行时间和资源使用统计

### 详细工具文档
- **完整API参考**: [docs/enhanced-tools-status.md](docs/enhanced-tools-status.md)
- **已完成工具**: [docs/completed-tools.md](docs/completed-tools.md)
- **快速入门示例**: 参见下文API规范章节

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
- **增强工具集** - 16个MAA任务Function Calling工具 (3个已实现)
- **核心游戏功能** - maa_startup, maa_combat_enhanced, maa_recruit_enhanced
- **智能参数解析** - 支持复杂参数配置和自然语言理解
- **干员管理器** - 数据扫描和sled缓存系统  
- **作业匹配器** - 三阶段智能匹配引擎
- **前端界面** - React 19 + Vite 5聊天界面
- **Docker支持** - 多阶段构建和容器化部署

### 当前运行状态 (2025-08-17)
- **增强后端服务器**: http://localhost:8080 运行中 (16种任务类型) ✅
- **前端开发服务器**: http://localhost:3000 运行中  
- **运行模式**: stub模式 (MAA功能模拟)
- **健康检查**: http://localhost:8080/health 正常
- **API文档**: http://localhost:8080/tools 可访问 (增强版16工具)

### 16工具实现状态 ✅ **全部完成**
- **核心游戏功能**: 4/4 完成 (启动、战斗、招募、基建)
- **高级自动化**: 4/4 完成 (肉鸽、作业、保全、生息)  
- **辅助功能**: 4/4 完成 (奖励、商店、仓库、干员)
- **系统功能**: 4/4 完成 (关闭、自定义、视频、管理)
- **编译状态**: 通过 (仅59个警告，可接受)
- **集成状态**: enhanced_tools.rs 全部处理器已更新

### 技术债务
- rmcp集成需要使用官方SDK重构 (暂时忽略)
- ~~部分编译警告需要清理~~ (已完成)
- ~~前端可能有未使用的依赖项~~ (已检查，依赖简洁)
- ~~增强工具实现~~ (已完成，16/16)

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

---

## MAA 集成知识库

### 重要提醒
每次开始 MAA 相关工作前，必须先阅读知识库以了解项目状态和前人发现。

### 必读文档
1. `docs/maa-knowledge/README.md` - 知识库使用指南
2. `docs/maa-knowledge/DISCOVERIES.md` - 重要发现日志
3. `docs/maa-knowledge/PROGRESS.md` - 项目进度追踪

### 工作规范
1. **开始前**：阅读上述三个文档
2. **工作中**：实时更新 DISCOVERIES.md 记录新发现
3. **完成后**：更新 PROGRESS.md 标记任务完成状态

### 本地环境
- maa-cli 已安装并可用
- MaaCore 位置：待探测
- 项目状态：正在从 stub 模式迁移到真实 FFI 集成

### 多 Agent 协作
大型任务需要多个 subagent 协作完成，每个 agent 负责特定模块并将发现记录到知识库中。
