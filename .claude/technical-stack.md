# 技术栈详细说明

## 后端架构 (Rust)
- **Web框架**: Axum - 高性能异步HTTP服务器
- **异步运行时**: Tokio - 协程调度和异步IO
- **并发模型**: thread_local! 单例 - 避免Arc<Mutex<>>复杂性
- **序列化**: Serde JSON - 高效的结构化数据处理
- **HTTP客户端**: Reqwest - AI API调用

## Function Tools层
- **工具数量**: 17个完整Function Calling工具
- **分类**: 核心游戏(4) + 高级自动化(4) + 辅助功能(4) + 系统功能(5)
- **调用方式**: HTTP POST /call + Function Calling协议
- **状态管理**: 任务队列 + SSE实时推送

## MAA集成 (FFI)
- **绑定库**: maa-sys - 官方Rust绑定
- **核心库**: libMaaCore.dylib/so/dll - C++实现的图像识别引擎
- **资源文件**: resource/ - 模板图片、任务配置、OCR模型
- **设备连接**: ADB协议 - 支持Android模拟器和真机

## Python决策层 (计划中)
- **FFI桥接**: PyO3 - Rust与Python双向调用
- **决策引擎**: 基于MAA任务链思想的智能策略系统
- **状态持久化**: sled/JSON - 轻量级账号状态存储
- **配置管理**: TOML/YAML - 决策规则配置化

## 实时通信
- **事件流**: Server-Sent Events (SSE) - 任务进度实时推送
- **WebSocket**: 可选的双向通信通道
- **消息格式**: JSON结构化事件数据

## 开发工具链
- **构建**: Cargo - Rust包管理和构建系统
- **测试**: cargo test + stub-mode - 支持无MAA环境开发
- **文档**: docs/ - 技术文档和架构说明
- **前端**: React 19 + Vite 5 - 现代化Web开发栈

## 数据存储
- **运行时状态**: 内存中Arc<Mutex<TaskStatus>>
- **配置文件**: JSON/TOML格式
- **账号数据**: sled嵌入式数据库 (计划)
- **日志**: 结构化日志输出

## 性能特性
- **异步处理**: 非阻塞任务执行
- **零拷贝**: 减少内存分配开销  
- **缓存优化**: 图像模板缓存
- **连接复用**: HTTP/2支持

## 安全考虑
- **权限控制**: 工具级权限管理
- **输入验证**: JSON Schema验证
- **错误处理**: Result<T, E>模式
- **资源限制**: 防止资源耗尽攻击