# API 参考文档

## 概述

MAA 智能控制中间层提供基于 Function Calling 的 HTTP API 接口。

## 基础信息

- **Base URL**: `http://localhost:8080`
- **Content-Type**: `application/json`
- **当前状态**: Stub 模式（返回模拟数据）

## 核心端点

### 1. 获取工具列表

获取所有可用的 Function Calling 工具定义。

```http
GET /tools
```

**响应格式**:
返回符合 OpenAI Function Calling 标准的工具定义数组。

### 2. 执行函数调用

执行指定的函数。

```http
POST /call
```

**请求格式**:
```json
{
  "function_call": {
    "name": "function_name",
    "arguments": {
      "param1": "value1"
    }
  }
}
```

### 3. 健康检查

检查服务状态。

```http
GET /health
```

**响应**:
```json
{
  "status": "ok",
  "mode": "stub"
}
```

## 支持的函数

### maa_command

执行自然语言命令。

**参数**:
- `command` (string): 自然语言命令

**示例**:
```json
{
  "function_call": {
    "name": "maa_command",
    "arguments": {
      "command": "截图"
    }
  }
}
```

### maa_operators

管理干员信息。

**参数**:
- `action` (string): 操作类型 ("scan", "query", "update")
- `name` (string, 可选): 干员名称

**示例**:
```json
{
  "function_call": {
    "name": "maa_operators",
    "arguments": {
      "action": "query",
      "name": "银灰"
    }
  }
}
```

### maa_copilot

作业匹配推荐。

**参数**:
- `stage` (string): 关卡名称
- `mode` (string): 匹配模式 ("simple", "level", "smart")

**示例**:
```json
{
  "function_call": {
    "name": "maa_copilot",
    "arguments": {
      "stage": "1-7",
      "mode": "simple"
    }
  }
}
```

### maa_status

获取系统状态。

**参数**: 无

**示例**:
```json
{
  "function_call": {
    "name": "maa_status",
    "arguments": {}
  }
}
```

## 响应格式

所有函数调用返回统一格式：

```json
{
  "success": true,
  "result": {
    // 具体结果数据
  },
  "message": "操作说明"
}
```

错误响应：
```json
{
  "success": false,
  "error": "错误描述",
  "code": "ERROR_CODE"
}
```

## 使用示例

### curl 命令示例

```bash
# 获取工具列表
curl http://localhost:8080/tools

# 执行截图命令
curl -X POST http://localhost:8080/call \
  -H "Content-Type: application/json" \
  -d '{
    "function_call": {
      "name": "maa_command",
      "arguments": {"command": "截图"}
    }
  }'

# 查询干员信息
curl -X POST http://localhost:8080/call \
  -H "Content-Type: application/json" \
  -d '{
    "function_call": {
      "name": "maa_operators",
      "arguments": {"action": "query", "name": "银灰"}
    }
  }'
```

## 当前限制

1. **Stub 模式**: 当前返回模拟数据，不执行实际 MAA 操作
2. **错误处理**: 基础错误处理，详细错误信息待完善
3. **认证**: 暂无认证机制
4. **率限**: 暂无请求频率限制

## 集成指南

### 与 AI 模型集成

本 API 设计兼容主流 AI 模型的 Function Calling 协议：

- OpenAI GPT 系列
- Anthropic Claude
- 其他支持 Function Calling 的模型

### Python 集成示例

```python
import requests

# 获取工具定义
tools_response = requests.get("http://localhost:8080/tools")
tools = tools_response.json()

# 执行函数调用
call_data = {
    "function_call": {
        "name": "maa_command",
        "arguments": {"command": "截图"}
    }
}
response = requests.post("http://localhost:8080/call", json=call_data)
result = response.json()
```

注意：当前为开发版本，主要功能以 Stub 模式运行。实际 MAA 集成需要启用 FFI 模式。