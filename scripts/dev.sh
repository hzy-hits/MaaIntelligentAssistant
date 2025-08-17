#!/bin/bash

# MAA 开发模式启动脚本

set -e

echo "启动 MAA 开发环境..."

# 检查是否在正确的目录
if [ ! -f "Cargo.toml" ]; then
    echo "错误：请在项目根目录运行此脚本"
    exit 1
fi

# 启动后端（开发模式）
echo "启动后端（开发模式）..."
RUST_LOG=debug cargo run &
BACKEND_PID=$!

# 等待后端启动
echo "等待后端服务启动..."
sleep 3

# 检查后端健康状态
if curl -s http://localhost:8080/health > /dev/null; then
    echo "后端服务启动成功"
else
    echo "后端服务启动失败，请检查日志"
    kill $BACKEND_PID 2>/dev/null || true
    exit 1
fi

# 进入前端目录
cd maa-chat-ui

# 检查依赖
if [ ! -d "node_modules" ]; then
    echo "安装前端依赖..."
    npm install
fi

echo "启动前端开发服务器..."
npm run dev &
FRONTEND_PID=$!

echo ""
echo "开发环境启动完成！"
echo ""
echo "后端 API: http://localhost:8080"
echo "前端界面: http://localhost:3000"
echo "健康检查: http://localhost:8080/health"
echo "工具列表: http://localhost:8080/tools"
echo ""
echo "日志级别: DEBUG"
echo "热重载: 已启用"
echo ""
echo "按 Ctrl+C 停止所有服务"

# 捕获中断信号
trap 'echo ""; echo "正在停止开发服务..."; kill $BACKEND_PID $FRONTEND_PID 2>/dev/null || true; exit 0' INT

# 保持脚本运行
wait