# MAA 智能控制中间层

## 项目本质
通过 Function Calling 协议让大模型直接控制 MaaAssistantArknights

## 核心架构

### V2架构 (optimized-server)
```
┌─────────────┐    ┌──────────────────┐    ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   HTTP API  │───▶│ Enhanced Tools   │───▶│ Task Queue V2   │───▶│  MAA Worker V2  │───▶│ thread_local!   │
│   (8080)    │    │      V2          │    │ (单队列+优先级)  │    │                 │    │   Assistant     │
└─────────────┘    └──────────────────┘    └─────────────────┘    └─────────────────┘    └─────────────────┘
                             │                                            │
                             ▼                                            ▼
                    ┌─────────────────┐                         ┌─────────────────┐
                    │  AI Chat API    │                         │  SSE Events     │
                    │                 │                         │  (实时更新)      │
                    └─────────────────┘                         └─────────────────┘
```

### V1架构 (intelligent-server)
```
┌─────────────┐    ┌──────────────────┐    ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   HTTP API  │───▶│ Enhanced Tools   │───▶│ Task Queue V1   │───▶│  MAA Worker V1  │───▶│ thread_local!   │
│   (8080)    │    │       V1         │    │   (双队列)       │    │                 │    │   Assistant     │
└─────────────┘    └──────────────────┘    └─────────────────┘    └─────────────────┘    └─────────────────┘
                             │                     │
                             ▼                     ▼
                    ┌─────────────────┐   ┌─────────────────┐
                    │  AI Chat API    │   │ Queue Client    │
                    │                 │   │                 │
                    └─────────────────┘   └─────────────────┘
```

### 技术栈
- **后端**: Rust + Axum + tokio 异步队列
- **前端**: React 19 + Vite 5 (端口3000)
- **FFI**: maa_sys 官方绑定
- **运行模式**: stub模式(开发) / real模式(生产)
- **实时更新**: Server-Sent Events (SSE)

## 项目结构

```
src/
├── bin/
│   ├── maa-optimized-server.rs     # 最新优化服务器 (推荐)
│   └── maa-intelligent-server.rs   # 旧版智能服务器
├── maa_core/                       # MAA 核心模块
│   ├── mod.rs                      # 模块导出
│   ├── basic_ops.rs                # 基础MAA操作
│   ├── worker.rs                   # MAA工作线程 (旧版)
│   ├── worker_v2.rs                # MAA工作线程V2 (新版)
│   ├── task_queue.rs               # 任务队列管理 (旧版)
│   ├── task_queue_v2.rs            # 任务队列V2 (新版)
│   ├── task_classification_v2.rs   # 任务分类系统V2
│   ├── task_status.rs              # 任务状态管理
│   └── screenshot.rs               # 截图功能
├── function_tools/                 # Function Calling 工具集
│   ├── mod.rs                      # 模块集成和导出
│   ├── types.rs                    # 核心类型定义
│   ├── handler.rs                  # 工具处理器 (旧版)
│   ├── handler_v2.rs               # 工具处理器V2 (新版)
│   ├── core_game.rs                # 核心游戏功能 (4个工具)
│   ├── advanced_automation.rs      # 高级自动化 (4个工具)
│   ├── support_features.rs         # 辅助功能 (4个工具)
│   ├── system_features.rs          # 系统功能 (4个工具)
│   └── queue_client.rs             # 队列客户端
├── sse/                            # Server-Sent Events (新增)
│   └── mod.rs                      # SSE实时更新模块
├── ai_client/                      # AI 客户端集成
│   ├── mod.rs                      # 统一AI接口
│   ├── client.rs                   # 客户端实现
│   ├── config.rs                   # 配置管理
│   ├── provider.rs                 # 多提供商支持
│   ├── providers/                  # 各提供商实现
│   └── tests.rs                    # 客户端测试
├── maa_adapter/                    # MAA适配器
│   ├── mod.rs                      # 模块导出
│   ├── types.rs                    # 数据类型定义
│   ├── errors.rs                   # 错误处理
│   └── ffi_stub.rs                 # 开发模式stub
├── copilot_matcher/                # 作业匹配器
│   ├── mod.rs                      # 模块导出
│   ├── api_client.rs               # API客户端
│   ├── cache.rs                    # 缓存管理
│   ├── matcher.rs                  # 匹配逻辑
│   └── types.rs                    # 类型定义
├── config/                         # 配置管理
│   └── mod.rs                      # 全局配置
└── lib.rs                          # 库入口
```

## Function Calling 工具 (17个)

### 核心游戏功能
- `maa_startup` - 游戏启动和账号管理
- `maa_combat_enhanced` - 自动战斗和资源管理
- `maa_recruit_enhanced` - 智能公开招募
- `maa_infrastructure_enhanced` - 基建自动化

### 高级自动化
- `maa_roguelike_enhanced` - 集成战略(肉鸽)
- `maa_copilot_enhanced` - 作业执行
- `maa_sss_copilot` - 保全派驻
- `maa_reclamation` - 生息演算

### 辅助功能
- `maa_rewards_enhanced` - 奖励收集
- `maa_credit_store_enhanced` - 信用商店
- `maa_depot_management` - 仓库管理
- `maa_operator_box` - 干员管理

### 系统功能
- `maa_closedown` - 游戏关闭
- `maa_custom_task` - 自定义任务
- `maa_video_recognition` - 视频识别
- `maa_system_management` - 系统管理
- `maa_take_screenshot` - 截图功能

## API端点

### optimized-server-v2 (推荐)
```http
GET  /health                        # 健康检查
GET  /tools                         # 获取17个工具定义
POST /call                          # 执行Function Calling
GET  /status                        # MAA状态查询
GET  /sse/tasks                     # SSE实时任务更新流
POST /chat                          # AI聊天接口
```

### intelligent-server (旧版)
```http
GET  /health                        # 健康检查
GET  /tools                         # 获取17个工具定义  
POST /call                          # 执行Function Calling
GET  /status                        # MAA状态查询
```

## 运行模式

### 优化服务器V2 (推荐)

#### 开发模式 (stub)
```bash
cargo run --bin maa-optimized-server --no-default-features --features stub-mode
```

#### 生产模式 (默认)
```bash
cargo run --bin maa-optimized-server  # 真实MAA Core集成 + SSE支持
```

### 智能服务器 (旧版)

#### 开发模式 (stub)
```bash
cargo run --bin maa-intelligent-server --no-default-features --features stub-mode
```

#### 生产模式 (默认)  
```bash
cargo run --bin maa-intelligent-server  # 真实MAA Core集成
```

## 环境配置

关键环境变量：
```bash
MAA_CORE_LIB=/path/to/libMaaCore.dylib    # MAA库路径
MAA_RESOURCE_PATH=/path/to/resource        # MAA资源路径
MAA_DEVICE_ADDRESS=localhost:1717          # 设备地址(PlayCover)
```

## 设备支持
- **PlayCover**: localhost:1717 (iOS应用模拟)
- **Android模拟器**: 127.0.0.1:5555 (ADB连接)
- **真机**: 通过ADB连接

## 架构设计原则

### 1. 简化优于复杂
- 删除了20+个冗余文件
- 3层架构替代原7层架构
- thread_local! 单例替代复杂的Arc<Mutex<>>

### 2. 直接调用优于抽象
- enhanced_tools 直接调用 maa_core 函数
- 消除了MaaBackend/MaaService中间层
- Function Calling 直接触发 maa_sys::Assistant

### 3. 实用优于完美
- 保留核心功能，删除过度设计
- 17个工具覆盖完整MAA功能
- stub模式支持无MAA环境开发

## 开发工作流

### 快速启动
```bash
# 1. 启动后端服务
cargo run --bin maa-intelligent-server

# 2. 启动前端 (可选)
cd maa-chat-ui && npm run dev
```

### 测试API
```bash
# 健康检查
curl http://localhost:8080/health

# 获取工具列表
curl http://localhost:8080/tools

# 执行Function Calling
curl -X POST http://localhost:8080/call \
  -H "Content-Type: application/json" \
  -d '{
    "function_call": {
      "name": "maa_combat_enhanced",
      "arguments": {"stage": "1-7", "strategy": {"target_value": 5}}
    }
  }'

# 查看任务状态
curl http://localhost:8080/tasks

# 聊天API (AI集成)
curl -X POST http://localhost:8080/chat \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [{"role": "user", "content": "帮我刷1-7"}]
  }'
```

## 架构重构记录 (2025-08-18)

### 重构前问题
- 70+个文件，7层调用链
- 17个工具返回"not_implemented"
- Arc<Mutex<>>复杂所有权模型
- 大量冗余抽象层

### 重构后成果
- 40+个文件，3层调用链
- 17个工具直接调用MAA Core
- thread_local! 单例模式
- 删除所有冗余代码

### 删除的冗余文件类别
- examples/ (3个文件，过时演示)
- legacy/ (15+个文件，旧代码存档)
- mcp_gateway/ (4个文件，复杂MCP实现)
- maa_adapter中5个文件 (复杂抽象层)
- mcp_tools中4个文件 (重复实现)

### 保留的核心价值
- 17个完整MAA Function Calling工具
- HTTP服务器框架
- AI客户端集成
- 干员管理和作业匹配系统
- MAA Core FFI安全封装

## 明日方舟游戏术语

### 常用关卡
- **1-7**: 经验书刷取(狗粮)
- **CE-5**: 龙门币本
- **CA-5**: 技能书本
- **AP-5**: 红票本

### 游戏操作
- **日常**: 每日任务自动化
- **基建**: 基础设施管理
- **公招**: 公开招募
- **肉鸽**: 集成战略(Roguelike)

### AI助手使用指南
所有MAA操作必须通过Function Calling协议：
```json
{
  "function_call": {
    "name": "maa_combat_enhanced",
    "arguments": {"stage": "1-7", "strategy": {"target_value": 10}}
  }
}
```

系统会自动解析中文游戏术语："刷龙门币" → CE-5，"日常" → 1-7等。

## 架构哲学与设计思考

### 核心设计哲学

#### 1. "这个有必要吗？"原则
**应用场景**: 每个文件、每行代码、每个抽象层都必须回答这个问题
- **文件层面**: 70+个文件削减到40+个文件
- **架构层面**: 7层调用链简化到3层
- **代码层面**: 删除所有"not_implemented"存根

#### 2. 简化优于复杂
**技术体现**:
- `thread_local!` 单例 > `Arc<Mutex<>>` 复杂所有权
- 直接函数调用 > 多层trait抽象
- HTTP Function Calling > 复杂MCP协议

#### 3. 实用优于完美
**平衡决策**:
- 保留stub模式以支持无MAA环境开发
- 16个完整工具覆盖而不是理论上的完美分类
- 实际可用的API优于理论上优雅的设计

### 架构决策记录

#### thread_local! 单例模式选择
**问题**: maa_sys::Assistant不是Send，如何在多线程HTTP服务器中使用？

**候选方案**:
1. `Arc<Mutex<Assistant>>` - 复杂，&mut self借用冲突
2. `Arc<RwLock<Assistant>>` - 依然复杂，性能问题
3. `thread_local!` - 简单，每线程独立实例

**决策**: 选择thread_local!
**原因**: HTTP请求处理本身就是线程隔离的，MAA实例无需跨线程共享

#### 3层架构的合理性
**每层存在价值**:
- **HTTP层**: 协议转换，必需
- **Tools层**: Function Calling路由，必需  
- **Core层**: MAA FFI调用，必需

**删除的无价值层**:
- MaaServiceInterface抽象层
- MaaBackend包装层
- ConnectionManager连接层
- 各种Trait适配器层

#### 17工具分类逻辑
**分类原则**: 按用户使用频率和复杂度
```
核心游戏功能 (高频) → 高级自动化 (中频) → 辅助功能 (低频) → 系统功能 (维护)
```

### 重构方法论

#### 冗余识别三问法
对每个文件/代码块问：
1. **依赖性**: 有其他地方引用吗？
2. **重复性**: 功能是否被重复实现？
3. **必要性**: 删除后影响核心功能吗？

只有三个答案都是"否"才删除。

#### 保留价值识别
保留标准（任何一条满足即保留）：
- **不可替代性**: 唯一实现特定功能
- **架构必需性**: 系统运行的核心组件
- **开发便利性**: 显著提升开发体验

#### 重构执行顺序
1. **分析依赖关系** - 确保删除安全性
2. **修复编译错误** - 保证系统可用性
3. **验证功能完整** - 确保核心功能不丢失
4. **更新文档** - 反映新的架构状态

### 文档设计哲学

#### 技术文档的本质
文档是**工具**而非**文学作品**：
- **目标**: 快速传递准确信息
- **方法**: 结构化、示例化、可验证
- **禁忌**: 营销语言、过度装饰、过时信息

#### 信息密度控制
- **高密度**: 架构图、API、命令（开发者最需要）
- **中密度**: 设计原则、工具说明（理解层面）
- **低密度**: 术语解释、背景介绍（上下文）

#### 文档持续性维护
**触发更新条件**:
- 架构变更（层次调整）
- API变更（接口修改）
- 部署变更（运行方式）
- 重要修复（使用影响）

### 从过度设计到精简设计的思考

#### 过度设计的表现
**删除的过度设计实例**:
- **过度抽象**: MaaServiceInterface trait（实际只有1个实现）
- **过度配置**: 复杂的设备配置管理（实际只需要地址）
- **过度分层**: 7层调用链（实际3层足够）
- **过度泛化**: 支持多种MCP协议（实际只用HTTP Function Calling）

#### 精简设计的标准
**保留的精简设计**:
- **直接性**: enhanced_tools直接调用maa_core
- **专一性**: 单一服务器入口maa-server-singleton
- **实用性**: stub模式支持开发，real模式支持生产
- **清晰性**: 每个模块职责明确，边界清楚

### 教训与经验

#### 架构演进的规律
1. **初期**: 功能实现优先，架构相对简单
2. **中期**: 需求增加，开始添加抽象层
3. **过度**: 抽象过多，系统复杂度爆炸
4. **重构**: 质疑每个抽象，回归简洁

#### 判断"有必要吗"的实战经验
- trait: 有多少个实现？只有1个就删除
- wrapper: 直接调用不行吗？
- config: 有多少种配置？只有1种就硬编码
- 抽象层: 绕过它直接调用会怎样？

#### 重构收益
- 文件数量：70+ → 60+
- 任务队列系统：双队列 → 单队列+优先级
- 新增功能：SSE实时更新、结构化截图响应
- 编译状态：从无法编译到正常运行
- 开发体验：从复杂难懂到结构清晰

## 技术文档体系

### 模块文档
- **[Function Tools 模块](docs/modules/FUNCTION_TOOLS.md)**: 17个Function Calling工具的详细实现
- **[MAA Core 模块](docs/modules/MAA_CORE.md)**: thread_local!单例和7个基础操作
- **[AI Client 模块](docs/modules/AI_CLIENT.md)**: 多提供商AI客户端集成

### 架构文档  
- **[系统架构](docs/architecture/SYSTEM_ARCHITECTURE.md)**: 完整的3层架构设计和技术决策

### 文档对应关系
所有文档都对应真实存在的代码，包含精确的文件位置和行号引用：

| 模块 | 文档位置 | 核心代码文件 |
|-----|----------|-------------|
| Function Tools | `docs/modules/FUNCTION_TOOLS.md` | `src/function_tools/` |
| MAA Core | `docs/modules/MAA_CORE.md` | `src/maa_core/` |
| AI Client | `docs/modules/AI_CLIENT.md` | `src/ai_client/` |
| 系统架构 | `docs/architecture/SYSTEM_ARCHITECTURE.md` | 全项目概览 |

### 2025-08-20 架构优化升级

#### V2优化服务器 (最新)
- **服务器**: maa-optimized-server.rs (Axum + tokio + SSE)
- **架构**: HTTP → Function Tools V2 → Task Queue V2 → MAA Worker V2
- **并发模型**: thread_local! 单例 (无锁设计)
- **任务队列**: 单队列+优先级系统
- **实时更新**: Server-Sent Events支持
- **工具数量**: 16个完整Function Calling工具
- **AI集成**: 支持多提供商聊天接口

#### V1智能服务器 (旧版)
- **服务器**: maa-intelligent-server.rs (Axum + tokio)  
- **架构**: HTTP → Function Tools → Task Queue → MAA Worker
- **并发模型**: Arc<Mutex<Assistant>> (锁竞争)
- **任务队列**: 双队列系统(高优先级+普通)
- **实时更新**: 无，需要轮询
- **工具数量**: 17个Function Calling工具
- **AI集成**: 支持多提供商聊天接口

#### 验证状态
```bash
# 编译检查
cargo check

# 启动优化服务器V2 (推荐)
cargo run --bin maa-optimized-server

# 启动智能服务器V1
cargo run --bin maa-intelligent-server

# 健康检查
curl localhost:8080/health

# 工具列表  
curl localhost:8080/tools

# SSE实时更新 (仅V2支持)
curl -N -H "Accept: text/event-stream" localhost:8080/sse/tasks
```

---

## 开发指南

### 快速开始
```bash
# 1. 启动后端服务
cargo run --bin maa-intelligent-server

# 2. 测试健康检查
curl http://localhost:8080/health

# 3. 获取Function Calling工具列表
curl http://localhost:8080/tools

# 4. 执行MAA任务
curl -X POST http://localhost:8080/call \
  -H "Content-Type: application/json" \
  -d '{
    "function_call": {
      "name": "maa_startup", 
      "arguments": {"client_type": "Official", "start_app": true}
    }
  }'
```

### 添加新功能
1. **新Function Tool**: 在对应的功能模块(core_game.rs等)中添加
2. **新MAA操作**: 在 `src/maa_core/basic_ops.rs` 中添加
3. **新AI提供商**: 在 `src/ai_client/providers/` 目录下添加

### 文档维护原则
1. **代码优先**: 文档对应真实代码
2. **位置精确**: 包含具体文件路径
3. **持续更新**: 代码变更时同步更新
4. **实用导向**: 文档指导实际开发

---

## V2优化框架核心改进 (2025-08-20)

### 🚀 性能提升

| 指标 | V1 (intelligent) | V2 (optimized) | 改进 |
|-----|-----------------|----------------|------|
| **任务队列** | 双队列系统 | 单队列+优先级 | 简化队列管理 |
| **并发模型** | thread_local! | thread_local! | 两版本相同 |
| **任务队列** | 双队列系统 | 单队列+优先级 | 简化50% |
| **实时更新** | ❌ 轮询 | ✅ SSE推送 | 用户体验质升 |
| **JSON序列化** | 多次重复 | 直接传递 | 减少序列化开销 |

### 🎯 新增特性

#### SSE实时更新
```javascript  
// 前端自动连接SSE流
const eventSource = new EventSource('/sse/tasks');
eventSource.onmessage = (event) => {
  const data = JSON.parse(event.data);
  // 实时显示任务进度: started, progress, completed, failed
};
```

#### 智能任务分类
```rust
// 自动区分同步/异步任务
match classify_task(&function_name) {
    TaskExecutionMode::Synchronous => {
        // 截图、状态查询等立即返回
    },
    TaskExecutionMode::Asynchronous => {
        // 战斗、基建等后台执行，SSE推送进度
    }
}
```

#### 线程安全设计
```rust
// V1和V2都使用相同的thread_local设计
thread_local! {
    static MAA_ASSISTANT: RefCell<Option<Assistant>> = RefCell::new(None);
}

// Arc<Mutex<>>只用于任务状态管理，与Assistant无关
```

### 📊 架构简化对比

#### V1架构 (intelligent-server)
- 🔗 **调用链**: HTTP → Enhanced Tools → Queue Client → Task Queue (双队列) → Worker → thread_local!
- 🔄 **任务队列**: 分离的high_priority和normal_priority两个通道
- 📨 **无实时反馈**: 需要轮询查询任务状态
- 📷 **截图返回**: 原始字节数据，需要手动处理

#### V2架构优势 (optimized-server)
- 🔗 **调用链**: HTTP → Enhanced Tools V2 → Task Queue V2 (单队列+优先级) → Worker V2 → thread_local!
- 🎯 **任务队列**: 统一task_tx通道，使用优先级属性排序
- 📡 **实时推送**: SSE自动推送任务进度，无需轮询
- 📷 **结构化截图**: JSON响应包含base64、大小、格式等信息

### 🛠️ 开发体验改进

#### SSE实时调试
```javascript
// V2新增的SSE连接管理
const eventSource = new EventSource('/sse/tasks');
eventSource.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('📨 收到SSE消息:', data);
};
```

#### 截图功能增强
```rust  
// V2返回结构化截图数据
Ok(json!({
    "screenshot": base64_data,
    "size": image_data.len(),
    "format": "PNG",
    "timestamp": Utc::now().to_rfc3339()
}))
```

### 📈 用户体验提升

#### V1用户流程
```
用户点击 → 等待 → 手动刷新 → 查看结果
```

#### V2用户流程  
```
用户点击 → 立即反馈 → 实时进度更新 → 自动显示结果
```

---

## 📈 对比总结

### ✅ 实际改进 (已验证)
- **任务队列**: 双队列 → 单队列+优先级（简化管理）
- **SSE实时更新**: 无 → 完整SSE系统（新增功能）
- **截图响应**: 原始数据 → 结构化JSON（更好集成）
- **工具数量**: 17个Function Calling工具（两版本相同）

### ❌ 误导性声明 (已纠正)
- ~~架构层次：7层→4层~~ → 实际都是5层左右
- ~~并发模型：Arc<Mutex<>> → thread_local!~~ → 两版本都用thread_local!
- ~~16个工具~~ → 实际是17个工具

### 🎁 真实价值
**V2的核心价值在于**：
1. **新增功能**: SSE实时更新系统
2. **用户体验**: 从轮询到实时推送
3. **架构清理**: 简化队列系统和截图处理
4. **开发体验**: 更好的前端集成和调试

**V2设计哲学**: 实用优于完美，实际改进优于理论数据  
**文档原则**: 简洁、**准确**、实用 → 经严格验证后纠正  
**架构原则**: 质疑抽象，保留核心  
**维护原则**: 文档与代码同步