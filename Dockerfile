# 多阶段构建
FROM node:18-alpine as frontend-builder

# 构建前端
WORKDIR /app/frontend
COPY maa-chat-ui/package*.json ./
RUN npm ci --only=production
COPY maa-chat-ui/ ./
RUN npm run build

# Rust 后端构建
FROM rust:1.70-slim as backend-builder

# 安装系统依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 设置工作目录
WORKDIR /app

# 复制配置文件
COPY Cargo.toml Cargo.lock ./

# 复制源代码
COPY src ./src

# 构建项目
RUN cargo build --release

# 运行阶段
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 创建应用用户
RUN useradd -m -u 1000 maa && mkdir -p /app/data /app/static && chown -R maa:maa /app

# 设置工作目录
WORKDIR /app

# 从构建阶段复制文件
COPY --from=backend-builder /app/target/release/maa-intelligent-server /app/maa-server
COPY --from=frontend-builder /app/frontend/dist /app/static

# 复制配置文件
COPY --chown=maa:maa .env* ./

# 设置权限
RUN chmod +x /app/maa-server

# 切换到应用用户
USER maa

# 暴露端口
EXPOSE 8080

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# 启动应用
CMD ["/app/maa-server"]