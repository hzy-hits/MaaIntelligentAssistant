# 信用商店功能

## 功能概述

自动化信用商店管理系统，支持信用点获取、智能购买策略、商品优先级管理等功能。

## 核心功能

### 1. 信用点获取
```json
{
  "credit_earning": {
    "visit_friends": {
      "type": "boolean",
      "description": "自动访问好友获取信用点",
      "default": true
    },
    "clear_of1": {
      "type": "boolean", 
      "description": "使用助战干员通关OF-1获取信用点",
      "default": true
    },
    "daily_limit": {
      "type": "number",
      "description": "每日信用点获取上限",
      "value": 300
    }
  }
}
```

### 2. 购买策略配置
```json
{
  "purchase_strategy": {
    "credit_threshold": {
      "type": "number",
      "description": "信用点保留阈值，低于此值停止购买",
      "default": 300
    },
    "priority_items": {
      "type": "string",
      "description": "优先购买物品关键词，用分号分隔",
      "example": "碳;技能概要;芯片",
      "separators": [";", "；"]
    },
    "blacklist": {
      "type": "string", 
      "description": "黑名单物品关键词，用分号分隔",
      "example": "家具;装潢",
      "separators": [";", "；"]
    }
  }
}
```

### 3. 关键词匹配规则
```json
{
  "keyword_matching": {
    "flexible_match": {
      "description": "支持灵活匹配",
      "examples": {
        "碳": ["碳", "碳素", "碳素组"],
        "技能": ["技能概要", "技能概要·卷1", "技能概要·卷2"],
        "芯片": ["芯片", "芯片组", "双芯片"]
      }
    },
    "priority_order": {
      "description": "按关键词出现顺序确定优先级",
      "example": "碳;技能概要;芯片 → 碳素类优先级最高"
    }
  }
}
```

## 商品分类与推荐

### 1. 高优先级商品
```json
{
  "high_priority": {
    "materials": {
      "carbon": ["碳", "碳素", "碳素组"],
      "skill_summary": ["技能概要", "技能概要·卷1", "技能概要·卷2", "技能概要·卷3"],
      "chips": ["芯片", "芯片组", "双芯片"],
      "lmd": ["龙门币"]
    },
    "resources": {
      "recruitment_permits": ["招聘许可", "高级招聘许可"],
      "sanity_potions": ["理智药剂", "理智合剂"]
    }
  }
}
```

### 2. 中等优先级商品
```json
{
  "medium_priority": {
    "materials": {
      "building_materials": ["建筑材料", "家具零件"],
      "enhancement_materials": ["强化材料", "改良材料"]
    },
    "consumables": {
      "acceleration_permits": ["加急许可", "作战记录"]
    }
  }
}
```

### 3. 低优先级商品
```json
{
  "low_priority": {
    "decorative": {
      "furniture": ["家具", "装潢"],
      "themes": ["主题", "套装"]
    },
    "misc": {
      "gifts": ["礼品", "纪念品"]
    }
  }
}
```

## Function Calling 工具设计

```typescript
{
  name: "maa_credit_store",
  description: "管理信用商店，包括获取信用点和智能购买商品",
  parameters: {
    operation_type: {
      type: "string",
      enum: ["earn", "shop", "both"],
      description: "操作类型：earn（获取信用点）、shop（购买商品）、both（两者都做）",
      default: "both"
    },
    visit_friends: {
      type: "boolean",
      description: "是否访问好友获取信用点",
      default: true
    },
    clear_of1: {
      type: "boolean",
      description: "是否通关OF-1获取信用点",
      default: true
    },
    priority_items: {
      type: "string",
      description: "优先购买的商品关键词，用分号分隔（如：碳;技能概要;芯片）",
      default: "碳;技能概要;芯片;龙门币"
    },
    blacklist: {
      type: "string",
      description: "不购买的商品关键词，用分号分隔（如：家具;装潢）",
      default: "家具;装潢;礼品"
    },
    credit_threshold: {
      type: "number",
      description: "信用点保留阈值，低于此值停止购买",
      default: 300,
      minimum: 0
    },
    max_purchase_per_item: {
      type: "number",
      description: "每种商品最大购买数量",
      default: 99
    }
  }
}
```

## 购买策略算法

### 1. 优先级计算
```json
{
  "priority_calculation": {
    "keyword_priority": "按关键词列表顺序分配优先级权重",
    "cost_efficiency": "考虑商品性价比（价值/信用点）",
    "availability": "考虑商品剩余数量",
    "daily_refresh": "考虑每日刷新商品的稀有度"
  }
}
```

### 2. 购买决策流程
```json
{
  "decision_flow": [
    "1. 检查当前信用点数量",
    "2. 扫描商店中所有商品",
    "3. 过滤黑名单商品",
    "4. 按优先级关键词匹配商品",
    "5. 计算综合优先级分数",
    "6. 按分数排序执行购买",
    "7. 检查信用点阈值，决定是否继续"
  ]
}
```

## 自然语言理解示例

- "帮我刷信用商店" → operation_type="both"
- "只要获取信用点，不买东西" → operation_type="earn"
- "买信用商店的碳素和技能书" → priority_items="碳;技能概要"
- "信用商店全买，但不要家具" → blacklist="家具;装潢"
- "信用点保留500，其他的都买" → credit_threshold=500

## 常用配置模板

### 1. 新手推荐配置
```json
{
  "beginner_config": {
    "priority_items": "龙门币;技能概要;碳;芯片",
    "blacklist": "家具;装潢;礼品;纪念品",
    "credit_threshold": 200
  }
}
```

### 2. 进阶玩家配置
```json
{
  "advanced_config": {
    "priority_items": "碳;技能概要;芯片;招聘许可",
    "blacklist": "家具;装潢",
    "credit_threshold": 500
  }
}
```

### 3. 材料收集配置
```json
{
  "material_focus_config": {
    "priority_items": "碳;双芯片;技能概要·卷3;理智药剂",
    "blacklist": "家具;装潢;礼品;作战记录",
    "credit_threshold": 300
  }
}
```

## 注意事项

### 效率优化
1. **访问顺序**：先访问好友，再清理商店，最后通关OF-1
2. **时间安排**：建议在每日重置后执行，商品种类最多
3. **信用点管理**：保留一定数量应对突发优质商品

### 常见问题
1. **关键词设置**：过于宽泛可能买到不需要的商品
2. **阈值设置**：过高可能错过优质商品，过低可能无法应对刷新
3. **黑名单维护**：需要根据游戏版本更新调整