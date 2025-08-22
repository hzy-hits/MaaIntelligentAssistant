#!/bin/bash

# 启动Sub Agent研究任务脚本
# 在指定的TMux会话中启动Claude Code Sub Agent

echo "🤖 启动MAA Sub Agent研究任务..."

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 项目路径
PROJECT_PATH="/Users/ivena/Desktop/Fairy/maa/maa-remote-server"

# 检查TMux会话是否存在的函数
check_session() {
    local session_name=$1
    if ! tmux has-session -t "$session_name" 2>/dev/null; then
        echo -e "${RED}错误: TMux会话 '$session_name' 不存在${NC}"
        echo "请先运行: ./scripts/start-parallel-research.sh"
        return 1
    fi
    return 0
}

# 启动Agent的通用函数
launch_agent() {
    local session_name=$1
    local agent_number=$2
    local agent_description=$3
    local task_prompt=$4
    
    echo -e "${BLUE}启动 Agent $agent_number: $agent_description${NC}"
    
    if check_session "$session_name"; then
        # 切换到项目目录并启动Claude Code
        tmux send-keys -t "$session_name" "cd $PROJECT_PATH" Enter
        tmux send-keys -t "$session_name" "echo '🚀 启动 Agent $agent_number: $agent_description'" Enter
        tmux send-keys -t "$session_name" "claude --session $session_name" Enter
        
        # 发送任务提示 (需要手动执行)
        echo -e "${YELLOW}在 $session_name 会话中执行以下任务:${NC}"
        echo "$task_prompt"
        echo ""
    fi
}

echo -e "${YELLOW}准备启动5个Sub Agent...${NC}"

# Agent 2: 图像识别系统
launch_agent "maa-image-rec" "2" "图像识别系统" "
深入研究MAA的图像识别系统，分析以下内容：
1. 模板匹配算法 (MatchTemplate)
2. OCR文字识别 (OcrDetect)  
3. 界面状态判定机制
4. ROI区域管理
5. 图像预处理流程
输出: docs/maa-research/IMAGE_RECOGNITION.md
"

# Agent 3: 基建智能调度
launch_agent "maa-infrast" "3" "基建智能调度" "
深入研究MAA的基建调度系统，分析以下内容：
1. custom_infrast/*.json 排班配置
2. 干员效率计算算法
3. 243/153/333布局策略
4. 心情管理机制
5. 无人机调度逻辑
输出: docs/maa-research/INFRAST_SCHEDULING.md
"

# Agent 4: 战斗决策系统  
launch_agent "maa-battle" "4" "战斗决策系统" "
深入研究MAA的战斗决策系统，分析以下内容：
1. copilot/*.json 作业系统
2. 战斗操作序列
3. 技能释放时机判定
4. 干员部署策略
5. 关卡自动化流程
输出: docs/maa-research/BATTLE_STRATEGY.md
"

# Agent 5: FFI接口设计
launch_agent "maa-ffi" "5" "FFI接口设计" "
深入研究MAA的FFI接口设计，分析以下内容：
1. include/AsstCaller.h C接口
2. 回调机制和事件处理
3. 任务管理和状态同步
4. Python绑定设计方案
5. PyO3集成架构建议
输出: docs/maa-research/FFI_INTEGRATION.md
"

echo -e "${GREEN}✅ 所有Sub Agent启动命令已发送到对应的TMux会话${NC}"
echo ""
echo -e "${BLUE}查看各会话状态:${NC}"
echo "tmux attach -t maa-image-rec    # Agent 2"
echo "tmux attach -t maa-infrast      # Agent 3"  
echo "tmux attach -t maa-battle       # Agent 4"
echo "tmux attach -t maa-ffi          # Agent 5"
echo ""
echo -e "${YELLOW}注意: 需要在各会话中手动启动Claude Code并输入研究任务${NC}"