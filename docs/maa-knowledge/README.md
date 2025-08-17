# MAA 集成知识库

## 概述

这个知识库记录了 MAA (MaaAssistantArknights) 与我们智能服务器的集成过程中的所有发现、分析和实现方案。

## 目录结构

```
docs/maa-knowledge/
├── README.md                       # 本文件 - 知识库使用指南
├── DISCOVERIES.md                  # 重要发现日志
├── PROGRESS.md                     # 进度追踪
├── architecture/                   # 架构分析
├── features/                       # 功能文档
├── protocols/                      # 协议规范
├── implementation/                 # 实现方案
└── examples/                       # 实际例子
```

## 使用规范

### 对于所有 Agent

**开始工作前必须**：
1. 阅读 `DISCOVERIES.md` 了解前人的重要发现
2. 阅读 `PROGRESS.md` 了解当前项目进度
3. 阅读相关的功能文档

**工作过程中必须**：
1. 实时更新 `DISCOVERIES.md` 记录新发现
2. 更新对应的功能文档
3. 记录遇到的问题和解决方案

**工作结束时必须**：
1. 更新 `PROGRESS.md` 标记完成的任务
2. 在 `DISCOVERIES.md` 总结关键发现
3. 为下一个 agent 留下明确的后续任务描述

### 文档更新原则

1. **及时性**：发现问题立即记录
2. **详细性**：包含足够的上下文信息
3. **可操作性**：后续 agent 能根据记录继续工作
4. **可追溯性**：记录决策的原因和依据

## 本地环境信息

- **maa-cli 状态**：已安装，可通过命令行测试
- **MaaCore 位置**：待探测并记录
- **资源文件位置**：待探测并记录
- **开发环境**：macOS，Rust 项目

## 项目目标

将我们的 Function Calling 工具从当前的 4 个简单工具，扩展到覆盖 MAA 全部功能的智能工具集：

- 支持 MAA 的全部 16 种任务类型
- 提供智能的自然语言理解
- 实现与本地 MaaCore 的无缝集成
- 为 AI 提供强大的游戏控制能力

## 重要链接

- [MAA 官方文档](https://maa.plus/docs/)
- [maa-cli 项目](https://github.com/MaaAssistantArknights/maa-cli)
- [MaaCore 主项目](https://github.com/MaaAssistantArknights/MaaAssistantArknights)