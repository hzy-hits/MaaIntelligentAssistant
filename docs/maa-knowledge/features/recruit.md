# 公开招募功能

## 功能概述

自动化公开招募系统，支持智能标签识别、自动招募策略、数据上报等功能。

## 核心功能

### 1. 招募模式
```json
{
  "recruit_mode": {
    "auto_confirm": {
      "description": "自动确认模式 - MAA自动选择标签并执行招募",
      "value": "auto"
    },
    "manual_confirm": {
      "description": "手动确认模式 - 用户手动选择标签",
      "value": "manual"
    }
  }
}
```

### 2. 加急许可证管理
```json
{
  "permit_settings": {
    "use_permit": {
      "type": "boolean",
      "description": "是否使用加急许可证",
      "default": false
    },
    "permit_limit": {
      "type": "number",
      "description": "加急许可证使用上限",
      "default": 999
    }
  }
}
```

### 3. 招募次数控制
```json
{
  "recruit_count": {
    "type": "number", 
    "description": "最大招募次数限制",
    "default": 0,
    "note": "0表示无限制，直到招募票用完"
  }
}
```

## 智能标签系统

### 1. 自动标签识别
- 基于OCR技术识别招募标签
- 支持多语言标签识别
- 自动纠错常见识别错误

### 2. 标签组合策略
```json
{
  "tag_strategy": {
    "priority_rare": {
      "type": "boolean",
      "description": "优先选择稀有标签组合",
      "default": true
    },
    "avoid_robot": {
      "type": "boolean", 
      "description": "避免选择会出机器人的标签",
      "default": true
    },
    "min_star": {
      "type": "number",
      "description": "最低星级要求",
      "enum": [1, 2, 3, 4, 5, 6],
      "default": 3
    }
  }
}
```

### 3. 特殊标签处理
```json
{
  "special_tags": {
    "senior_operator": {
      "priority": "highest",
      "notification": true,
      "description": "高级资深干员标签"
    },
    "top_operator": {
      "priority": "highest", 
      "notification": true,
      "description": "资深干员标签"
    },
    "support": {
      "priority": "high",
      "description": "支援机械标签"
    }
  }
}
```

## 数据统计与上报

### 1. 招募数据统计
```json
{
  "statistics": {
    "track_results": {
      "type": "boolean",
      "description": "是否统计招募结果",
      "default": true
    },
    "upload_data": {
      "type": "boolean",
      "description": "是否上报统计数据",
      "default": false
    }
  }
}
```

### 2. 外部平台集成
- 支持上报到企鹅物流统计
- 支持上报到一图流统计
- 提供匿名化数据保护

## 通知系统

### 1. 稀有标签通知
```json
{
  "notifications": {
    "rare_tags": {
      "type": "boolean",
      "description": "稀有标签（1星、5星、6星）出现时通知",
      "default": true
    },
    "guarantee_4star": {
      "type": "boolean",
      "description": "保底4星标签组合通知",
      "default": true
    }
  }
}
```

## Function Calling 工具设计

```typescript
{
  name: "maa_recruit", 
  description: "执行公开招募，支持自动标签选择、加急许可证使用、招募策略配置",
  parameters: {
    mode: {
      type: "string",
      enum: ["auto", "manual"],
      description: "招募模式：auto（自动确认）或manual（手动确认）",
      default: "auto"
    },
    max_times: {
      type: "number",
      description: "最大招募次数，0表示用完所有招募票",
      default: 0
    },
    use_permit: {
      type: "boolean",
      description: "是否使用加急许可证",
      default: false
    },
    permit_limit: {
      type: "number",
      description: "加急许可证使用上限",
      default: 999
    },
    min_star: {
      type: "number",
      enum: [1, 2, 3, 4, 5, 6],
      description: "最低星级要求，低于此星级的标签组合将被跳过",
      default: 3
    },
    priority_rare: {
      type: "boolean",
      description: "是否优先选择稀有标签组合（支援机械、高资、资深等）",
      default: true
    },
    notify_rare: {
      type: "boolean", 
      description: "稀有标签出现时是否发送通知",
      default: true
    },
    upload_data: {
      type: "boolean",
      "description": "是否上报招募数据到统计平台",
      default: false
    }
  }
}
```

## 自然语言理解示例

- "帮我做公招" → mode="auto", max_times=0
- "公招10次，可以用加急" → mode="auto", max_times=10, use_permit=true  
- "公招只要4星以上" → mode="auto", min_star=4
- "做公招，有好标签通知我" → mode="auto", notify_rare=true
- "手动公招，我要自己选标签" → mode="manual"

## 标签组合优先级

### 高优先级组合
1. **6星保证**：资深干员 + 特定职业/特性
2. **5星保证**：高级资深干员 + 特定组合
3. **4星保证**：特定标签组合

### 避免的组合
1. **1星风险**：可能出现机器人的标签组合
2. **冲突标签**：互相排斥的标签组合