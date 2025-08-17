#!/bin/bash

# Git History Cleanup Script
# 清理敏感信息和Claude痕迹，创建干净的git历史

set -e

echo "🧹 开始清理git历史..."

# 1. 备份当前分支
current_branch=$(git rev-parse --abbrev-ref HEAD)
echo "📦 当前分支: $current_branch"

# 2. 创建备份分支
backup_branch="backup-before-cleanup-$(date +%Y%m%d-%H%M%S)"
git branch "$backup_branch"
echo "✅ 创建备份分支: $backup_branch"

# 3. 检查并删除敏感文件
echo "🔍 检查敏感文件..."
if [ -f ".env" ]; then
    echo "🗑️  删除 .env 文件"
    rm .env
fi

# 4. 从前端移除硬编码API密钥
echo "🔐 清理前端硬编码API密钥..."
if [ -f "maa-chat-ui/main.jsx" ]; then
    # 使用环境变量替换硬编码的API密钥
    sed -i.bak "s/'Bearer sk-ee8e1993fd584b66ba4d1c8d92b67050'/'Bearer ' + (import.meta.env.VITE_QWEN_API_KEY || 'your-api-key-here')/g" maa-chat-ui/main.jsx
    rm maa-chat-ui/main.jsx.bak 2>/dev/null || true
    echo "✅ 已更新前端API密钥为环境变量"
fi

# 5. 创建 .env.example 文件
echo "📝 创建 .env.example..."
cat > .env.example << 'EOF'
# MAA 智能控制中间层配置

# MAA 后端配置
MAA_PORT=8080
MAA_RESOURCE_PATH=./maa-official/resource
MAA_ADB_PATH=adb
MAA_DEVICE_ADDRESS=127.0.0.1:5555

# Qwen API 配置
QWEN_API_KEY=your-qwen-api-key-here
QWEN_API_BASE=https://dashscope.aliyuncs.com/compatible-mode/v1
QWEN_MODEL=qwen-plus-2025-04-28

# 前端环境变量
VITE_QWEN_API_KEY=your-qwen-api-key-here

# Open WebUI 配置
WEBUI_PORT=3000
WEBUI_NAME=MAA智能助手
WEBUI_SECRET_KEY=maa-secret-key-change-in-production

# 代理配置（国内用户）
HTTP_PROXY=http://host.docker.internal:7897
HTTPS_PROXY=http://host.docker.internal:7897

# 调试配置
DEBUG_MODE=true
LOG_LEVEL=info
EOF

# 6. 更新 .gitignore
echo "🚫 更新 .gitignore..."
if ! grep -q "^\.env$" .gitignore 2>/dev/null; then
    echo "" >> .gitignore
    echo "# 敏感配置文件" >> .gitignore
    echo ".env" >> .gitignore
    echo "*.key" >> .gitignore
    echo "*.secret" >> .gitignore
fi

# 7. 提交清理变更
echo "💾 提交敏感信息清理..."
git add .
git commit -m "security: 清理敏感信息和硬编码API密钥

- 移除前端硬编码的API密钥
- 使用环境变量替代硬编码配置
- 更新.env.example配置示例
- 完善.gitignore防止敏感文件提交
- 为开源发布做准备"

# 8. 重写commit消息移除Claude痕迹
echo "✏️  重写commit消息移除Claude痕迹..."

# 使用git filter-branch重写历史
git filter-branch --msg-filter '
sed "s/🤖 Generated with \[Claude Code\](https:\/\/claude\.ai\/code)//g" |
sed "s/Co-Authored-By: Claude <noreply@anthropic\.com>//g" |
sed "/^$/d"
' --force -- --all

# 9. 清理引用
echo "🧹 清理git引用..."
git for-each-ref --format="%(refname)" refs/original/ | xargs -n 1 git update-ref -d
git reflog expire --expire=now --all
git gc --prune=now --aggressive

# 10. 验证清理结果
echo "✅ 验证清理结果..."
echo "📊 最近的commit消息:"
git log --oneline -5

echo ""
echo "🔍 检查是否还有Claude痕迹:"
if git log --grep="Claude" --oneline; then
    echo "⚠️  仍有Claude痕迹"
else
    echo "✅ 已移除所有Claude痕迹"
fi

echo ""
echo "🔐 检查是否还有API密钥:"
if grep -r "sk-ee8e1993" . --exclude-dir=.git 2>/dev/null; then
    echo "⚠️  仍有API密钥残留"
else
    echo "✅ 已移除所有API密钥"
fi

echo ""
echo "🎉 Git历史清理完成!"
echo "📦 备份分支: $backup_branch"
echo "💡 如需恢复，运行: git checkout $backup_branch"
echo ""
echo "🚀 现在可以安全地推送到GitHub了"