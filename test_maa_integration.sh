#!/bin/bash
#
# MAA Core 集成测试脚本
# 
# 此脚本验证 MAA.app 的库文件和环境配置

set -e  # 遇到错误立即退出

echo "🔍 MAA Core macOS 集成检查"
echo "================================"

# 检查 .env 文件
if [[ ! -f ".env" ]]; then
    echo "❌ 错误: .env 文件不存在"
    exit 1
fi

echo "✅ .env 文件存在"

# 读取环境变量
export $(grep -v '^#' .env | xargs)

echo ""
echo "📋 环境变量配置:"
echo "MAA_APP_PATH: ${MAA_APP_PATH:-未设置}"
echo "MAA_CORE_DIR: ${MAA_CORE_DIR:-未设置}"
echo "MAA_CORE_LIB: ${MAA_CORE_LIB:-未设置}"
echo "MAA_RESOURCE_PATH: ${MAA_RESOURCE_PATH:-未设置}"
echo "MAA_ADB_PATH: ${MAA_ADB_PATH:-未设置}"

echo ""
echo "🔍 验证MAA文件:"

# 检查MAA应用
if [[ ! -d "${MAA_APP_PATH}" ]]; then
    echo "❌ MAA.app 不存在: ${MAA_APP_PATH}"
    exit 1
fi
echo "✅ MAA.app 存在: ${MAA_APP_PATH}"

# 检查MAA Core动态库
if [[ ! -f "${MAA_CORE_LIB}" ]]; then
    echo "❌ MAA Core 库不存在: ${MAA_CORE_LIB}"
    exit 1
fi
echo "✅ MAA Core 库存在: ${MAA_CORE_LIB}"

# 检查库文件架构
echo ""
echo "📊 MAA Core 库信息:"
file "${MAA_CORE_LIB}"
echo ""
echo "📏 库文件大小:"
ls -lh "${MAA_CORE_LIB}"

# 检查ADB
if [[ ! -f "${MAA_ADB_PATH}" ]]; then
    echo "❌ ADB 不存在: ${MAA_ADB_PATH}"
    exit 1
fi
echo "✅ ADB 存在: ${MAA_ADB_PATH}"

# 检查资源目录
if [[ ! -d "${MAA_RESOURCE_PATH}" ]]; then
    echo "❌ 资源目录不存在: ${MAA_RESOURCE_PATH}"
    exit 1
fi
echo "✅ 资源目录存在: ${MAA_RESOURCE_PATH}"

# 检查关键资源文件
resource_files=("config.json" "battle_data.json" "stages.json" "recruitment.json")
for file in "${resource_files[@]}"; do
    if [[ ! -f "${MAA_RESOURCE_PATH}/${file}" ]]; then
        echo "❌ 缺少资源文件: ${file}"
        exit 1
    fi
done
echo "✅ 关键资源文件完整"

# 检查依赖库
deps=("libopencv_world4.408.dylib" "libonnxruntime.1.18.0.dylib" "libfastdeploy_ppocr.dylib")
for dep in "${deps[@]}"; do
    if [[ ! -f "${MAA_CORE_DIR}/${dep}" ]]; then
        echo "❌ 缺少依赖库: ${dep}"
        exit 1
    fi
done
echo "✅ 依赖库完整"

echo ""
echo "🔧 编译测试:"

# 测试编译
echo "测试编译 (stub 模式)..."
if cargo check; then
    echo "✅ Stub 模式编译成功"
else
    echo "❌ Stub 模式编译失败"
    exit 1
fi

echo ""
echo "测试编译 (带 MAA Core)..."
if cargo check --features with-maa-core; then
    echo "✅ MAA Core 模式编译成功"
else
    echo "❌ MAA Core 模式编译失败"
    exit 1
fi

echo ""
echo "🚀 启动测试:"

# 测试启动单例服务器 (后台运行5秒)
echo "测试启动单例服务器..."
timeout 5s cargo run --bin maa-server --features with-maa-core &
server_pid=$!

sleep 2

# 检查服务器是否正在运行
if kill -0 $server_pid 2>/dev/null; then
    echo "✅ 单例服务器启动成功"
    kill $server_pid 2>/dev/null || true
else
    echo "❌ 单例服务器启动失败"
fi

echo ""
echo "🎉 MAA Core 集成检查完成!"
echo ""
echo "📝 下一步操作:"
echo "1. 启动开发模式: ./scripts/dev.sh"
echo "2. 启动生产模式: cargo run --bin maa-server --features with-maa-core"
echo "3. 测试Function Calling: curl http://localhost:8080/tools"
echo ""
echo "💡 如果遇到问题:"
echo "- 检查 MAA.app 是否正确安装"
echo "- 确保 .env 配置正确"
echo "- 查看日志: RUST_LOG=debug cargo run --features with-maa-core"