#!/bin/bash

# MAA并行研究终止脚本
# 清理所有研究相关的TMux会话

echo "🛑 终止MAA并行研究环境..."

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 显示当前会话
echo -e "${BLUE}当前TMux会话:${NC}"
tmux list-sessions 2>/dev/null

echo ""
echo -e "${YELLOW}终止所有MAA研究会话...${NC}"

# 定义要终止的会话列表
sessions=(
    "maa-main"
    "maa-task-system" 
    "maa-image-rec"
    "maa-infrast"
    "maa-battle"
    "maa-ffi"
)

# 逐个终止会话
for session in "${sessions[@]}"; do
    if tmux has-session -t "$session" 2>/dev/null; then
        echo -e "${GREEN}终止会话: $session${NC}"
        tmux kill-session -t "$session"
    else
        echo -e "${YELLOW}会话不存在: $session${NC}"
    fi
done

echo ""
echo -e "${BLUE}清理完成后的TMux会话:${NC}"
tmux list-sessions 2>/dev/null

if [ $? -eq 1 ]; then
    echo -e "${GREEN}✅ 所有MAA研究会话已终止${NC}"
else
    echo -e "${YELLOW}⚠️  仍有其他TMux会话运行中${NC}"
fi

echo ""
echo -e "${BLUE}研究环境清理完成! 👋${NC}"