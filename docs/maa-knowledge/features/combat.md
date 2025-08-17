# 战斗系统功能

## 功能概述

自动战斗系统，支持多种关卡类型的自动刷取，包含智能理智管理、掉落统计、自动使用道具等高级功能。

## 核心功能

### 1. 关卡类型支持
- **主线关卡**：普通/困难模式
- **资源关卡**：
  - CE系列（龙门币本）
  - LS系列（经验书本）
  - CA系列（技能书本）
  - AP系列（红票本）
- **剿灭作战**：周常剿灭任务
- **活动关卡**：限时活动关卡

### 2. 理智管理
```json
{
  "sanity_management": {
    "use_medicine": {
      "type": "boolean",
      "description": "是否使用理智药剂",
      "default": false
    },
    "use_stone": {
      "type": "boolean", 
      "description": "是否使用源石回复理智",
      "default": false
    },
    "medicine_limit": {
      "type": "number",
      "description": "理智药剂使用上限",
      "default": 999
    },
    "stone_limit": {
      "type": "number",
      "description": "源石使用上限", 
      "default": 0
    }
  }
}
```

### 3. 战斗次数控制
```json
{
  "battle_count": {
    "type": "number",
    "description": "指定战斗次数，0表示无限制",
    "default": 0
  },
  "target_material": {
    "type": "string",
    "description": "目标掉落材料，获得后停止",
    "required": false
  }
}
```

## 高级设置

### 1. 代理指挥配置
```json
{
  "auto_agent": {
    "enable": {
      "type": "boolean",
      "description": "是否自动选择代理指挥",
      "default": true
    },
    "backup_stage": {
      "type": "string", 
      "description": "代理指挥失败时的后备关卡",
      "example": "1-7"
    }
  }
}
```

### 2. 掉落统计
```json
{
  "drop_tracking": {
    "enable": {
      "type": "boolean",
      "description": "是否启用掉落统计",
      "default": true
    },
    "upload_penguin": {
      "type": "boolean",
      "description": "是否上报企鹅物流",
      "default": false
    },
    "upload_yituliu": {
      "type": "boolean", 
      "description": "是否上报一图流",
      "default": false
    }
  }
}
```

### 3. 异常处理
```json
{
  "error_handling": {
    "auto_reconnect": {
      "type": "boolean",
      "description": "断线时自动重连",
      "default": true
    },
    "continue_on_level_up": {
      "type": "boolean",
      "description": "升级后继续战斗",
      "default": true
    },
    "clear_remaining_sanity": {
      "type": "boolean",
      "description": "清空剩余理智",
      "default": false
    }
  }
}
```

## 特殊机制

### 1. 短路机制
- 任务完成条件：任一条件满足即停止
- 条件类型：
  - 达到指定次数
  - 获得目标材料
  - 理智耗尽

### 2. PRTS代理卡
- 自动使用PRTS代理指挥卡
- 失败时自动切换到后备关卡

## Function Calling 工具设计

```typescript
{
  name: "maa_combat",
  description: "执行自动战斗，支持关卡刷取、理智管理、掉落统计等功能",
  parameters: {
    stage: {
      type: "string",
      description: "关卡代码（如1-7、CE-5、AP-5等）",
      required: true
    },
    battle_count: {
      type: "number",
      description: "战斗次数，0表示无限制直到理智耗尽",
      default: 0
    },
    use_medicine: {
      type: "boolean",
      description: "是否使用理智药剂",
      default: false
    },
    medicine_limit: {
      type: "number",
      description: "理智药剂使用上限",
      default: 999
    },
    use_stone: {
      type: "boolean",
      description": "是否使用源石恢复理智",
      default: false
    },
    stone_limit: {
      type: "number", 
      description: "源石使用上限",
      default: 0
    },
    target_material: {
      type: "string",
      description: "目标掉落材料名称，获得后停止",
      required: false
    },
    auto_agent: {
      type: "boolean",
      description: "是否自动选择代理指挥",
      default: true
    },
    backup_stage: {
      type: "string",
      description: "代理指挥失败时的后备关卡",
      required: false
    },
    drop_tracking: {
      type: "boolean",
      description": "是否启用掉落统计",
      default: true
    }
  }
}
```

## 常用关卡映射

```json
{
  "resource_stages": {
    "龙门币": ["CE-5", "CE-6"],
    "经验书": ["LS-5", "LS-6"], 
    "技能书": ["CA-5"],
    "红票": ["AP-5"],
    "狗粮": ["1-7"],
    "固源岩": ["1-7", "S4-1"],
    "酯原料": ["2-10", "S3-2"]
  }
}
```

## 自然语言理解示例

- "刷1-7 50次" → stage="1-7", battle_count=50
- "刷龙门币本直到理智用完" → stage="CE-5", battle_count=0
- "刷经验书，可以嗑药" → stage="LS-5", use_medicine=true
- "刷固源岩到获得10个" → stage="1-7", target_material="固源岩"