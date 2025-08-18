#!/bin/bash

# MAA 智能控制系统 Docker 构建脚本（真实 MAA 模式）
# 构建支持真实 MAA Core 集成的 Docker 镜像

set -e

echo "🐳 MAA 智能控制系统 Docker 构建（真实模式）"
echo "=========================================="

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

# 检查本地 MAA 安装
MAA_APP_PATH="${MAA_APP_PATH:-/Applications/MAA.app}"
if [ ! -d "$MAA_APP_PATH" ]; then
    echo "❌ 错误：未找到 MAA.app"
    echo "请确保 MAA.app 安装在 $MAA_APP_PATH"
    echo "或设置环境变量 MAA_APP_PATH 指向正确路径"
    exit 1
fi

# 获取版本信息
VERSION=${1:-"latest"}
IMAGE_NAME="maa-intelligent-server"
FULL_IMAGE_NAME="${IMAGE_NAME}:${VERSION}-real"

echo "🏷️  构建镜像: ${FULL_IMAGE_NAME}"
echo "📁 MAA 路径: $MAA_APP_PATH"
echo ""

# 清理旧的构建缓存（可选）
if [ "$2" = "--clean" ]; then
    echo "🧹 清理 Docker 构建缓存..."
    docker builder prune -f
    echo ""
fi

# 准备 MAA Core 文件
echo "📦 准备 MAA Core 文件..."
MAA_CORE_DIR="$MAA_APP_PATH/Contents/Frameworks"
MAA_RESOURCE_PATH="$MAA_APP_PATH/Contents/Resources"

# 创建临时目录用于复制 MAA 文件
TEMP_MAA_DIR="./maa-core-temp"
rm -rf "$TEMP_MAA_DIR"
mkdir -p "$TEMP_MAA_DIR/lib" "$TEMP_MAA_DIR/resource"

# 复制 MAA 库文件
if [ -f "$MAA_CORE_DIR/libMaaCore.dylib" ]; then
    echo "📋 复制 MAA Core 库文件..."
    cp "$MAA_CORE_DIR"/*.dylib "$TEMP_MAA_DIR/lib/" 2>/dev/null || true
    cp "$MAA_CORE_DIR"/*.so "$TEMP_MAA_DIR/lib/" 2>/dev/null || true
else
    echo "❌ 错误：未找到 MAA Core 库文件"
    echo "预期路径: $MAA_CORE_DIR/libMaaCore.dylib"
    exit 1
fi

# 复制 MAA 资源文件
if [ -d "$MAA_RESOURCE_PATH" ]; then
    echo "📋 复制 MAA 资源文件..."
    cp -R "$MAA_RESOURCE_PATH"/* "$TEMP_MAA_DIR/resource/"
else
    echo "❌ 错误：未找到 MAA 资源目录"
    echo "预期路径: $MAA_RESOURCE_PATH"
    exit 1
fi

echo "✅ MAA 文件准备完成"
echo ""

# 构建镜像
echo "📦 开始构建 Docker 镜像（真实模式）..."
docker build \
    --tag "${FULL_IMAGE_NAME}" \
    --tag "${IMAGE_NAME}:latest-real" \
    --build-arg BUILD_MODE=real \
    --build-arg BUILDKIT_INLINE_CACHE=1 \
    .

# 清理临时目录
echo "🧹 清理临时文件..."
rm -rf "$TEMP_MAA_DIR"

echo ""
echo "✅ Docker 镜像构建完成（真实模式）！"
echo ""
echo "📊 镜像信息:"
docker images "${IMAGE_NAME}" | head -3

echo ""
echo "🚀 下一步操作:"
echo "1. 测试运行:   docker run --rm -p 8080:8080 -e MAA_DEVICE_ADDRESS=localhost:1717 ${FULL_IMAGE_NAME}"
echo "2. Docker Compose: MAA_BUILD_MODE=real docker-compose up"
echo "3. 推送镜像:   docker push ${FULL_IMAGE_NAME}"
echo ""
echo "💡 使用说明:"
echo "- 真实模式需要设备连接（PlayCover 或 Android 模拟器）"
echo "- 设备地址设置: -e MAA_DEVICE_ADDRESS=localhost:1717"
echo "- 查看日志: docker logs -f <container-id>"