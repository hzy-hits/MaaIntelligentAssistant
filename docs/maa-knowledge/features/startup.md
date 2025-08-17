# 游戏启动功能

## 功能概述

自动启动游戏的完整流程，包括模拟器启动、游戏客户端启动、账号切换等功能。

## 核心功能

1. **自动启动模拟器**
   - 需要在连接设置中开启"ADB连接失败时尝试启动模拟器"
   - 需要额外配置模拟器启动设置

2. **自动启动游戏客户端**
   - 自动检测和启动对应的游戏客户端
   - 支持多种客户端类型

3. **自动进入游戏**
   - 自动完成登录流程
   - 进入游戏主界面

## 账号切换功能

### 配置要求
- 使用登录名标识账号
- 支持部分匹配（无需输入完整账号）
- 需要保证匹配的唯一性

### 示例配置
```
官服账号: 123****8901
- 可用输入: "123", "8901", "123****8901"

B服账号: 张三  
- 可用输入: "张三", "张", "三"
```

## 参数结构

### 账号切换参数
```json
{
  "account": {
    "type": "string",
    "description": "账号标识（支持部分匹配）",
    "required": false
  },
  "client_type": {
    "type": "string", 
    "enum": ["Official", "Bilibili", "Txwy", "YoStarEN", "YoStarJP", "YoStarKR"],
    "description": "客户端类型",
    "required": false
  }
}
```

## 推荐使用方式

1. **配合配置切换使用**
   - 不同账号使用不同配置文件
   - 提高切换效率

2. **配合定时执行**
   - 定时启动游戏执行日常任务
   - 自动化程度更高

## Function Calling 工具设计建议

```typescript
{
  name: "maa_startup",
  description: "启动游戏，支持模拟器启动、客户端启动和账号切换",
  parameters: {
    account: {
      type: "string",
      description: "账号标识，支持部分匹配（如手机号后4位、用户名等）"
    },
    client_type: {
      type: "string",
      enum: ["Official", "Bilibili", "Txwy", "YoStarEN", "YoStarJP", "YoStarKR"],
      description: "客户端类型，默认为Official"
    },
    start_emulator: {
      type: "boolean", 
      description: "是否尝试启动模拟器，默认为true"
    }
  }
}
```

## 技术实现要点

1. **依赖检查**
   - 验证模拟器配置
   - 检查ADB连接设置

2. **错误处理**
   - 模拟器启动失败处理
   - 账号匹配失败处理
   - 游戏启动超时处理

3. **状态监控**
   - 启动进度反馈
   - 异常状态检测