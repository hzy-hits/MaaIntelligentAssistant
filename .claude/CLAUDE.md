# MAA智能控制中间层 - Claude Code项目记忆

## 项目概述
通过Function Calling协议让大模型直接控制MaaAssistantArknights的智能中间层系统。

## 当前研究阶段：MAA源码深度分析
正在进行MAA官方源码的并行研究，以便设计Python智能决策层。

@.claude/maa-research/research-plan.md
@.claude/maa-research/parallel-agents.md
@.claude/technical-stack.md
@.claude/development-workflow.md

## 架构设计哲学
1. **简化优于复杂** - thread_local! 单例 > Arc<Mutex<>>
2. **直接调用优于抽象** - Function Calling直接触发maa_sys::Assistant  
3. **实用优于完美** - 17个工具覆盖完整MAA功能

## 技术栈
- **后端**: Rust + Axum + tokio异步队列
- **决策层**: Python + PyO3 FFI桥接（计划中）
- **前端**: React 19 + Vite 5 (端口3000)
- **FFI**: maa_sys官方绑定
- **实时更新**: Server-Sent Events (SSE)
- **持久化**: sled/JSON轻量级存储

## 开发命令
```bash
# V2优化服务器(推荐)
cargo run --bin maa-optimized-server

# V1智能服务器
cargo run --bin maa-intelligent-server  

# 开发模式(stub)
cargo run --bin maa-optimized-server --no-default-features --features stub-mode

# 健康检查
curl localhost:8080/health

# 获取工具列表
curl localhost:8080/tools

# SSE实时更新
curl -N -H "Accept: text/event-stream" localhost:8080/sse/tasks

# 前端开发
cd maa-chat-ui && npm run dev
```

## 并行研究环境
- **研究目录**: ~/maa-research/maa-official-study
- **文档输出**: docs/maa-research/
- **TMux会话**: 6个独立研究session

## Function Calling工具(17个)
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

## 环境配置
```bash
MAA_CORE_LIB=/path/to/libMaaCore.dylib
MAA_RESOURCE_PATH=/path/to/resource  
MAA_DEVICE_ADDRESS=localhost:1717
```

## 项目状态
- ✅ V2架构重构完成 (简化队列+SSE支持)
- ✅ 17个Function Calling工具实现
- 🔄 MAA源码深度研究进行中
- ⏳ Python决策层设计阶段
- ⏳ PyO3集成开发计划

## 开发注意事项
- 保持Rust为架构层，Python为业务层的清晰分工
- 使用sled/JSON而非PostgreSQL进行轻量级持久化
- 所有MAA操作必须通过Function Calling协议
- 研究成果不污染主项目Git历史

## 团队协作
- 使用tmux session进行并行研究
- 每个Sub Agent负责独立模块分析
- 定期同步研究发现到CLAUDE.md
- 通过docs/maa-research/记录技术细节