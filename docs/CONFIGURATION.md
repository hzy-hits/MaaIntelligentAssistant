# MAA 智能控制中间层 - 配置说明文档

## 概述

本项目采用分层配置系统，支持 TOML 配置文件 + 环境变量的灵活配置方式。所有硬编码配置已统一管理，便于部署和维护。

## 配置文件结构

### 主配置文件
- **位置**: `config/app.toml`
- **格式**: TOML 格式
- **作用**: 定义所有默认配置和可选项

### 环境配置文件
- **位置**: `.env` 
- **格式**: KEY=VALUE 格式
- **作用**: 运行时环境变量，优先级高于配置文件

### 配置模板
- **位置**: `.env.example`
- **作用**: 环境配置模板，复制后修改使用

## 配置项详解

### 服务器配置 [server]

```toml
[server]
default_port = "8080"              # 默认服务端口
default_host = "0.0.0.0"          # 绑定地址
health_check_path = "/health"      # 健康检查路径
tools_path = "/tools"              # 工具列表路径
call_path = "/call"                # Function Calling 路径
status_path = "/status"            # 状态查询路径
```

**环境变量覆盖**:
- `MAA_PORT` - 覆盖服务端口

### 设备连接配置 [device]

```toml
[device]
playcover_address = "127.0.0.1:1717"        # PlayCover 连接地址
android_emulator_address = "127.0.0.1:5555"  # Android 模拟器地址
touch_mode_playcover = "MacPlayTools"        # PlayCover 触摸模式
connection_timeout_ms = 10000               # 连接超时时间(毫秒)
retry_attempts = 3                          # 重试次数
```

**环境变量覆盖**:
- `MAA_DEVICE_ADDRESS` - 覆盖设备连接地址

### MAA Core 配置 [maa]

```toml
[maa]
# 默认路径 (macOS)
default_app_path = "/Applications/MAA.app"
default_core_lib_path = "/Applications/MAA.app/Contents/Frameworks/libMaaCore.dylib"
default_resource_path = "/Applications/MAA.app/Contents/Resources" 
default_adb_path = "/Applications/MAA.app/Contents/MacOS/adb"

# 备用搜索路径
fallback_lib_paths = [
    "/Applications/MAA.app/Contents/Frameworks/libMaaCore.dylib",
    "/Users/ivena/Library/Application Support/com.loong.maa/lib/libMaaCore.dylib"
]

fallback_resource_paths = [
    "/Applications/MAA.app/Contents/Resources",
    "/Users/ivena/Desktop/Fairy/maa/maa-remote-server/maa-official/resource"
]

# 运行模式标识符
stub_version = "stub"
backend_mode_real = "real"
backend_mode_stub = "stub"
```

**环境变量覆盖**:
- `MAA_CORE_LIB` - MAA 动态库路径
- `MAA_RESOURCE_PATH` - MAA 资源文件路径
- `MAA_ADB_PATH` - ADB 可执行文件路径
- `MAA_APP_PATH` - MAA 应用程序路径
- `MAA_CORE_DIR` - MAA 核心库目录
- `DYLD_LIBRARY_PATH` - macOS 动态库搜索路径
- `MAA_BACKEND_MODE` - 后端模式 (real/stub)
- `MAA_VERBOSE` - 详细输出模式
- `MAA_FORCE_STUB` - 强制使用模拟模式

### 游戏客户端配置 [client]

```toml
[client]
default_client = "Official"
supported_clients = [
    "Official",    # 官服
    "Bilibili",    # B服  
    "txwy",        # 腾讯微游戏
    "YoStarEN",    # 国际服英文
    "YoStarJP",    # 日服
    "YoStarKR"     # 韩服
]
```

### 关卡配置 [stages]

```toml
[stages]
common_stages = ["1-7", "CE-5", "CA-5", "AP-5"]           # 常用关卡
material_stages = ["CE-5", "CA-5", "AP-5", "SK-5"]        # 材料关卡
example_stages = ["1-7", "CE-5", "龙门币本", "狗粮", "H6-4"]  # 示例关卡
```

### 日志配置 [logging]

```toml
[logging]
default_level = "info"                                    # 默认日志级别
levels = ["error", "warn", "info", "debug", "trace"]     # 支持的日志级别
debug_mode_level = "debug"                               # 调试模式日志级别
production_level = "warn"                                # 生产模式日志级别
```

**环境变量覆盖**:
- `LOG_LEVEL` - 日志级别
- `DEBUG_MODE` - 调试模式开关

### AI 客户端配置 [ai]

```toml
[ai]
default_provider = "qwen"                                 # 默认 AI 提供商
supported_providers = ["qwen", "openai", "claude", "kimi", "ollama"]
default_qwen_model = "qwen-plus-2025-04-28"             # 通义千问默认模型
default_openai_model = "gpt-4"                          # OpenAI 默认模型
qwen_base_url = "https://dashscope.aliyuncs.com/compatible-mode/v1"
openai_base_url = "https://api.openai.com/v1"
```

**环境变量覆盖**:
- `AI_PROVIDER` - AI 提供商
- `AI_API_KEY` - AI API 密钥
- `AI_BASE_URL` - AI 服务基础URL
- `AI_MODEL` - AI 模型名称

### Web UI 配置 [webui]

```toml
[webui]
default_port = "3000"                    # Web UI 端口
default_name = "MAA智能助手"              # 应用名称
default_secret_key = "change-this-in-production"  # 会话密钥
```

**环境变量覆盖**:
- `WEBUI_PORT` - Web UI 端口
- `WEBUI_NAME` - 应用显示名称
- `WEBUI_SECRET_KEY` - 会话安全密钥

### 性能配置 [performance]

```toml
[performance]
task_queue_buffer_size = 1000      # 任务队列缓冲区大小
response_timeout_ms = 30000        # 响应超时时间
worker_heartbeat_ms = 1000         # 工作线程心跳间隔
connection_pool_size = 10          # 连接池大小
max_concurrent_requests = 100      # 最大并发请求数
```

### 消息配置 [messages]

```toml
[messages]
success = "Operation completed successfully"
failure = "Operation failed"
timeout = "Operation timed out"
connection_error = "Connection error"
invalid_request = "Invalid request"
system_error = "System error"
```

### 状态码配置 [status_codes]

```toml
[status_codes]
success = 0
failure = -1
timeout = -2
connection_error = -3
invalid_request = -4
```

## 配置优先级

配置系统按以下优先级加载配置：

1. **环境变量** (最高优先级)
2. **`.env` 文件**
3. **`config/app.toml` 文件**
4. **代码中的默认值** (最低优先级)

## 部署配置示例

### 开发环境配置

```bash
# .env 文件
MAA_BACKEND_MODE=stub
LOG_LEVEL=debug
DEBUG_MODE=true
MAA_DEVICE_ADDRESS=127.0.0.1:1717
```

### 生产环境配置

```bash
# .env 文件  
MAA_BACKEND_MODE=real
LOG_LEVEL=warn
DEBUG_MODE=false
MAA_CORE_LIB=/Applications/MAA.app/Contents/Frameworks/libMaaCore.dylib
MAA_RESOURCE_PATH=/Applications/MAA.app/Contents/Resources
DYLD_LIBRARY_PATH=/Applications/MAA.app/Contents/Frameworks
```

### Docker 环境配置

```bash
# docker-compose.yml 环境变量
environment:
  - MAA_PORT=8080
  - MAA_BACKEND_MODE=real
  - LOG_LEVEL=info
  - AI_PROVIDER=qwen
  - AI_API_KEY=your-api-key
```

## 配置验证

系统启动时会自动验证配置：

1. **配置文件语法检查** - TOML 格式验证
2. **必需配置检查** - 确保关键配置存在
3. **配置值验证** - 检查枚举值、数值范围等
4. **路径存在性检查** - 验证文件路径是否有效
5. **网络连接检查** - 测试设备连接地址

配置错误时系统会：
- 输出详细错误信息
- 回退到默认配置继续运行
- 记录警告日志便于排查

## 配置热更新

部分配置支持热更新（无需重启服务）：

- **日志级别** - 通过 API 接口动态调整
- **AI 配置** - 支持运行时切换提供商
- **性能参数** - 部分参数支持动态调整

不支持热更新的配置：
- 服务器端口和绑定地址
- MAA 库路径
- 数据库连接配置

## 故障排查

### 常见配置问题

1. **配置文件不存在**
   ```
   Warning: Failed to load config file, using defaults
   ```
   解决：检查 `config/app.toml` 文件是否存在

2. **MAA 库路径错误**
   ```
   Error: 加载 MAA Core 库失败
   ```
   解决：检查 `MAA_CORE_LIB` 环境变量或配置文件中的路径

3. **设备连接失败**
   ```
   Error: PlayCover连接失败: Connection refused
   ```
   解决：检查 `MAA_DEVICE_ADDRESS` 配置和设备状态

4. **权限不足**
   ```
   Error: Permission denied
   ```
   解决：检查文件权限和 macOS 安全设置

### 调试技巧

1. **启用详细日志**
   ```bash
   LOG_LEVEL=debug cargo run --bin maa-server
   ```

2. **配置验证**
   ```bash
   # 检查配置加载情况
   curl http://localhost:8080/status | jq .config
   ```

3. **环境变量检查**
   ```bash
   env | grep MAA_
   env | grep AI_
   ```

## 最佳实践

1. **生产部署**：
   - 使用环境变量管理敏感配置
   - 设置合适的日志级别
   - 配置监控和告警

2. **开发环境**：
   - 使用 `.env` 文件管理本地配置
   - 启用详细日志便于调试
   - 使用 stub 模式进行功能测试

3. **配置管理**：
   - 版本控制配置文件但不包含敏感信息
   - 使用配置模板 (`.env.example`) 
   - 定期检查和更新配置文档

4. **安全考虑**：
   - API 密钥等敏感信息只通过环境变量传递
   - 生产环境不使用默认密钥
   - 限制配置文件访问权限