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
COPY build.rs ./

# 复制源代码
COPY src ./src

# Docker构建：仅支持Stub模式（开发环境）
# 真实MAA集成请使用本地部署 ./scripts/deploy-local.sh
RUN cargo build --release --bin maa-server

# 运行阶段（开发环境专用）
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
COPY --from=backend-builder /app/target/release/maa-server /app/maa-server
COPY --from=frontend-builder /app/frontend/dist /app/static

# 复制配置文件和文档
COPY --chown=maa:maa .env* ./
COPY --chown=maa:maa config/ ./config/
COPY --chown=maa:maa docs/ ./docs/

# 设置权限
RUN chmod +x /app/maa-server

# 切换到应用用户
USER maa

# 暴露端口
EXPOSE 8080

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# 设置环境变量（Stub模式）
ENV MAA_BACKEND_MODE=stub
ENV RUST_LOG=info

# 启动应用
CMD ["/app/maa-server"]