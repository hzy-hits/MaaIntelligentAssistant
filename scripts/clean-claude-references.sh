#!/bin/bash

# 清理Claude引用脚本
# 安全地重写最近的commit消息移除Claude痕迹

set -e

echo "🧹 开始清理Claude引用..."

# 检查当前状态
if [ -n "$(git status --porcelain)" ]; then
    echo "❌ 工作区不干净，请先提交或储藏更改"
    exit 1
fi

# 备份当前分支
current_branch=$(git rev-parse --abbrev-ref HEAD)
backup_branch="backup-before-claude-cleanup-$(date +%Y%m%d-%H%M%S)"
git branch "$backup_branch"
echo "📦 创建备份分支: $backup_branch"

# 获取需要清理的commit范围（最近8个commit应该足够了）
commits_to_clean=$(git rev-list --reverse HEAD~8..HEAD)

echo "🔍 检查需要清理的commit..."
for commit in $commits_to_clean; do
    if git show --format=%B $commit | grep -q -E "(Generated with.*Claude|Co-Authored-By.*Claude)"; then
        echo "  - $(git show --format="%h %s" -s $commit)"
    fi
done

echo ""
echo "✏️  开始清理commit消息..."

# 使用git filter-branch清理commit消息
git filter-branch -f --msg-filter '
    sed "/🤖 Generated with \[Claude Code\]/d" |
    sed "/Co-Authored-By: Claude/d" |
    sed "/^$/N;/^\n$/d"
' HEAD~8..HEAD

echo "🧹 清理git引用..."
git for-each-ref --format="%(refname)" refs/original/ | xargs -n 1 git update-ref -d 2>/dev/null || true

echo "✅ 验证清理结果..."
echo "📊 最近的commit消息:"
git log --oneline -8

echo ""
echo "🔍 检查是否还有Claude痕迹:"
claude_refs=$(git log --grep="Claude" --oneline HEAD~8..HEAD | wc -l)
if [ "$claude_refs" -gt 0 ]; then
    echo "⚠️  仍有 $claude_refs 个commit包含Claude痕迹"
    git log --grep="Claude" --oneline HEAD~8..HEAD
else
    echo "✅ 已移除所有Claude痕迹"
fi

echo ""
echo "🎉 Claude引用清理完成!"
echo "📦 备份分支: $backup_branch"
echo "💡 如需恢复，运行: git reset --hard $backup_branch"