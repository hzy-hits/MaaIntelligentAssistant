#!/bin/bash

# MAA 智能控制中间层启动脚本
# 基于消息队列 + 单线程工作者架构
# 同时启动后端服务和前端界面

set -e

echo "🚀 启动 MAA 智能控制中间层..."

# 检查是否在正确的目录
if [ ! -f "Cargo.toml" ]; then
    echo "❌ 错误：请在项目根目录运行此脚本"
    exit 1
fi

# 检查运行模式
if [[ "${1:-}" == "--production" ]] || [[ "${1:-}" == "--real" ]]; then
    echo "🔥 生产模式：启用真实 MAA Core 集成"
    FEATURES="--features with-maa-core"
    MODE_DESC="(真实 MAA)"
else
    echo "🛠️ 开发模式：使用模拟 MAA 功能"
    FEATURES=""
    MODE_DESC="(模拟模式)"
fi

# 启动后端服务
echo "📡 启动 MAA 后端服务 ${MODE_DESC}..."
cargo run --bin maa-server ${FEATURES} &
BACKEND_PID=$!

# 等待后端启动
echo "⏳ 等待后端服务启动..."
sleep 3

# 检查后端是否启动成功
if curl -s http://localhost:8080/health > /dev/null; then
    echo "✅ 后端服务启动成功"
    
    # 显示系统状态
    echo "📊 系统架构: HTTP异步 → 消息队列 → MAA单线程工作者"
    
    # 获取可用工具数量
    TOOLS_COUNT=$(curl -s http://localhost:8080/tools 2>/dev/null | jq -r '.functions | length' 2>/dev/null || echo "未知")
    echo "🔧 可用 Function Calling 工具: ${TOOLS_COUNT} 个"
else
    echo "❌ 后端服务启动失败"
    kill $BACKEND_PID 2>/dev/null || true
    exit 1
fi

# 检查前端目录是否存在
if [ -d "maa-chat-ui" ]; then
    # 进入前端目录并启动
    echo "💬 启动前端界面..."
    cd maa-chat-ui

    # 安装依赖（如果需要）
    if [ ! -d "node_modules" ]; then
        echo "📦 安装前端依赖..."
        npm install
    fi

    # 启动前端
    npm run dev &
    FRONTEND_PID=$!
    
    FRONTEND_AVAILABLE="✅ Web UI: http://localhost:3000"
else
    echo "⚠️  前端目录不存在，仅启动后端服务"
    FRONTEND_PID=""
    FRONTEND_AVAILABLE="❌ Web UI: 未安装"
fi

echo ""
echo "🎉 MAA 智能控制中间层启动完成！"
echo ""
echo "📡 后端 API: http://localhost:8080"
echo "${FRONTEND_AVAILABLE}"
echo ""
echo "🔍 快速测试:"
echo "  健康检查: curl http://localhost:8080/health"
echo "  工具列表: curl http://localhost:8080/tools"
echo ""
echo "⚡ 按 Ctrl+C 停止所有服务"

# 等待用户中断
trap 'echo ""; echo "🛑 正在停止所有服务..."; kill $BACKEND_PID $FRONTEND_PID 2>/dev/null || true; echo "✅ 服务已停止"; exit 0' INT

# 保持脚本运行
wait