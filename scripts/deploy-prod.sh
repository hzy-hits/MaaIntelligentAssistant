#!/bin/bash

# MAA 智能控制系统生产部署脚本
# 使用 Docker Compose 进行生产环境部署

set -e

echo "🚀 MAA 智能控制系统生产部署"
echo "============================"

# 检查是否在正确的目录
if [ ! -f "docker-compose.yml" ]; then
    echo "❌ 错误：请在项目根目录运行此脚本"
    exit 1
fi

# 检查必要工具
for cmd in docker docker-compose; do
    if ! command -v $cmd &> /dev/null; then
        echo "❌ 错误：未安装 $cmd"
        exit 1
    fi
done

# 检查 .env 文件
if [ ! -f ".env" ]; then
    echo "❌ 错误：.env 文件不存在"
    echo "💡 请先运行 ./scripts/setup-env.sh 创建环境配置"
    exit 1
fi

# 验证必要的环境变量
echo "🔍 验证环境配置..."
source .env

required_vars=("AI_PROVIDER" "AI_API_KEY" "AI_MODEL")
for var in "${required_vars[@]}"; do
    if [ -z "${!var}" ]; then
        echo "❌ 错误：缺少必要环境变量 $var"
        echo "请检查 .env 文件配置"
        exit 1
    fi
done

echo "✅ 环境配置验证通过"
echo ""

# 停止现有服务
echo "🛑 停止现有服务..."
docker-compose down --remove-orphans || true

# 清理旧镜像（可选）
if [ "$1" = "--clean" ]; then
    echo "🧹 清理旧镜像..."
    docker-compose down --volumes --remove-orphans || true
    docker system prune -f
    echo ""
fi

# 构建并启动服务
echo "📦 构建并启动服务..."
docker-compose up --build -d

# 等待服务启动
echo "⏳ 等待服务启动..."
sleep 10

# 健康检查
echo "💓 执行健康检查..."
max_attempts=30
attempt=1

while [ $attempt -le $max_attempts ]; do
    if curl -f -s http://localhost:${MAA_PORT:-8080}/health > /dev/null 2>&1; then
        echo "✅ 服务启动成功！"
        break
    fi
    
    if [ $attempt -eq $max_attempts ]; then
        echo "❌ 服务启动超时"
        echo "📋 查看日志:"
        docker-compose logs --tail=20
        exit 1
    fi
    
    echo "⏳ 尝试 $attempt/$max_attempts，等待服务响应..."
    sleep 2
    ((attempt++))
done

# 显示服务状态
echo ""
echo "📊 服务状态:"
docker-compose ps

echo ""
echo "🌐 服务端点:"
echo "- API 服务: http://localhost:${MAA_PORT:-8080}"
echo "- 健康检查: http://localhost:${MAA_PORT:-8080}/health"
echo "- Function Tools: http://localhost:${MAA_PORT:-8080}/tools"
echo "- API 文档: http://localhost:${MAA_PORT:-8080}/docs"

echo ""
echo "📋 管理命令:"
echo "- 查看日志: docker-compose logs -f"
echo "- 重启服务: docker-compose restart"
echo "- 停止服务: docker-compose down"
echo "- 更新服务: docker-compose up --build -d"

echo ""
echo "🎉 生产环境部署完成！"

# 测试基础功能
echo ""
echo "🧪 执行基础功能测试..."
echo "测试健康检查端点..."
health_response=$(curl -s http://localhost:${MAA_PORT:-8080}/health | jq -r '.status' 2>/dev/null || echo "error")

if [ "$health_response" = "healthy" ]; then
    echo "✅ 健康检查通过"
else
    echo "⚠️  健康检查异常: $health_response"
fi

echo "测试 Function Tools 端点..."
tools_count=$(curl -s http://localhost:${MAA_PORT:-8080}/tools | jq '.functions | length' 2>/dev/null || echo "0")

if [ "$tools_count" = "16" ]; then
    echo "✅ Function Tools 加载完成 ($tools_count 个工具)"
else
    echo "⚠️  Function Tools 异常: $tools_count 个工具"
fi

echo ""
echo "📈 监控建议:"
echo "- CPU 使用率监控"
echo "- 内存使用率监控" 
echo "- API 响应时间监控"
echo "- Docker 容器健康状态监控"
echo ""
echo "🔧 故障排除："
echo "- 查看实时日志: docker-compose logs -f maa-server"
echo "- 进入容器调试: docker-compose exec maa-server /bin/sh"
echo "- 重新构建: ./scripts/docker-build.sh && docker-compose up --build -d"