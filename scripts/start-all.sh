#!/bin/bash

# MAA 智能助手启动脚本
# 同时启动后端服务和前端界面

set -e

echo "启动 MAA 智能助手系统..."

# 检查是否在正确的目录
if [ ! -f "Cargo.toml" ]; then
    echo "错误：请在项目根目录运行此脚本"
    exit 1
fi

# 启动后端服务（使用单例服务器）
echo "启动 MAA 后端服务（单例模式）..."
cargo run --bin maa-server &
BACKEND_PID=$!

# 等待后端启动
echo "等待后端服务启动..."
sleep 3

# 检查后端是否启动成功
if curl -s http://localhost:8080/health > /dev/null; then
    echo "后端服务启动成功"
else
    echo "后端服务启动失败"
    kill $BACKEND_PID 2>/dev/null || true
    exit 1
fi

# 进入前端目录并启动
echo "启动前端界面..."
cd maa-chat-ui

# 安装依赖（如果需要）
if [ ! -d "node_modules" ]; then
    echo "安装前端依赖..."
    npm install
fi

# 启动前端
npm run dev &
FRONTEND_PID=$!

echo ""
echo "MAA 智能助手启动完成！"
echo ""
echo "后端服务: http://localhost:8080"
echo "前端界面: http://localhost:3000"
echo ""
echo "按 Ctrl+C 停止所有服务"

# 等待用户中断
trap 'echo ""; echo "正在停止服务..."; kill $BACKEND_PID $FRONTEND_PID 2>/dev/null || true; exit 0' INT

# 保持脚本运行
wait