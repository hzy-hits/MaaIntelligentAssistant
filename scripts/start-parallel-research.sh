#!/bin/bash

# MAA并行研究启动脚本
# 使用TMux创建多个Session，每个Sub Agent独立研究

echo "🚀 启动MAA并行研究环境..."

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'  
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 检查tmux是否安装
if ! command -v tmux &> /dev/null; then
    echo -e "${RED}错误: 请先安装tmux${NC}"
    echo "macOS: brew install tmux"
    echo "Ubuntu: sudo apt install tmux"
    exit 1
fi

# 检查研究目录是否存在
RESEARCH_DIR="$HOME/maa-research/maa-official-study"
if [ ! -d "$RESEARCH_DIR" ]; then
    echo -e "${RED}错误: 研究目录不存在: $RESEARCH_DIR${NC}"
    echo "请先运行: mkdir -p ~/maa-research && cd ~/maa-research && git clone --depth 1 https://github.com/MaaAssistantArknights/MaaAssistantArknights.git maa-official-study"
    exit 1
fi

# 终止已存在的研究session
echo -e "${YELLOW}清理已存在的研究会话...${NC}"
tmux kill-session -t maa-main 2>/dev/null
tmux kill-session -t maa-task-system 2>/dev/null  
tmux kill-session -t maa-image-rec 2>/dev/null
tmux kill-session -t maa-infrast 2>/dev/null
tmux kill-session -t maa-battle 2>/dev/null
tmux kill-session -t maa-ffi 2>/dev/null

echo -e "${BLUE}创建TMux研究会话...${NC}"

# 创建主控制会话
tmux new-session -d -s maa-main
tmux send-keys -t maa-main "echo '🎯 MAA研究主控制台'" Enter
tmux send-keys -t maa-main "echo '使用以下命令切换到各研究会话:'" Enter
tmux send-keys -t maa-main "echo '  tmux attach -t maa-task-system  # Agent 1: 任务系统'" Enter
tmux send-keys -t maa-main "echo '  tmux attach -t maa-image-rec    # Agent 2: 图像识别'" Enter  
tmux send-keys -t maa-main "echo '  tmux attach -t maa-infrast      # Agent 3: 基建调度'" Enter
tmux send-keys -t maa-main "echo '  tmux attach -t maa-battle       # Agent 4: 战斗策略'" Enter
tmux send-keys -t maa-main "echo '  tmux attach -t maa-ffi          # Agent 5: FFI接口'" Enter
tmux send-keys -t maa-main "echo ''" Enter
tmux send-keys -t maa-main "echo '实时监控所有会话状态:'" Enter
tmux send-keys -t maa-main "watch -n 2 'tmux list-sessions'" Enter

# 创建各研究会话
echo -e "${GREEN}创建Agent 1: 任务系统研究会话${NC}"
tmux new-session -d -s maa-task-system
tmux send-keys -t maa-task-system "cd $RESEARCH_DIR" Enter
tmux send-keys -t maa-task-system "echo '🎯 Agent 1: MAA任务系统深度分析'" Enter
tmux send-keys -t maa-task-system "echo '研究目标: JSON任务定义、状态机转换、决策模式'" Enter
tmux send-keys -t maa-task-system "echo '输出文档: docs/maa-research/TASK_SYSTEM.md'" Enter
tmux send-keys -t maa-task-system "echo '状态: ✅ 已完成 (952行分析文档)'" Enter
tmux send-keys -t maa-task-system "ls -la resource/tasks/" Enter

echo -e "${GREEN}创建Agent 2: 图像识别研究会话${NC}"
tmux new-session -d -s maa-image-rec  
tmux send-keys -t maa-image-rec "cd $RESEARCH_DIR" Enter
tmux send-keys -t maa-image-rec "echo '🎯 Agent 2: MAA图像识别系统分析'" Enter
tmux send-keys -t maa-image-rec "echo '研究目标: 模板匹配、OCR识别、界面状态判定'" Enter
tmux send-keys -t maa-image-rec "echo '输出文档: docs/maa-research/IMAGE_RECOGNITION.md'" Enter
tmux send-keys -t maa-image-rec "echo '状态: 🔄 待启动 Sub Agent...'" Enter
tmux send-keys -t maa-image-rec "find resource -name '*.png' | head -10" Enter

echo -e "${GREEN}创建Agent 3: 基建调度研究会话${NC}"
tmux new-session -d -s maa-infrast
tmux send-keys -t maa-infrast "cd $RESEARCH_DIR" Enter  
tmux send-keys -t maa-infrast "echo '🎯 Agent 3: MAA基建智能调度分析'" Enter
tmux send-keys -t maa-infrast "echo '研究目标: 排班配置、干员效率计算、调度算法'" Enter
tmux send-keys -t maa-infrast "echo '输出文档: docs/maa-research/INFRAST_SCHEDULING.md'" Enter
tmux send-keys -t maa-infrast "echo '状态: 🔄 待启动 Sub Agent...'" Enter
tmux send-keys -t maa-infrast "ls -la resource/custom_infrast/" Enter

echo -e "${GREEN}创建Agent 4: 战斗策略研究会话${NC}"
tmux new-session -d -s maa-battle
tmux send-keys -t maa-battle "cd $RESEARCH_DIR" Enter
tmux send-keys -t maa-battle "echo '🎯 Agent 4: MAA战斗决策系统分析'" Enter  
tmux send-keys -t maa-battle "echo '研究目标: 作业系统、操作序列、技能释放时机'" Enter
tmux send-keys -t maa-battle "echo '输出文档: docs/maa-research/BATTLE_STRATEGY.md'" Enter
tmux send-keys -t maa-battle "echo '状态: 🔄 待启动 Sub Agent...'" Enter
tmux send-keys -t maa-battle "ls -la resource/copilot/ | head -10" Enter

echo -e "${GREEN}创建Agent 5: FFI接口研究会话${NC}"
tmux new-session -d -s maa-ffi
tmux send-keys -t maa-ffi "cd $RESEARCH_DIR" Enter
tmux send-keys -t maa-ffi "echo '🎯 Agent 5: MAA FFI接口设计分析'" Enter
tmux send-keys -t maa-ffi "echo '研究目标: AsstCaller.h接口、回调机制、Python桥接'" Enter  
tmux send-keys -t maa-ffi "echo '输出文档: docs/maa-research/FFI_INTEGRATION.md'" Enter
tmux send-keys -t maa-ffi "echo '状态: 🔄 待启动 Sub Agent...'" Enter
tmux send-keys -t maa-ffi "ls -la include/" Enter

echo -e "${BLUE}📊 研究环境启动完成!${NC}"
echo ""
echo -e "${YELLOW}使用指南:${NC}"
echo "1. 查看所有会话: tmux list-sessions"
echo "2. 切换到主控制台: tmux attach -t maa-main"  
echo "3. 切换到研究会话: tmux attach -t maa-task-system"
echo "4. 分离当前会话: Ctrl+B, D"
echo "5. 终止所有研究: ./scripts/stop-parallel-research.sh"
echo ""
echo -e "${GREEN}各Sub Agent研究任务已准备就绪! 🚀${NC}"
echo ""

# 显示当前会话状态
echo -e "${BLUE}当前TMux会话状态:${NC}"
tmux list-sessions