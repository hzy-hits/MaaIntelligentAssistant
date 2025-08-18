#!/bin/bash

# MAA 智能控制系统 Docker 构建脚本
# 构建适合容器部署的 Docker 镜像（Stub模式）

set -e

echo "🐳 MAA 智能控制系统 Docker 构建"
echo "================================"

# 检查是否在正确的目录
if [ ! -f "Cargo.toml" ]; then
    echo "❌ 错误：请在项目根目录运行此脚本"
    exit 1
fi

# 检查 Docker 是否安装
if ! command -v docker &> /dev/null; then
    echo "❌ 错误：未安装 Docker"
    echo "请访问 https://docker.com 安装 Docker"
    exit 1
fi

# 检查 .env 文件
if [ ! -f ".env" ]; then
    echo "⚠️  警告：.env 文件不存在"
    echo "💡 创建示例 .env 文件..."
    cat > .env << 'EOF'
# MAA 智能控制系统环境配置

# 服务配置
MAA_PORT=8080
LOG_LEVEL=info
DEBUG_MODE=false

# AI 客户端配置（必需）
AI_PROVIDER=qwen
AI_API_KEY=your-api-key-here
AI_BASE_URL=https://dashscope.aliyuncs.com/compatible-mode/v1
AI_MODEL=qwen-plus-2025-04-28

# Web UI 配置
WEBUI_NAME=MAA智能助手
WEBUI_SECRET_KEY=maa-secret-key-change-in-production

# MAA 配置（容器模式）
MAA_BACKEND_MODE=stub
MAA_VERBOSE=true
EOF
    echo "✅ 已创建示例 .env 文件，请编辑后重新运行"
    exit 1
fi

# 获取版本信息
VERSION=${1:-"latest"}
IMAGE_NAME="maa-intelligent-server"
FULL_IMAGE_NAME="${IMAGE_NAME}:${VERSION}"

echo "🏷️  构建镜像: ${FULL_IMAGE_NAME}"
echo ""

# 清理旧的构建缓存（可选）
if [ "$2" = "--clean" ]; then
    echo "🧹 清理 Docker 构建缓存..."
    docker builder prune -f
    echo ""
fi

# 构建镜像
echo "📦 开始构建 Docker 镜像..."
docker build \
    --tag "${FULL_IMAGE_NAME}" \
    --tag "${IMAGE_NAME}:latest" \
    --build-arg BUILDKIT_INLINE_CACHE=1 \
    .

echo ""
echo "✅ Docker 镜像构建完成！"
echo ""
echo "📊 镜像信息:"
docker images "${IMAGE_NAME}" | head -2

echo ""
echo "🚀 下一步操作:"
echo "1. 测试运行:   docker run --rm -p 8080:8080 ${IMAGE_NAME}"
echo "2. Docker Compose: docker-compose up"
echo "3. 推送镜像:   docker push ${FULL_IMAGE_NAME}"
echo ""
echo "💡 端口映射:"
echo "- HTTP API: http://localhost:8080"
echo "- 健康检查: http://localhost:8080/health"
echo "- Function Tools: http://localhost:8080/tools"