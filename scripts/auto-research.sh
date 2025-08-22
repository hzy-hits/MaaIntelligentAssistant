#!/bin/bash

# MAA自动化并行研究脚本
# 自动创建TMux会话并启动Sub Agent任务

echo "🚀 启动MAA自动化并行研究..."

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 项目配置
PROJECT_PATH="/Users/ivena/Desktop/Fairy/maa/maa-remote-server"
RESEARCH_DIR="$HOME/maa-research/maa-official-study"

# 检查依赖
check_dependencies() {
    echo -e "${BLUE}检查依赖环境...${NC}"
    
    if ! command -v tmux &> /dev/null; then
        echo -e "${RED}❌ 未安装tmux${NC}"
        echo "安装: brew install tmux (macOS) 或 sudo apt install tmux (Linux)"
        exit 1
    fi
    
    if [ ! -d "$RESEARCH_DIR" ]; then
        echo -e "${YELLOW}⚠️  MAA研究目录不存在，正在创建...${NC}"
        mkdir -p ~/maa-research
        cd ~/maa-research
        git clone --depth 1 https://github.com/MaaAssistantArknights/MaaAssistantArknights.git maa-official-study
        echo -e "${GREEN}✅ MAA研究目录创建完成${NC}"
    fi
    
    if [ ! -d "$PROJECT_PATH" ]; then
        echo -e "${RED}❌ 项目路径不存在: $PROJECT_PATH${NC}"
        exit 1
    fi
}

# 清理已存在的会话
cleanup_sessions() {
    echo -e "${YELLOW}清理已存在的研究会话...${NC}"
    sessions=("maa-main" "maa-task-system" "maa-image-rec" "maa-infrast" "maa-battle" "maa-ffi")
    
    for session in "${sessions[@]}"; do
        if tmux has-session -t "$session" 2>/dev/null; then
            echo -e "${PURPLE}终止会话: $session${NC}"
            tmux kill-session -t "$session"
        fi
    done
}

# 创建研究会话并启动Agent
create_research_session() {
    local session_name=$1
    local agent_num=$2
    local agent_desc=$3
    local research_focus=$4
    local output_file=$5
    local status=$6
    
    echo -e "${CYAN}创建 $session_name 会话 (Agent $agent_num)${NC}"
    
    # 创建会话
    tmux new-session -d -s "$session_name"
    
    # 设置研究环境
    tmux send-keys -t "$session_name" "cd $RESEARCH_DIR" Enter
    tmux send-keys -t "$session_name" "clear" Enter
    
    # 显示Agent信息
    tmux send-keys -t "$session_name" "echo '════════════════════════════════════════════'" Enter
    tmux send-keys -t "$session_name" "echo '🎯 Agent $agent_num: $agent_desc'" Enter
    tmux send-keys -t "$session_name" "echo '════════════════════════════════════════════'" Enter
    tmux send-keys -t "$session_name" "echo '📁 研究目录: $(pwd)'" Enter
    tmux send-keys -t "$session_name" "echo '🎯 研究重点: $research_focus'" Enter
    tmux send-keys -t "$session_name" "echo '📄 输出文档: $output_file'" Enter
    tmux send-keys -t "$session_name" "echo '📊 当前状态: $status'" Enter
    tmux send-keys -t "$session_name" "echo '════════════════════════════════════════════'" Enter
    tmux send-keys -t "$session_name" "echo ''" Enter
    
    # 根据Agent类型显示相关文件
    case $agent_num in
        "2")
            tmux send-keys -t "$session_name" "echo '🔍 图像识别相关文件:'" Enter
            tmux send-keys -t "$session_name" "find resource -name '*.png' | head -5" Enter
            ;;
        "3")
            tmux send-keys -t "$session_name" "echo '🏗️  基建配置文件:'" Enter
            tmux send-keys -t "$session_name" "ls -la resource/custom_infrast/" Enter
            ;;
        "4")
            tmux send-keys -t "$session_name" "echo '⚔️  战斗作业文件:'" Enter  
            tmux send-keys -t "$session_name" "ls -la resource/copilot/ | head -5" Enter
            ;;
        "5")
            tmux send-keys -t "$session_name" "echo '🔗 FFI接口文件:'" Enter
            tmux send-keys -t "$session_name" "ls -la include/" Enter
            ;;
    esac
    
    tmux send-keys -t "$session_name" "echo ''" Enter
    tmux send-keys -t "$session_name" "echo '💡 手动启动Sub Agent研究:'" Enter
    tmux send-keys -t "$session_name" "echo '1. cd $PROJECT_PATH'" Enter
    tmux send-keys -t "$session_name" "echo '2. claude --session $session_name'" Enter
    tmux send-keys -t "$session_name" "echo '3. 输入研究任务提示'" Enter
    tmux send-keys -t "$session_name" "echo ''" Enter
}

# 创建主控制会话
create_main_session() {
    echo -e "${BLUE}创建主控制会话...${NC}"
    
    tmux new-session -d -s maa-main
    tmux send-keys -t maa-main "clear" Enter
    tmux send-keys -t maa-main "echo '🎛️  MAA并行研究主控制台'" Enter
    tmux send-keys -t maa-main "echo '════════════════════════════════════════════'" Enter
    tmux send-keys -t maa-main "echo ''" Enter
    tmux send-keys -t maa-main "echo '📊 研究会话状态:'" Enter
    tmux send-keys -t maa-main "tmux list-sessions" Enter
    tmux send-keys -t maa-main "echo ''" Enter
    tmux send-keys -t maa-main "echo '🚀 切换到研究会话:'" Enter
    tmux send-keys -t maa-main "echo '  tmux attach -t maa-task-system  # ✅ Agent 1 (已完成)'" Enter
    tmux send-keys -t maa-main "echo '  tmux attach -t maa-image-rec    # 🔄 Agent 2 (图像识别)'" Enter
    tmux send-keys -t maa-main "echo '  tmux attach -t maa-infrast      # 🔄 Agent 3 (基建调度)'" Enter
    tmux send-keys -t maa-main "echo '  tmux attach -t maa-battle       # 🔄 Agent 4 (战斗策略)'" Enter
    tmux send-keys -t maa-main "echo '  tmux attach -t maa-ffi          # 🔄 Agent 5 (FFI接口)'" Enter
    tmux send-keys -t maa-main "echo ''" Enter
    tmux send-keys -t maa-main "echo '⌨️  快捷操作:'" Enter
    tmux send-keys -t maa-main "echo '  Ctrl+B, D        # 分离当前会话'" Enter
    tmux send-keys -t maa-main "echo '  tmux list-sessions # 查看所有会话'" Enter
    tmux send-keys -t maa-main "echo '  ./scripts/stop-parallel-research.sh # 终止所有会话'" Enter
    tmux send-keys -t maa-main "echo ''" Enter
    tmux send-keys -t maa-main "echo '⏱️  实时监控 (每5秒刷新):'" Enter
    tmux send-keys -t maa-main "watch -n 5 'tmux list-sessions && echo && echo \"📈 研究进度:\" && ls -la docs/maa-research/*.md 2>/dev/null | wc -l | xargs -I {} echo \"已完成文档: {} 份\"'" Enter
}

# 主函数
main() {
    echo -e "${PURPLE}═══════════════════════════════════════════════════════${NC}"
    echo -e "${PURPLE}           🧠 MAA智能研究并行启动系统 v2.0            ${NC}"
    echo -e "${PURPLE}═══════════════════════════════════════════════════════${NC}"
    echo ""
    
    # 检查依赖
    check_dependencies
    
    # 清理旧会话
    cleanup_sessions
    sleep 1
    
    # 创建研究会话
    echo -e "${GREEN}🔧 创建并行研究环境...${NC}"
    
    create_research_session "maa-image-rec" "2" "图像识别系统" "模板匹配+OCR+界面判定" "IMAGE_RECOGNITION.md" "🔄 待启动"
    create_research_session "maa-infrast" "3" "基建智能调度" "排班配置+效率计算+调度算法" "INFRAST_SCHEDULING.md" "🔄 待启动"  
    create_research_session "maa-battle" "4" "战斗决策系统" "作业系统+操作序列+技能判定" "BATTLE_STRATEGY.md" "🔄 待启动"
    create_research_session "maa-ffi" "5" "FFI接口设计" "C接口+回调机制+Python桥接" "FFI_INTEGRATION.md" "🔄 待启动"
    
    # 创建主控制会话
    create_main_session
    
    echo ""
    echo -e "${GREEN}✅ 并行研究环境创建完成!${NC}"
    echo ""
    echo -e "${YELLOW}📋 后续操作指南:${NC}"
    echo -e "${CYAN}1. 查看主控制台:${NC} tmux attach -t maa-main"
    echo -e "${CYAN}2. 启动研究任务:${NC} 在各会话中运行 claude --session <session-name>"
    echo -e "${CYAN}3. 分离会话:${NC} Ctrl+B, 然后按 D"
    echo -e "${CYAN}4. 终止所有研究:${NC} ./scripts/stop-parallel-research.sh"
    echo ""
    echo -e "${BLUE}🎯 当前TMux会话状态:${NC}"
    tmux list-sessions
    echo ""
    echo -e "${PURPLE}研究环境已就绪，开始MAA深度分析! 🚀${NC}"
}

# 执行主函数
main