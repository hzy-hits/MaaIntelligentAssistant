#!/bin/bash

# MAA 智能控制中间层 - 开发模式启动脚本
# 使用模拟 MAA 功能，支持热重载和详细日志

set -e

echo "🛠️ 启动 MAA 开发环境..."

# 检查是否在正确的目录
if [ ! -f "Cargo.toml" ]; then
    echo "❌ 错误：请在项目根目录运行此脚本"
    exit 1
fi

# 启动后端（开发模式 - Stub模式）
echo "🚀 启动后端服务（模拟模式 + 详细日志）..."
RUST_LOG=debug cargo run --bin maa-server &
BACKEND_PID=$!

# 等待后端启动
echo "⏳ 等待后端服务启动..."
sleep 3

# 检查后端健康状态
if curl -s http://localhost:8080/health > /dev/null; then
    echo "✅ 后端服务启动成功"
    echo "📊 架构模式: HTTP异步 → 消息队列 → MAA模拟器"
else
    echo "❌ 后端服务启动失败，请检查日志"
    kill $BACKEND_PID 2>/dev/null || true
    exit 1
fi

# 检查前端目录是否存在
if [ -d "maa-chat-ui" ]; then
    # 进入前端目录
    cd maa-chat-ui

    # 检查依赖
    if [ ! -d "node_modules" ]; then
        echo "📦 安装前端依赖..."
        npm install
    fi

    echo "💬 启动前端开发服务器..."
    npm run dev &
    FRONTEND_PID=$!
    
    FRONTEND_INFO="💬 前端界面: http://localhost:3000"
else
    echo "⚠️  前端目录不存在，仅启动后端服务"
    FRONTEND_PID=""
    FRONTEND_INFO="❌ 前端: 未安装"
fi

echo ""
echo "🎉 开发环境启动完成！"
echo ""
echo "🚀 后端 API: http://localhost:8080"
echo "${FRONTEND_INFO}"
echo ""
echo "🔍 快速测试:"
echo "  健康检查: curl http://localhost:8080/health"
echo "  工具列表: curl http://localhost:8080/tools"
echo ""
echo "⚙️  开发特性:"
echo "  日志级别: DEBUG (详细输出)"
echo "  MAA 模式: 模拟器 (无需真实设备)"
echo "  热重载: Rust 自动重编译"
echo ""
echo "⚡ 按 Ctrl+C 停止所有服务"

# 捕获中断信号
trap 'echo ""; echo "🛑 正在停止开发服务..."; kill $BACKEND_PID $FRONTEND_PID 2>/dev/null || true; echo "✅ 开发环境已停止"; exit 0' INT

# 保持脚本运行
wait