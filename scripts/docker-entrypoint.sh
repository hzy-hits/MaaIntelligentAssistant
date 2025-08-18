#!/bin/bash

# MAA Docker 容器启动脚本
# 自动检测运行模式并设置相应环境

set -e

echo "🐳 MAA 智能控制系统 Docker 容器启动"
echo "=================================="

# 检测运行模式
if [ -f "/app/maa-core/lib/libMaaCore.dylib" ] || [ -f "/app/maa-core/lib/libMaaCore.so" ]; then
    echo "🔍 检测到 MAA Core 库文件，使用真实模式"
    export MAA_BACKEND_MODE=real
    
    # 检查必要文件
    if [ ! -d "/app/maa-core/resource" ]; then
        echo "❌ 错误：MAA 资源目录不存在 /app/maa-core/resource"
        echo "请确保挂载了完整的 MAA 资源目录"
        exit 1
    fi
    
    # 设置库路径
    if [ -f "/app/maa-core/lib/libMaaCore.dylib" ]; then
        export MAA_CORE_LIB="/app/maa-core/lib/libMaaCore.dylib"
        export DYLD_LIBRARY_PATH="/app/maa-core/lib:$DYLD_LIBRARY_PATH"
    elif [ -f "/app/maa-core/lib/libMaaCore.so" ]; then
        export MAA_CORE_LIB="/app/maa-core/lib/libMaaCore.so"
        export LD_LIBRARY_PATH="/app/maa-core/lib:$LD_LIBRARY_PATH"
    fi
    
    echo "✅ MAA Core 配置："
    echo "   库文件: $MAA_CORE_LIB"
    echo "   资源路径: $MAA_RESOURCE_PATH"
    echo "   设备地址: ${MAA_DEVICE_ADDRESS:-localhost:1717}"
    
else
    echo "🔍 未检测到 MAA Core 库文件，使用 Stub 模式"
    export MAA_BACKEND_MODE=stub
fi

# 显示环境信息
echo ""
echo "📊 运行环境："
echo "   运行模式: $MAA_BACKEND_MODE"
echo "   服务端口: ${MAA_PORT:-8080}"
echo "   AI 提供商: ${AI_PROVIDER:-qwen}"
echo "   日志级别: ${RUST_LOG:-info}"

# 验证 AI 配置
if [ -z "$AI_API_KEY" ]; then
    echo "⚠️  警告：未设置 AI_API_KEY，Function Calling 功能将受限"
fi

echo ""
echo "🚀 启动 MAA 服务器..."
echo ""

# 启动应用
exec /app/maa-server