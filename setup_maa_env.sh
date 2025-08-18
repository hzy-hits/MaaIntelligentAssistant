#!/bin/bash
#
# MAA macOS 环境配置脚本
# 
# 此脚本配置环境变量以使用系统安装的 MAA.app

# MAA 应用路径
export MAA_APP_PATH="/Applications/MAA.app"

# MAA Core 库路径 (动态库目录)
export MAA_CORE_DIR="${MAA_APP_PATH}/Contents/Frameworks"

# MAA 资源路径 (游戏数据和配置)
export MAA_RESOURCE_DIR="${MAA_APP_PATH}/Contents/Resources/resource"

# ADB 路径 (Android 调试桥)
export MAA_ADB_PATH="${MAA_APP_PATH}/Contents/MacOS/adb"

# 动态库搜索路径 (让系统能找到 MAA 的 .dylib 文件)
export DYLD_LIBRARY_PATH="${MAA_CORE_DIR}:${DYLD_LIBRARY_PATH}"

# MAA Core 库文件的完整路径
export MAA_CORE_LIB="${MAA_CORE_DIR}/libMaaCore.dylib"

# 验证文件存在
if [[ ! -f "${MAA_CORE_LIB}" ]]; then
    echo "❌ 错误: MAA Core 库文件不存在: ${MAA_CORE_LIB}"
    echo "请确保 MAA.app 已正确安装在 /Applications/ 目录下"
    exit 1
fi

if [[ ! -f "${MAA_ADB_PATH}" ]]; then
    echo "❌ 错误: ADB 文件不存在: ${MAA_ADB_PATH}"
    exit 1
fi

if [[ ! -d "${MAA_RESOURCE_DIR}" ]]; then
    echo "❌ 错误: MAA 资源目录不存在: ${MAA_RESOURCE_DIR}"
    exit 1
fi

echo "✅ MAA macOS 环境配置成功!"
echo "📍 MAA Core 库: ${MAA_CORE_LIB}"
echo "📍 ADB 路径: ${MAA_ADB_PATH}"
echo "📍 资源目录: ${MAA_RESOURCE_DIR}"
echo ""
echo "🚀 现在可以运行: cargo run --features with-maa-core"
echo ""

# 显示库信息
echo "📊 MAA Core 库信息:"
file "${MAA_CORE_LIB}"
echo ""
echo "📏 库文件大小:"
ls -lh "${MAA_CORE_LIB}"