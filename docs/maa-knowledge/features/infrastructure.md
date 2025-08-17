# 基建管理功能

## 功能概述

自动化基建设施管理系统，支持智能排班、资源收集、无人机使用、效率优化等功能。

## 核心功能

### 1. 设施类型与优化
```json
{
  "facilities": {
    "manufacturing": {
      "types": ["经验书", "赤金", "源石碎片", "芯片"],
      "optimization": "单设施内最优排班"
    },
    "trading_post": {
      "types": ["订单处理"],
      "optimization": "效率最大化"
    },
    "power_plant": {
      "types": ["电力生产"],
      "optimization": "电力供应稳定"
    },
    "dormitory": {
      "types": ["心情恢复"],
      "optimization": "基于心情值的智能分配"
    },
    "office": {
      "types": ["线索搜集", "线索交换"],
      "optimization": "智能干员选择"
    },
    "control_center": {
      "types": ["指挥中心"],
      "optimization": "部分实现复杂策略"
    }
  }
}
```

### 2. 排班策略
```json
{
  "scheduling_strategy": {
    "optimization_scope": {
      "type": "string",
      "value": "single_facility",
      "description": "当前支持单设施内优化，不支持跨设施全局优化"
    },
    "special_combinations": {
      "type": "array",
      "description": "支持特殊干员组合",
      "examples": ["温蒂+杜宾", "巫恋+杜宾", "稀音+杜宾"]
    },
    "skill_detection": {
      "type": "boolean",
      "description": "自动识别干员技能组合",
      "default": true
    }
  }
}
```

### 3. 无人机使用策略
```json
{
  "drone_usage": {
    "auto_use": {
      "type": "boolean", 
      "description": "是否自动使用无人机",
      "default": false
    },
    "priority_facility": {
      "type": "string",
      "enum": ["manufacturing", "trading_post", "none"],
      "description": "优先使用无人机的设施类型"
    },
    "threshold": {
      "type": "number",
      "description": "无人机使用阈值（剩余百分比）",
      "default": 10
    }
  }
}
```

## 设施管理详细配置

### 1. 制造站管理
```json
{
  "manufacturing": {
    "product_types": {
      "exp_book": {
        "name": "经验书",
        "operators": ["能天使", "慕斯", "夜烟"],
        "efficiency_bonus": "经验书制造效率+XX%"
      },
      "gold": {
        "name": "赤金", 
        "operators": ["杰西卡", "白雪", "流星"],
        "efficiency_bonus": "赤金制造效率+XX%"
      },
      "originium": {
        "name": "源石碎片",
        "operators": ["夜魔", "格雷伊", "芳汀"],
        "efficiency_bonus": "源石碎片制造效率+XX%"
      }
    }
  }
}
```

### 2. 贸易站管理
```json
{
  "trading_post": {
    "order_priority": {
      "type": "string",
      "enum": ["efficiency", "speed", "balance"],
      "description": "订单处理优先级策略"
    },
    "operators": ["德克萨斯", "拉普兰德", "能天使"],
    "efficiency_bonus": "订单获得效率+XX%"
  }
}
```

### 3. 宿舍管理
```json
{
  "dormitory": {
    "assignment_rule": {
      "mood_threshold": {
        "type": "number",
        "description": "心情值阈值，低于此值的干员优先进入宿舍",
        "default": 12
      },
      "comfort_level": {
        "type": "number", 
        "description": "宿舍舒适度等级",
        "range": [1, 5000]
      }
    }
  }
}
```

### 4. 会客室（线索交换）
```json
{
  "office": {
    "clue_strategy": {
      "auto_give": {
        "type": "boolean",
        "description": "是否自动赠送线索",
        "default": true
      },
      "auto_receive": {
        "type": "boolean",
        "description": "是否自动接收线索",
        "default": true
      },
      "preferred_operators": ["阿米娅", "集成战略", "红"]
    }
  }
}
```

## Function Calling 工具设计

```typescript
{
  name: "maa_infrastructure",
  description: "执行基建设施管理，包括自动排班、收菜、无人机使用等",
  parameters: {
    operation_type: {
      type: "string",
      enum: ["collect", "schedule", "drone", "all"],
      description: "操作类型：collect（收菜）、schedule（排班）、drone（无人机）、all（全部）",
      default: "all"
    },
    facilities: {
      type: "array",
      items: {
        type: "string",
        enum: ["manufacturing", "trading", "power", "dormitory", "office", "control"]
      },
      description: "要管理的设施类型，空数组表示所有设施",
      default: []
    },
    use_drone: {
      type: "boolean",
      description: "是否使用无人机加速",
      default: false
    },
    drone_facility: {
      type: "string",
      enum: ["manufacturing", "trading"],
      description: "无人机优先使用的设施类型",
      default: "manufacturing"
    },
    mood_threshold: {
      type: "number",
      description": "宿舍分配的心情阈值，低于此值的干员优先休息",
      default: 12,
      minimum: 0,
      maximum: 24
    },
    auto_clue: {
      type: "boolean",
      description: "是否自动处理线索（赠送和接收）",
      default: true
    }
  }
}
```

## 效率优化策略

### 1. 干员技能组合
```json
{
  "skill_combinations": {
    "manufacturing_exp": {
      "primary": "能天使",
      "secondary": ["慕斯", "夜烟"],
      "effect": "经验书制造效率最大化"
    },
    "manufacturing_gold": {
      "primary": "杰西卡", 
      "secondary": ["白雪", "流星"],
      "effect": "赤金制造效率最大化"
    },
    "trading_efficiency": {
      "primary": "德克萨斯",
      "secondary": ["拉普兰德", "能天使"],
      "effect": "订单获得效率最大化"
    }
  }
}
```

### 2. 设施优先级
```json
{
  "priority_order": [
    "贸易站（订单获得）",
    "制造站（高需求材料）", 
    "发电站（电力供应）",
    "宿舍（心情恢复）",
    "会客室（线索交换）",
    "控制中枢（全局加成）"
  ]
}
```

## 自然语言理解示例

- "帮我收基建" → operation_type="collect"
- "基建排班" → operation_type="schedule"
- "基建全套，包括无人机" → operation_type="all", use_drone=true
- "只收制造站和贸易站" → facilities=["manufacturing", "trading"]
- "基建收菜，心情低于10的去休息" → operation_type="all", mood_threshold=10

## 注意事项

### 当前限制
1. **优化范围**：仅支持单设施内优化，不支持跨设施全局优化
2. **控制中枢**：复杂策略仅部分实现
3. **预设队列**：支持预设轮换，但灵活性有限

### 未来改进方向
1. 跨设施全局优化算法
2. 更复杂的控制中枢策略
3. 自定义排班规则
4. 实时效率监控