# 安装和部署指南

## 系统要求

- Rust 1.70+ 
- MAA Core 库 (通过 git submodule 集成)
- 操作系统: Windows, macOS, Linux

## 安装步骤

### 1. 克隆项目

```bash
git clone --recursive https://github.com/your-org/maa-remote-server.git
cd maa-remote-server
```

### 2. 初始化 MAA 官方子模块

```bash
# 如果没有使用 --recursive 选项
git submodule update --init --recursive
```

### 3. 安装 Rust 依赖

```bash
cargo build
```

### 4. 配置环境

创建 `.env` 文件：

```env
# 服务器端口
PORT=8080

# MAA 配置
MAA_RESOURCE_PATH=./maa-official/resource
MAA_LOG_LEVEL=info

# AI 配置 (可选)
OPENAI_API_KEY=your_openai_key
CLAUDE_API_KEY=your_claude_key
```

### 5. 启动服务

```bash
# 开发模式
cargo run

# 生产模式
cargo run --release

# 后台运行
nohup cargo run --release > maa-server.log 2>&1 &
```

## 验证安装

### 检查服务状态

```bash
curl http://localhost:8080/health
```

预期返回：
```json
{
  "status": "ok",
  "version": "0.1.0",
  "modules": {
    "maa_adapter": "ready",
    "operator_manager": "ready",
    "copilot_matcher": "ready"
  }
}
```

### 获取可用工具

```bash
curl http://localhost:8080/tools
```

### 测试函数调用

```bash
curl -X POST http://localhost:8080/call \
  -H "Content-Type: application/json" \
  -d '{
    "function_call": {
      "name": "maa_status",
      "arguments": {}
    }
  }'
```

## Docker 部署

### 构建镜像

```bash
docker build -t maa-intelligent-server .
```

### 运行容器

```bash
docker run -d \
  --name maa-server \
  -p 8080:8080 \
  -v $(pwd)/config:/app/config \
  maa-intelligent-server
```

## 故障排除

### 常见问题

#### 1. MAA Core 编译失败

```bash
# 检查子模块状态
git submodule status

# 重新初始化子模块
git submodule update --init --recursive --force
```

#### 2. FFI 绑定错误

确保 MAA Core 库正确编译：

```bash
# 检查 MAA 库文件
ls maa-official/src/Rust/target/

# 重新构建
cd maa-official
./build.sh  # Linux/macOS
# 或
./build.cmd  # Windows
```

#### 3. 端口被占用

```bash
# 检查端口使用
lsof -i :8080

# 使用其他端口
PORT=3000 cargo run
```

#### 4. 权限问题

确保对以下目录有读写权限：
- `./cache/` - 缓存目录
- `./logs/` - 日志目录
- `./config/` - 配置目录

### 日志调试

启用详细日志：

```bash
RUST_LOG=debug cargo run
```

查看特定模块日志：

```bash
RUST_LOG=maa_intelligent_server::maa_adapter=trace cargo run
```

## 性能优化

### 生产环境配置

```toml
# Cargo.toml
[profile.release]
lto = true
codegen-units = 1
panic = "abort"
```

### 内存优化

在 `.env` 中设置：

```env
# 限制缓存大小
CACHE_MAX_SIZE=100MB
CACHE_TTL=3600

# 数据库优化
SLED_CACHE_CAPACITY=1000000
```

### 并发优化

```env
# 工作线程数
TOKIO_WORKER_THREADS=4

# 连接池大小
HTTP_POOL_SIZE=100
```

## 监控和维护

### 健康检查

设置定期健康检查：

```bash
# 添加到 crontab
*/5 * * * * curl -f http://localhost:8080/health || systemctl restart maa-server
```

### 日志轮转

配置 logrotate：

```
/var/log/maa-server/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 644 maa-server maa-server
    postrotate
        pkill -USR1 maa-server
    endscript
}
```

### 备份策略

重要文件备份：

```bash
# 备份配置和缓存
tar -czf maa-server-backup-$(date +%Y%m%d).tar.gz \
  config/ cache/ .env
```