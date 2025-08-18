#!/bin/bash

# MAA 本地部署脚本
# 部署后端服务和前端界面

set -e

echo "MAA 本地部署"
echo "==============="

# 检查是否在正确的目录
if [ ! -f "Cargo.toml" ]; then
    echo "错误：请在项目根目录运行此脚本"
    exit 1
fi

# 检查必要工具
for cmd in cargo node npm; do
    if ! command -v $cmd &> /dev/null; then
        echo "错误：未安装 $cmd"
        case $cmd in
            cargo)
                echo "请安装 Rust: https://rustup.rs/"
                ;;
            node|npm)
                echo "请安装 Node.js: https://nodejs.org/"
                ;;
        esac
        exit 1
    fi
done

# 检查 .env 文件
if [ ! -f ".env" ]; then
    echo "错误：.env 文件不存在"
    echo "请先运行 ./scripts/setup-env.sh 创建环境配置"
    exit 1
fi

# 验证环境配置
echo "验证环境配置..."
source .env

required_vars=("AI_PROVIDER" "AI_API_KEY" "AI_MODEL")
for var in "${required_vars[@]}"; do
    if [ -z "${!var}" ]; then
        echo "错误：缺少必要环境变量 $var"
        echo "请检查 .env 文件配置"
        exit 1
    fi
done

# 检查MAA模式配置
if [ "${MAA_BACKEND_MODE}" = "real" ]; then
    echo "检测到真实MAA模式配置"
    
    # 检查 MAA 安装
    MAA_APP_PATH="${MAA_APP_PATH:-/Applications/MAA.app}"
    if [ ! -d "$MAA_APP_PATH" ]; then
        echo "错误：未找到 MAA.app"
        echo "请确保 MAA.app 安装在 $MAA_APP_PATH"
        exit 1
    fi
    
    # 验证 MAA Core 库文件
    MAA_CORE_LIB="${MAA_CORE_LIB:-$MAA_APP_PATH/Contents/Frameworks/libMaaCore.dylib}"
    if [ ! -f "$MAA_CORE_LIB" ]; then
        echo "错误：未找到 MAA Core 库文件"
        echo "预期路径: $MAA_CORE_LIB"
        exit 1
    fi
    
    echo "MAA Core 配置验证通过"
    echo "   MAA 应用: $MAA_APP_PATH"
    echo "   库文件: $MAA_CORE_LIB"
    echo "   设备地址: ${MAA_DEVICE_ADDRESS:-localhost:1717}"
    
    # 设置环境变量
    echo "配置 MAA 环境变量..."
    source ./setup_maa_env.sh
    
    BUILD_FEATURES="--features with-maa-core"
else
    echo "使用 Stub 模式（开发测试）"
    BUILD_FEATURES=""
fi

echo "环境配置验证通过"
echo ""

# 停止现有进程（如果有）
echo "停止现有服务..."
pkill -f "maa-server" || true
pkill -f "vite" || true
sleep 2

# 构建项目
echo "构建项目..."
if [ "${MAA_BACKEND_MODE}" = "real" ]; then
    echo "   使用真实 MAA Core 集成..."
    cargo build --release --bin maa-server $BUILD_FEATURES
else
    echo "   使用 Stub 模式..."
    cargo build --release --bin maa-server
fi

echo "后端构建完成"

# 构建前端
echo "构建前端..."
cd maa-chat-ui
if [ ! -d "node_modules" ]; then
    echo "   安装前端依赖..."
    npm install
fi
echo "   构建前端资源..."
npm run build
cd ..

echo "前端构建完成"
echo ""

# 创建运行目录
mkdir -p logs data
mkdir -p maa-chat-ui/dist

# 启动服务
echo "启动 MAA 服务器..."
echo "   后端端口: ${MAA_PORT:-8080}"
echo "   模式: ${MAA_BACKEND_MODE:-stub}"
echo "   日志级别: ${RUST_LOG:-info}"
echo ""

# 后台启动后端服务
RUST_LOG=${RUST_LOG:-info} nohup ./target/release/maa-server > logs/maa-server.log 2>&1 &
SERVER_PID=$!

# 启动前端开发服务器
cd maa-chat-ui
nohup npm run dev > ../logs/frontend.log 2>&1 &
FRONTEND_PID=$!
cd ..

echo "服务信息:"
echo "   后端进程 ID: $SERVER_PID"
echo "   前端进程 ID: $FRONTEND_PID"
echo "   后端日志: logs/maa-server.log"
echo "   前端日志: logs/frontend.log"

# 等待服务启动
echo "等待服务启动..."
sleep 8

# 后端健康检查
echo "后端健康检查..."
max_attempts=15
attempt=1

while [ $attempt -le $max_attempts ]; do
    if curl -f -s http://localhost:${MAA_PORT:-8080}/health > /dev/null 2>&1; then
        echo "后端启动成功"
        break
    fi
    
    if [ $attempt -eq $max_attempts ]; then
        echo "后端启动超时"
        echo "查看日志:"
        tail -20 logs/maa-server.log
        exit 1
    fi
    
    echo "尝试 $attempt/$max_attempts，等待后端响应..."
    sleep 2
    ((attempt++))
done

# 前端健康检查
echo "前端健康检查..."
max_attempts=10
attempt=1

while [ $attempt -le $max_attempts ]; do
    if curl -f -s http://localhost:3000 > /dev/null 2>&1; then
        echo "前端启动成功"
        break
    fi
    
    if [ $attempt -eq $max_attempts ]; then
        echo "前端启动超时，但后端正常运行"
        break
    fi
    
    echo "尝试 $attempt/$max_attempts，等待前端响应..."
    sleep 2
    ((attempt++))
done

echo ""
echo "服务端点:"
echo "- 前端界面: http://localhost:3000"
echo "- API 服务: http://localhost:${MAA_PORT:-8080}"
echo "- 健康检查: http://localhost:${MAA_PORT:-8080}/health"
echo "- Function Tools: http://localhost:${MAA_PORT:-8080}/tools"

echo ""
echo "MAA 运行模式: ${MAA_BACKEND_MODE:-stub}"
if [ "${MAA_BACKEND_MODE}" = "real" ]; then
    echo "- 设备地址: ${MAA_DEVICE_ADDRESS:-localhost:1717}"
    echo "- MAA Core: $MAA_CORE_LIB"
fi

echo ""
echo "管理命令:"
echo "- 查看后端日志: tail -f logs/maa-server.log"
echo "- 查看前端日志: tail -f logs/frontend.log"
echo "- 重启服务: ./scripts/deploy-local.sh"
echo "- 停止服务: pkill -f maa-server && pkill -f vite"
echo "- 服务状态: ps aux | grep -E '(maa-server|vite)'"

echo ""
echo "本地部署完成"
echo "后端进程 ID: $SERVER_PID"
echo "前端进程 ID: $FRONTEND_PID"

# 测试基础功能
echo ""
echo "执行基础功能测试..."
echo "测试健康检查端点..."
health_response=$(curl -s http://localhost:${MAA_PORT:-8080}/health | jq -r '.status' 2>/dev/null || echo "error")

if [ "$health_response" = "healthy" ]; then
    echo "健康检查通过"
else
    echo "健康检查异常: $health_response"
fi

echo "测试 Function Tools 端点..."
tools_count=$(curl -s http://localhost:${MAA_PORT:-8080}/tools | jq '.functions | length' 2>/dev/null || echo "0")

if [ "$tools_count" = "16" ]; then
    echo "Function Tools 加载完成 ($tools_count 个工具)"
else
    echo "Function Tools 异常: $tools_count 个工具"
fi

echo ""
echo "故障排除:"
echo "- 查看后端日志: tail -f logs/maa-server.log"
echo "- 查看前端日志: tail -f logs/frontend.log"
echo "- 检查端口占用: lsof -i :${MAA_PORT:-8080} && lsof -i :3000"
echo "- 重新构建: cargo build --release --bin maa-server $BUILD_FEATURES"

if [ "${MAA_BACKEND_MODE}" = "real" ]; then
    echo ""
    echo "MAA 设备连接:"
    echo "- PlayCover: 确保明日方舟在 PlayCover 中运行"
    echo "- Android: 确保模拟器启动并连接 ADB"
    echo "- 测试连接: adb devices"
fi

echo ""
echo "使用说明:"
echo "1. 打开浏览器访问 http://localhost:3000"
echo "2. 在聊天界面中输入指令，如：帮我截个图"
echo "3. AI 助手会智能理解并执行 MAA 操作"