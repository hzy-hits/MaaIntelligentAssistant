#!/bin/bash

# MAA 智能控制系统环境配置脚本
# 交互式创建 .env 配置文件

set -e

echo "⚙️  MAA 智能控制系统环境配置"
echo "============================="

# 检查是否已存在 .env 文件
if [ -f ".env" ]; then
    echo "⚠️  .env 文件已存在"
    read -p "是否要重新配置？(y/N): " overwrite
    if [[ ! $overwrite =~ ^[Yy]$ ]]; then
        echo "保持现有配置，退出"
        exit 0
    fi
    echo "备份现有配置到 .env.backup.$(date +%Y%m%d-%H%M%S)"
    cp .env .env.backup.$(date +%Y%m%d-%H%M%S)
fi

echo ""
echo "🔧 开始配置环境变量..."
echo ""

# 服务配置
echo "📡 服务配置"
echo "----------"
read -p "服务端口 [8080]: " MAA_PORT
MAA_PORT=${MAA_PORT:-8080}

read -p "日志级别 (debug/info/warn/error) [info]: " LOG_LEVEL
LOG_LEVEL=${LOG_LEVEL:-info}

read -p "开启调试模式？(y/N): " DEBUG_MODE
if [[ $DEBUG_MODE =~ ^[Yy]$ ]]; then
    DEBUG_MODE=true
else
    DEBUG_MODE=false
fi

echo ""
echo "🤖 AI 客户端配置"
echo "---------------"
echo "支持的 AI 提供商: openai, azure, qwen, kimi, ollama"
read -p "AI 提供商 [qwen]: " AI_PROVIDER
AI_PROVIDER=${AI_PROVIDER:-qwen}

# 根据提供商设置默认值
case $AI_PROVIDER in
    openai)
        default_base_url="https://api.openai.com/v1"
        default_model="gpt-4-turbo-preview"
        ;;
    azure)
        default_base_url="https://your-resource.openai.azure.com"
        default_model="gpt-4"
        ;;
    qwen)
        default_base_url="https://dashscope.aliyuncs.com/compatible-mode/v1"
        default_model="qwen-plus-2025-04-28"
        ;;
    kimi)
        default_base_url="https://api.moonshot.cn/v1"
        default_model="moonshot-v1-8k"
        ;;
    ollama)
        default_base_url="http://localhost:11434/v1"
        default_model="llama2"
        ;;
    *)
        default_base_url=""
        default_model="gpt-4-turbo-preview"
        ;;
esac

echo ""
read -p "API 密钥（必需）: " AI_API_KEY
if [ -z "$AI_API_KEY" ]; then
    echo "❌ 错误：API 密钥不能为空"
    exit 1
fi

read -p "API 基础 URL [$default_base_url]: " AI_BASE_URL
AI_BASE_URL=${AI_BASE_URL:-$default_base_url}

read -p "模型名称 [$default_model]: " AI_MODEL
AI_MODEL=${AI_MODEL:-$default_model}

read -p "采样温度 (0.0-2.0) [0.7]: " AI_TEMPERATURE
AI_TEMPERATURE=${AI_TEMPERATURE:-0.7}

read -p "最大Token数 [4000]: " AI_MAX_TOKENS
AI_MAX_TOKENS=${AI_MAX_TOKENS:-4000}

echo ""
echo "🌐 Web UI 配置"
echo "-------------"
read -p "界面名称 [MAA智能助手]: " WEBUI_NAME
WEBUI_NAME=${WEBUI_NAME:-"MAA智能助手"}

read -p "安全密钥（用于会话加密）: " WEBUI_SECRET_KEY
if [ -z "$WEBUI_SECRET_KEY" ]; then
    # 生成随机密钥
    WEBUI_SECRET_KEY="maa-$(openssl rand -hex 16)"
    echo "自动生成密钥: $WEBUI_SECRET_KEY"
fi

echo ""
echo "🎮 MAA 配置"
echo "----------"
echo "运行模式："
echo "1. stub  - 模拟模式（适合开发和容器部署）"
echo "2. real  - 真实模式（需要本地 MAA Core）"
read -p "选择运行模式 (1/2) [1]: " mode_choice
if [ "$mode_choice" = "2" ]; then
    MAA_BACKEND_MODE=real
    
    echo ""
    echo "🔧 MAA Core 配置"
    read -p "MAA 应用路径 [/Applications/MAA.app]: " MAA_APP_PATH
    MAA_APP_PATH=${MAA_APP_PATH:-"/Applications/MAA.app"}
    
    read -p "设备地址 (localhost:1717 for PlayCover, 127.0.0.1:5555 for Android): " MAA_DEVICE_ADDRESS
    MAA_DEVICE_ADDRESS=${MAA_DEVICE_ADDRESS:-"localhost:1717"}
    
    MAA_CORE_DIR="$MAA_APP_PATH/Contents/Frameworks"
    MAA_RESOURCE_PATH="$MAA_APP_PATH/Contents/Resources"
    MAA_ADB_PATH="$MAA_APP_PATH/Contents/MacOS/adb"
    MAA_CORE_LIB="$MAA_CORE_DIR/libMaaCore.dylib"
else
    MAA_BACKEND_MODE=stub
fi

read -p "开启详细日志？(Y/n): " MAA_VERBOSE
if [[ $MAA_VERBOSE =~ ^[Nn]$ ]]; then
    MAA_VERBOSE=false
else
    MAA_VERBOSE=true
fi

# 网络代理配置（可选）
echo ""
echo "🌐 网络代理配置（可选）"
echo "-------------------"
read -p "HTTP 代理 (如: http://proxy.example.com:8080): " HTTP_PROXY
read -p "HTTPS 代理 (如: http://proxy.example.com:8080): " HTTPS_PROXY

# 生成 .env 文件
echo ""
echo "📝 生成 .env 文件..."

cat > .env << EOF
# MAA 智能控制中间层环境配置
# 生成时间: $(date)

# =====================================
# 服务配置
# =====================================
MAA_PORT=$MAA_PORT
LOG_LEVEL=$LOG_LEVEL
DEBUG_MODE=$DEBUG_MODE

# =====================================
# AI 客户端配置
# =====================================
AI_PROVIDER=$AI_PROVIDER
AI_API_KEY=$AI_API_KEY
AI_BASE_URL=$AI_BASE_URL
AI_MODEL=$AI_MODEL
AI_TEMPERATURE=$AI_TEMPERATURE
AI_MAX_TOKENS=$AI_MAX_TOKENS

# =====================================
# Web UI 配置  
# =====================================
WEBUI_PORT=3000
WEBUI_NAME=$WEBUI_NAME
WEBUI_SECRET_KEY=$WEBUI_SECRET_KEY

# =====================================
# MAA 配置
# =====================================
MAA_BACKEND_MODE=$MAA_BACKEND_MODE
MAA_VERBOSE=$MAA_VERBOSE
MAA_FORCE_STUB=false
EOF

# 如果是真实模式，添加 MAA Core 配置
if [ "$MAA_BACKEND_MODE" = "real" ]; then
    cat >> .env << EOF

# MAA Core 真实模式配置
MAA_APP_PATH=$MAA_APP_PATH
MAA_DEVICE_ADDRESS=$MAA_DEVICE_ADDRESS
MAA_CORE_DIR=$MAA_CORE_DIR
MAA_RESOURCE_PATH=$MAA_RESOURCE_PATH
MAA_ADB_PATH=$MAA_ADB_PATH
MAA_CORE_LIB=$MAA_CORE_LIB
DYLD_LIBRARY_PATH=$MAA_CORE_DIR
EOF
fi

# 添加网络代理配置（如果有）
if [ ! -z "$HTTP_PROXY" ] || [ ! -z "$HTTPS_PROXY" ]; then
    cat >> .env << EOF

# =====================================
# 网络代理配置
# =====================================
EOF
    [ ! -z "$HTTP_PROXY" ] && echo "HTTP_PROXY=$HTTP_PROXY" >> .env
    [ ! -z "$HTTPS_PROXY" ] && echo "HTTPS_PROXY=$HTTPS_PROXY" >> .env
fi

echo "✅ .env 文件创建完成！"
echo ""

# 验证配置
echo "🔍 配置验证："
echo "- 服务端口: $MAA_PORT"
echo "- AI 提供商: $AI_PROVIDER ($AI_MODEL)"
echo "- MAA 模式: $MAA_BACKEND_MODE"
echo "- 日志级别: $LOG_LEVEL"

if [ "$MAA_BACKEND_MODE" = "real" ]; then
    echo "- MAA 路径: $MAA_APP_PATH"
    echo "- 设备地址: $MAA_DEVICE_ADDRESS"
fi

echo ""
echo "🚀 下一步操作："
echo "1. 开发模式: ./scripts/dev.sh"
echo "2. 生产部署: ./scripts/deploy-prod.sh"
echo "3. Docker 构建: ./scripts/docker-build.sh"

if [ "$MAA_BACKEND_MODE" = "real" ]; then
    echo ""
    echo "⚠️  真实模式注意事项："
    echo "- 确保 MAA.app 已正确安装"
    echo "- 确保设备连接正常（ADB 或 PlayCover）"
    echo "- 运行前执行: source ./setup_maa_env.sh"
fi

echo ""
echo "🎉 环境配置完成！"