# 生息演算功能

## 功能概述

生息演算自动化支持，目前处于早期开发阶段，提供基础的自动运行功能，但稳定性有限。

## 重要声明

```json
{
  "development_status": {
    "stage": "early_development",
    "stability": "limited",
    "recommendation": "不建议无人值守运行",
    "support_level": "实验性功能"
  }
}
```

## 运行模式

### 1. 默认模式（全流程）
```json
{
  "default_mode": {
    "description": "完整的生息演算流程自动化",
    "start_condition": "必须从主界面开始任务",
    "restrictions": {
      "save_file": "不能存在已有存档",
      "team_state": "当前队伍不能有干员",
      "manual_start": "需要手动开始任务"
    },
    "stability": "不稳定，可能出现各种问题"
  }
}
```

### 2. 制造模式（刷分模式）
```json
{
  "manufacturing_mode": {
    "description": "专门用于刷取材料和分数",
    "start_condition": "必须从据点界面开始",
    "default_product": "荧光棒",
    "requirements": {
      "timing": "据点结算后第一天",
      "safety": "接下来三天无敌人袭击",
      "resources": "有足够的制造材料"
    },
    "known_issues": {
      "quantity_bug": "制造99倍数的数量可能导致卡死",
      "workaround": "避免选择99、198等数量"
    }
  }
}
```

## 配置参数

### 1. 基础配置
```json
{
  "basic_config": {
    "mode": {
      "type": "string",
      "enum": ["default", "manufacturing"],
      "description": "运行模式：default（全流程）或manufacturing（制造刷分）",
      "default": "manufacturing"
    },
    "auto_start": {
      "type": "boolean",
      "description": "是否自动开始（仅制造模式支持）",
      "default": false
    },
    "target_product": {
      "type": "string",
      "description": "目标制造产品",
      "default": "荧光棒",
      "options": ["荧光棒", "其他制造品"]
    }
  }
}
```

### 2. 安全设置
```json
{
  "safety_config": {
    "max_runtime": {
      "type": "number",
      "description": "最大运行时间（分钟），防止无限循环",
      "default": 60,
      "minimum": 10,
      "maximum": 180
    },
    "error_threshold": {
      "type": "number",
      "description": "错误次数阈值，超过后停止运行",
      "default": 3
    },
    "manual_intervention": {
      "type": "boolean",
      "description": "是否需要手动干预确认",
      "default": true
    }
  }
}
```

## Function Calling 工具设计

```typescript
{
  name: "maa_reclamation",
  description: "执行生息演算任务（实验性功能，稳定性有限）",
  parameters: {
    mode: {
      type: "string",
      enum: ["default", "manufacturing"],
      description: "运行模式：default（完整流程）或manufacturing（制造刷分模式）",
      default: "manufacturing"
    },
    target_product: {
      type: "string",
      description: "制造模式下的目标产品",
      default: "荧光棒"
    },
    max_runtime: {
      type: "number",
      description: "最大运行时间（分钟），防止卡死",
      default: 60,
      minimum: 10,
      maximum: 180
    },
    supervision_required: {
      type: "boolean",
      description: "是否需要人工监督（强烈建议开启）",
      default: true
    },
    auto_stop_on_error: {
      type: "boolean",
      description: "遇到错误时是否自动停止",
      default: true
    },
    manufacturing_quantity: {
      type: "number",
      description: "制造数量（避免99的倍数）",
      default: 50,
      note: "避免99、198等数量以防卡死"
    }
  }
}
```

## 使用前检查清单

### 1. 默认模式检查
```json
{
  "default_mode_checklist": [
    "确认从主界面开始",
    "确认无已有存档",
    "确认当前队伍为空",
    "准备手动开始任务",
    "确保有足够时间监督"
  ]
}
```

### 2. 制造模式检查
```json
{
  "manufacturing_mode_checklist": [
    "确认从据点界面开始",
    "确认为结算后第一天",
    "确认接下来三天无敌人",
    "确认有制造材料",
    "避免99倍数的制造数量"
  ]
}
```

## 错误处理与风险管理

### 1. 常见错误
```json
{
  "common_errors": {
    "stuck_in_ui": {
      "description": "界面卡死",
      "solution": "重启游戏重新开始",
      "prevention": "避免99倍数制造"
    },
    "save_conflict": {
      "description": "存档冲突",
      "solution": "清理存档重新开始",
      "prevention": "确保开始前无存档"
    },
    "team_conflict": {
      "description": "队伍冲突",
      "solution": "清空当前队伍",
      "prevention": "开始前确认队伍为空"
    }
  }
}
```

### 2. 监督建议
```json
{
  "supervision_guidelines": {
    "check_interval": "每10-15分钟检查一次",
    "key_points": [
      "界面是否正常响应",
      "是否出现异常弹窗",
      "制造进度是否正常",
      "资源消耗是否合理"
    ],
    "emergency_stop": "发现异常立即手动停止"
  }
}
```

## 自然语言理解示例

- "帮我刷生息演算" → mode="manufacturing", supervision_required=true
- "生息演算制造荧光棒" → mode="manufacturing", target_product="荧光棒"
- "生息演算全流程，我会看着" → mode="default", supervision_required=true
- "生息演算最多跑1小时" → max_runtime=60

## 版本兼容性

### 1. 当前支持版本
```json
{
  "supported_versions": {
    "game_version": "最新版本明日方舟",
    "maa_version": "最新版本MAA",
    "platform": ["Android", "模拟器"],
    "resolution": "1080p及以上推荐"
  }
}
```

### 2. 已知限制
```json
{
  "known_limitations": [
    "功能不完整，随时可能失效",
    "不支持复杂的投资策略",
    "无法处理特殊事件",
    "可能与游戏更新不兼容",
    "需要大量人工监督"
  ]
}
```

## 未来改进计划

```json
{
  "future_improvements": {
    "stability": "提高运行稳定性",
    "feature_completeness": "完善功能覆盖",
    "error_handling": "改进错误处理机制",
    "automation_level": "提高自动化程度",
    "user_experience": "简化配置流程"
  }
}
```

## 免责声明

**重要提醒**：生息演算功能目前为实验性质，可能导致：
1. 游戏进度损失
2. 资源浪费
3. 账号异常
4. 数据损坏

**建议**：
- 仅在测试环境使用
- 全程人工监督
- 做好数据备份
- 遇到问题立即停止