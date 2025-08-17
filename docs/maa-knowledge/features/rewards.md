# 奖励领取功能

## 功能概述

自动化奖励领取系统，支持每日奖励、每周奖励、邮件奖励、任务奖励等多种奖励的自动收集。

## 核心功能

### 1. 奖励类型
```json
{
  "reward_types": {
    "daily_rewards": {
      "description": "每日签到奖励",
      "frequency": "daily",
      "auto_collect": true
    },
    "weekly_rewards": {
      "description": "每周任务奖励",
      "frequency": "weekly", 
      "auto_collect": true
    },
    "mail_rewards": {
      "description": "邮件附件奖励",
      "frequency": "as_available",
      "auto_collect": true
    },
    "mission_rewards": {
      "description": "任务完成奖励",
      "frequency": "as_completed",
      "auto_collect": true
    },
    "event_rewards": {
      "description": "活动奖励",
      "frequency": "event_based",
      "auto_collect": true
    }
  }
}
```

### 2. 领取策略配置
```json
{
  "collection_strategy": {
    "auto_collect_all": {
      "type": "boolean",
      "description": "是否自动领取所有可领取奖励",
      "default": true
    },
    "mail_threshold": {
      "type": "number",
      "description": "邮件保留数量阈值，超过时自动清理",
      "default": 100
    },
    "selective_collection": {
      "type": "array",
      "description": "选择性领取的奖励类型",
      "items": {
        "type": "string",
        "enum": ["daily", "weekly", "mail", "mission", "event"]
      }
    }
  }
}
```

### 3. 奖励优先级
```json
{
  "priority_order": {
    "high_priority": [
      "源石奖励",
      "寻访凭证",
      "理智药剂",
      "招聘许可"
    ],
    "medium_priority": [
      "龙门币",
      "经验卡",
      "技能概要",
      "碳素材料"
    ],
    "low_priority": [
      "家具零件",
      "装潢用品",
      "礼品道具"
    ]
  }
}
```

## 详细功能模块

### 1. 每日奖励系统
```json
{
  "daily_system": {
    "sign_in": {
      "description": "每日签到",
      "rewards": ["龙门币", "经验卡", "理智药剂", "源石"]
    },
    "daily_missions": {
      "description": "每日任务",
      "auto_complete": false,
      "auto_collect": true
    },
    "login_rewards": {
      "description": "登录奖励",
      "duration": "7_days_cycle"
    }
  }
}
```

### 2. 每周奖励系统
```json
{
  "weekly_system": {
    "weekly_missions": {
      "description": "每周任务奖励",
      "reset_day": "monday",
      "rewards": ["寻访凭证", "源石", "高级材料"]
    },
    "annihilation_rewards": {
      "description": "剿灭作战奖励",
      "weekly_limit": 1800,
      "auto_collect": true
    }
  }
}
```

### 3. 邮件管理系统
```json
{
  "mail_system": {
    "auto_open": {
      "type": "boolean",
      "description": "是否自动打开邮件",
      "default": true
    },
    "auto_collect_attachments": {
      "type": "boolean",
      "description": "是否自动收取附件",
      "default": true
    },
    "auto_delete": {
      "type": "boolean",
      "description": "是否自动删除已读邮件",
      "default": false
    },
    "keep_important": {
      "type": "boolean",
      "description": "是否保留重要邮件",
      "default": true
    }
  }
}
```

## Function Calling 工具设计

```typescript
{
  name: "maa_rewards",
  description: "自动领取各种游戏奖励，包括每日、每周、邮件、任务奖励等",
  parameters: {
    reward_types: {
      type: "array",
      items: {
        type: "string",
        enum: ["daily", "weekly", "mail", "mission", "event", "all"]
      },
      description: "要领取的奖励类型，all表示所有类型",
      default: ["all"]
    },
    auto_sign_in: {
      type: "boolean",
      description: "是否自动每日签到",
      default: true
    },
    collect_mail: {
      type: "boolean",
      description: "是否收取邮件附件",
      default: true
    },
    mail_threshold: {
      type: "number",
      description: "邮件保留数量阈值，超过时自动清理",
      default: 100,
      minimum: 10,
      maximum: 999
    },
    collect_missions: {
      type: "boolean",
      description: "是否领取已完成的任务奖励",
      default: true
    },
    priority_only: {
      type: "boolean",
      description: "是否只领取高优先级奖励",
      default: false
    },
    delete_empty_mail: {
      type: "boolean",
      description: "是否删除无附件的已读邮件",
      default: false
    }
  }
}
```

## 奖励识别与分类

### 1. 自动识别系统
```json
{
  "recognition_system": {
    "reward_icon_detection": {
      "description": "基于图标识别奖励类型",
      "accuracy": "高精度",
      "supported_types": ["源石", "龙门币", "材料", "道具"]
    },
    "text_recognition": {
      "description": "基于文字识别奖励名称",
      "ocr_engine": "内置OCR",
      "language_support": ["中文", "英文", "日文", "韩文"]
    }
  }
}
```

### 2. 智能分类算法
```json
{
  "classification_algorithm": {
    "value_assessment": {
      "description": "评估奖励价值",
      "factors": ["稀有度", "市场价值", "用途重要性"]
    },
    "urgency_detection": {
      "description": "检测奖励紧急程度",
      "indicators": ["过期时间", "邮件数量", "存储空间"]
    }
  }
}
```

## 自然语言理解示例

- "帮我领取所有奖励" → reward_types=["all"]
- "只领每日签到" → reward_types=["daily"], auto_sign_in=true
- "收邮件但不要删除" → collect_mail=true, delete_empty_mail=false
- "领任务奖励" → reward_types=["mission"], collect_missions=true
- "清理邮箱，保留50封" → mail_threshold=50, delete_empty_mail=true

## 高级配置选项

### 1. 自定义过滤规则
```json
{
  "custom_filters": {
    "keyword_whitelist": {
      "type": "array",
      "description": "关键词白名单，只领取包含这些关键词的奖励",
      "example": ["源石", "凭证", "许可"]
    },
    "keyword_blacklist": {
      "type": "array", 
      "description": "关键词黑名单，跳过包含这些关键词的奖励",
      "example": ["家具", "装潢", "礼品"]
    },
    "minimum_value": {
      "type": "number",
      "description": "最低价值阈值，低于此值的奖励将被跳过"
    }
  }
}
```

### 2. 时间管理
```json
{
  "time_management": {
    "collection_schedule": {
      "daily_time": "09:00",
      "weekly_time": "monday_09:00", 
      "description": "推荐的奖励领取时间"
    },
    "expiry_monitoring": {
      "type": "boolean",
      "description": "是否监控奖励过期时间",
      "default": true
    }
  }
}
```

## 注意事项

### 存储管理
1. **包裹空间**：领取前检查包裹剩余空间
2. **材料上限**：某些材料有存储上限
3. **过期提醒**：及时领取有时效性的奖励

### 网络稳定性
1. **重试机制**：网络错误时自动重试
2. **断点续传**：支持中断后继续领取
3. **状态保存**：记录已领取的奖励状态