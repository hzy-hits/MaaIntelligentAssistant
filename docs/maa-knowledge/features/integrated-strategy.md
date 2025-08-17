# 肉鸽模式（集成战略）功能

## 功能概述

自动化集成战略（肉鸽）模式，支持多主题、多难度、智能干员选择、自动战斗策略等高级功能。

## 核心功能

### 1. 主题与难度配置
```json
{
  "theme_config": {
    "available_themes": {
      "phantom": {
        "name": "傀影与猩红孤钻",
        "difficulty_range": [3, 10],
        "recommended_squad": ["指挥分队", "突击战术分队"]
      },
      "tideflow": {
        "name": "水月与深蓝之树", 
        "difficulty_range": [3, 10],
        "recommended_squad": ["远程战术分队", "指挥分队"]
      },
      "challenge_nature": {
        "name": "探索挑战自然",
        "difficulty_range": [3, 8],
        "recommended_squad": ["突击战术分队"]
      },
      "facing_soul": {
        "name": "面向灵魂",
        "difficulty_range": [3, 9],
        "recommended_squad": ["指挥分队", "远程战术分队"]
      },
      "invite_garden": {
        "name": "邀请花园",
        "difficulty_range": [3, 10],
        "recommended_squad": ["指挥分队"]
      }
    },
    "auto_select_latest": {
      "type": "boolean",
      "description": "是否自动选择最新主题",
      "default": true
    }
  }
}
```

### 2. 分队配置系统
```json
{
  "squad_config": {
    "squad_types": {
      "command": {
        "name": "指挥分队",
        "description": "平衡型分队，适合多数主题",
        "starting_operators": 4,
        "focus": "versatility"
      },
      "assault": {
        "name": "突击战术分队",
        "description": "攻击型分队，适合快速推进",
        "starting_operators": 3,
        "focus": "offense"
      },
      "remote": {
        "name": "远程战术分队", 
        "description": "远程型分队，适合控制流",
        "starting_operators": 3,
        "focus": "ranged_control"
      }
    },
    "starting_operator": {
      "type": "string",
      "description": "起始干员选择",
      "support_friend": {
        "type": "boolean",
        "description": "是否可以使用好友助战干员",
        "default": true
      }
    }
  }
}
```

### 3. 战斗策略系统
```json
{
  "battle_strategy": {
    "auto_battle": {
      "type": "boolean",
      "description": "是否使用自动战斗",
      "default": true
    },
    "strategy_files": {
      "description": "内置作业文件",
      "auto_detection": "自动识别干员技能和等级",
      "fallback": "失败时使用通用策略"
    },
    "timeout_settings": {
      "battle_timeout": {
        "type": "number",
        "description": "单场战斗超时时间（分钟）",
        "default": 5
      },
      "auto_retreat": {
        "type": "boolean",
        "description": "超时后是否自动撤退",
        "default": true
      }
    }
  }
}
```

## 智能购买系统

### 1. 商店购买策略
```json
{
  "shop_strategy": {
    "priority_items": {
      "high": [
        "强力收藏品",
        "核心干员",
        "关键道具"
      ],
      "medium": [
        "辅助收藏品",
        "资源道具",
        "临时加成"
      ],
      "low": [
        "普通道具",
        "基础资源"
      ]
    },
    "purchase_logic": {
      "auto_buy_powerful": {
        "type": "boolean",
        "description": "是否自动购买强力收藏品",
        "default": true
      },
      "budget_management": {
        "type": "number",
        "description": "预算管理阈值",
        "default": 80
      }
    }
  }
}
```

### 2. 收藏品评估算法
```json
{
  "collection_evaluation": {
    "power_rating": {
      "description": "收藏品强度评级",
      "factors": ["效果强度", "适用性", "叠加效果"]
    },
    "synergy_detection": {
      "description": "协同效应检测",
      "analysis": "与已有收藏品的组合效果"
    }
  }
}
```

## 自动化设置

### 1. 运行参数
```json
{
  "automation_settings": {
    "performance_requirements": {
      "min_fps": 60,
      "stable_connection": true,
      "description": "需要稳定的60+FPS运行环境"
    },
    "error_handling": {
      "auto_reconnect": {
        "type": "boolean",
        "description": "断线后自动重连",
        "default": true
      },
      "auto_retry": {
        "type": "boolean",
        "description": "出现问题时自动重试",
        "default": true
      },
      "max_retries": {
        "type": "number",
        "description": "最大重试次数",
        "default": 3
      }
    }
  }
}
```

### 2. 探索控制
```json
{
  "exploration_control": {
    "target_theme_only": {
      "type": "boolean",
      "description": "是否只运行目标主题",
      "default": true,
      "note": "建议手动结束非目标主题的探索"
    },
    "difficulty_progression": {
      "type": "string",
      "enum": ["fixed", "progressive", "adaptive"],
      "description": "难度选择策略",
      "default": "fixed"
    }
  }
}
```

## Function Calling 工具设计

```typescript
{
  name: "maa_roguelike",
  description: "执行集成战略（肉鸽）模式，支持多主题、自动战斗、智能购买等功能",
  parameters: {
    theme: {
      type: "string",
      enum: ["phantom", "tideflow", "challenge_nature", "facing_soul", "invite_garden", "latest"],
      description: "选择主题，latest表示最新主题",
      default: "latest"
    },
    difficulty: {
      type: "number",
      description: "难度等级，根据主题不同范围3-10",
      minimum: 3,
      maximum: 10,
      default: 5
    },
    squad_type: {
      type: "string",
      enum: ["command", "assault", "remote"],
      description: "分队类型：command（指挥）、assault（突击）、remote（远程）",
      default: "command"
    },
    starting_operator: {
      type: "string",
      description: "起始干员名称，空字符串表示随机选择",
      default: ""
    },
    use_friend_support: {
      type: "boolean",
      description: "是否使用好友助战干员",
      default: true
    },
    auto_battle: {
      type: "boolean",
      description: "是否使用自动战斗策略",
      default: true
    },
    battle_timeout: {
      type: "number",
      description: "单场战斗超时时间（分钟）",
      default: 5,
      minimum: 1,
      maximum: 10
    },
    auto_shop: {
      type: "boolean",
      description: "是否自动购买商店物品",
      default: true
    },
    target_runs: {
      type: "number",
      description: "目标运行次数，0表示无限制",
      default: 1,
      minimum: 0
    }
  }
}
```

## 主题特性与策略

### 1. 傀影与猩红孤钻
```json
{
  "phantom_theme": {
    "characteristics": {
      "boss_type": "傀影",
      "special_mechanics": "孤钻系统",
      "recommended_strategy": "平衡型推进"
    },
    "optimal_squad": "指挥分队",
    "difficulty_recommendation": "5-7级适合练度较好的玩家"
  }
}
```

### 2. 水月与深蓝之树
```json
{
  "tideflow_theme": {
    "characteristics": {
      "boss_type": "深蓝之树",
      "special_mechanics": "水月机制",
      "recommended_strategy": "远程控制优先"
    },
    "optimal_squad": "远程战术分队",
    "difficulty_recommendation": "6-8级适合远程队配置"
  }
}
```

## 自然语言理解示例

- "帮我刷肉鸽" → theme="latest", difficulty=5, squad_type="command"
- "刷傀影主题，难度7级" → theme="phantom", difficulty=7
- "肉鸽用突击队，要朋友助战" → squad_type="assault", use_friend_support=true
- "自动肉鸽5次" → target_runs=5, auto_battle=true
- "肉鸽水月主题，用远程队" → theme="tideflow", squad_type="remote"

## 注意事项

### 性能要求
1. **帧率要求**：必须保持60+FPS稳定运行
2. **网络稳定**：避免频繁断线影响探索进度
3. **设备性能**：确保设备温度和性能稳定

### 首次使用
1. **手动设置**：首次使用需要手动设置初始难度
2. **策略适配**：建议先手动运行几次熟悉机制
3. **干员配置**：确保关键干员已经充分练度

### 风险管理
1. **意外终止**：系统会自动重试，但可能损失进度
2. **策略失效**：某些特殊关卡可能需要手动干预
3. **资源消耗**：长时间运行会消耗大量时间和设备电量