# MAA并行研究脚本使用指南

## 📋 脚本概览

| 脚本 | 功能 | 用途 |
|-----|------|------|
| `auto-research.sh` | 🚀 一键启动完整研究环境 | **推荐使用** |
| `start-parallel-research.sh` | 创建TMux会话 | 基础环境搭建 |
| `launch-agents.sh` | 启动Sub Agent任务 | 手动Agent管理 |
| `stop-parallel-research.sh` | 清理所有研究会话 | 环境清理 |

## 🚀 快速开始 (推荐)

### 1. 一键启动研究环境
```bash
cd /Users/ivena/Desktop/Fairy/maa/maa-remote-server
./scripts/auto-research.sh
```

这个脚本会自动：
- ✅ 检查并安装依赖 (tmux)
- ✅ 创建MAA研究目录 (~maa-research/)
- ✅ 克隆MAA官方仓库
- ✅ 创建6个TMux会话 (1个主控+5个研究)
- ✅ 设置每个会话的研究环境
- ✅ 显示详细的使用指南

### 2. 查看研究环境状态
```bash
# 查看所有TMux会话
tmux list-sessions

# 切换到主控制台
tmux attach -t maa-main
```

### 3. 启动各个Sub Agent研究
```bash
# Agent 2: 图像识别
tmux attach -t maa-image-rec
cd /Users/ivena/Desktop/Fairy/maa/maa-remote-server
claude --session maa-image-rec

# Agent 3: 基建调度  
tmux attach -t maa-infrast
cd /Users/ivena/Desktop/Fairy/maa/maa-remote-server
claude --session maa-infrast

# Agent 4: 战斗策略
tmux attach -t maa-battle
cd /Users/ivena/Desktop/Fairy/maa/maa-remote-server  
claude --session maa-battle

# Agent 5: FFI接口
tmux attach -t maa-ffi
cd /Users/ivena/Desktop/Fairy/maa/maa-remote-server
claude --session maa-ffi
```

### 4. 清理研究环境
```bash
./scripts/stop-parallel-research.sh
```

## 🔧 手动使用方式

### 步骤1: 创建基础环境
```bash
./scripts/start-parallel-research.sh
```

### 步骤2: 启动Agent任务
```bash
./scripts/launch-agents.sh
```

### 步骤3: 手动在各会话中启动Claude Code
```bash
# 切换到指定会话
tmux attach -t maa-image-rec

# 在会话中执行
cd /Users/ivena/Desktop/Fairy/maa/maa-remote-server
claude --session maa-image-rec
```

## 📊 研究任务详情

### Agent 1: 任务系统分析 ✅
- **状态**: 已完成
- **输出**: `docs/maa-research/TASK_SYSTEM.md` (952行)
- **会话**: maa-task-system

### Agent 2: 图像识别系统 🔄
- **研究重点**: 模板匹配、OCR识别、界面状态判定
- **输出**: `docs/maa-research/IMAGE_RECOGNITION.md`
- **会话**: maa-image-rec

### Agent 3: 基建智能调度 🔄
- **研究重点**: 排班配置、干员效率计算、调度算法
- **输出**: `docs/maa-research/INFRAST_SCHEDULING.md`
- **会话**: maa-infrast

### Agent 4: 战斗决策系统 🔄
- **研究重点**: 作业系统、操作序列、技能释放时机
- **输出**: `docs/maa-research/BATTLE_STRATEGY.md`
- **会话**: maa-battle

### Agent 5: FFI接口设计 🔄
- **研究重点**: C接口、回调机制、Python桥接方案
- **输出**: `docs/maa-research/FFI_INTEGRATION.md`
- **会话**: maa-ffi

## ⌨️ TMux快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+B, D` | 分离当前会话 |
| `Ctrl+B, C` | 创建新窗口 |
| `Ctrl+B, N` | 切换到下一个窗口 |
| `Ctrl+B, P` | 切换到上一个窗口 |
| `Ctrl+B, [` | 进入复制模式(可滚动查看历史) |

## 🔍 常用命令

```bash
# 查看所有TMux会话
tmux list-sessions

# 切换到指定会话
tmux attach -t <session-name>

# 在会话外向会话发送命令
tmux send-keys -t <session-name> "command" Enter

# 分离所有客户端
tmux detach

# 终止指定会话
tmux kill-session -t <session-name>

# 重命名会话
tmux rename-session -t <old-name> <new-name>
```

## 🐛 故障排除

### TMux会话无法创建
```bash
# 检查tmux是否安装
which tmux

# macOS安装
brew install tmux

# Linux安装  
sudo apt install tmux
```

### 研究目录不存在
```bash
# 手动创建研究环境
mkdir -p ~/maa-research
cd ~/maa-research
git clone --depth 1 https://github.com/MaaAssistantArknights/MaaAssistantArknights.git maa-official-study
```

### Claude Code无法启动
```bash
# 确认在正确的项目目录
cd /Users/ivena/Desktop/Fairy/maa/maa-remote-server

# 检查CLAUDE.md是否存在
ls -la .claude/CLAUDE.md

# 手动启动Claude Code
claude --session <session-name>
```

## 📈 研究进度追踪

```bash
# 查看已完成的研究文档
ls -la docs/maa-research/*.md

# 查看文档内容摘要
for file in docs/maa-research/*.md; do
    echo "=== $file ==="
    head -20 "$file"
    echo ""
done

# 监控研究进度
watch -n 10 'ls -la docs/maa-research/*.md | wc -l'
```

## 🎯 下一步行动

1. **启动环境**: `./scripts/auto-research.sh`
2. **启动Agent**: 在各会话中执行 `claude --session <session-name>`
3. **监控进度**: 通过主控制台 `maa-main` 查看状态
4. **收集成果**: 研究完成后整合各文档
5. **设计实现**: 基于研究成果设计Python决策层

---

**🚀 开始你的MAA深度研究之旅！**