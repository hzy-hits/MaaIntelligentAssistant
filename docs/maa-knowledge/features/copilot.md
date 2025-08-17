# 作业系统功能

## 功能概述

MAA作业系统是自动战斗的核心功能，支持通过JSON格式的作业文件实现全自动战斗，包含智能干员匹配、自动编队、战斗执行等完整流程。

## 核心功能

### 1. 作业支持模式
```json
{
  "supported_modes": {
    "normal_battle": {
      "description": "常规关卡自动战斗",
      "requirement": "可编队的关卡",
      "stability": "高"
    },
    "integrated_strategy": {
      "description": "集成战略（肉鸽）模式",
      "requirement": "肉鸽关卡",
      "stability": "中等"
    },
    "event_stages": {
      "description": "活动关卡",
      "requirement": "支持编队的活动关卡",
      "stability": "高"
    }
  }
}
```

### 2. 性能要求
```json
{
  "performance_requirements": {
    "min_fps": 60,
    "stability": "必须稳定保持60+FPS",
    "device": "推荐高性能设备或模拟器",
    "resolution": "1080p或更高分辨率"
  }
}
```

## 作业导入系统

### 1. 导入方式
```json
{
  "import_methods": {
    "local_json": {
      "description": "本地JSON文件导入",
      "format": "MAA标准作业格式",
      "encoding": "UTF-8"
    },
    "website_code": {
      "description": "网站分享代码导入",
      "source": "prts.plus等作业分享网站",
      "format": "base64编码的JSON"
    },
    "file_selection": {
      "description": "文件选择器导入",
      "supported_formats": [".json", ".txt"]
    }
  }
}
```

### 2. 作业验证
```json
{
  "validation_system": {
    "format_check": {
      "description": "JSON格式验证",
      "required_fields": ["title", "details", "opers", "groups", "actions"]
    },
    "compatibility_check": {
      "description": "版本兼容性检查",
      "game_version": "检查游戏版本兼容性",
      "maa_version": "检查MAA版本兼容性"
    },
    "operator_check": {
      "description": "干员可用性检查",
      "missing_operators": "识别缺失的干员",
      "alternative_suggestions": "提供替代干员建议"
    }
  }
}
```

## 自动编队系统

### 1. 编队规则
```json
{
  "team_formation": {
    "clear_existing": {
      "description": "清空当前编队",
      "automatic": true
    },
    "rebuild_team": {
      "description": "根据作业重新编队",
      "source": "作业文件中的干员配置"
    },
    "custom_additions": {
      "description": "添加自定义干员",
      "user_defined": "用户可以手动添加干员"
    },
    "trust_supplement": {
      "description": "低信赖度干员补充",
      "automatic_replacement": "自动替换低信赖度干员"
    }
  }
}
```

### 2. 编队限制
```json
{
  "formation_restrictions": {
    "paradox_simulation": {
      "description": "悖论模拟禁用自动编队",
      "reason": "特殊机制要求手动编队"
    },
    "certain_events": {
      "description": "某些活动关卡禁用",
      "reason": "特殊限制或机制冲突"
    },
    "manual_override": {
      "description": "用户可选择手动编队",
      "option": "在设置中禁用自动编队"
    }
  }
}
```

## 战斗列表功能

### 1. 连续战斗
```json
{
  "battle_list": {
    "same_area_battles": {
      "description": "同一地图区域的连续战斗",
      "efficiency": "避免重复加载地图"
    },
    "repeat_count": {
      "type": "number",
      "description": "每个关卡的重复次数",
      "default": 1,
      "range": [1, 999]
    },
    "order_management": {
      "description": "战斗顺序管理",
      "operations": ["添加", "删除", "重新排序"]
    }
  }
}
```

### 2. 停止条件
```json
{
  "stop_conditions": {
    "low_sanity": {
      "description": "理智不足时停止",
      "threshold": "无法开始下一场战斗"
    },
    "battle_failure": {
      "description": "战斗失败时停止",
      "retry_option": "可设置失败重试次数"
    },
    "non_3_star": {
      "description": "未获得3星时停止",
      "optional": "可在设置中禁用此条件"
    },
    "manual_stop": {
      "description": "用户手动停止",
      "always_available": true
    }
  }
}
```

## 作业制作与工具

### 1. 官方作业编辑器
```json
{
  "official_editor": {
    "url": "https://prts.plus/create",
    "features": [
      "可视化地图编辑",
      "干员放置工具",
      "技能时机设置",
      "坐标自动获取",
      "作业预览功能"
    ],
    "export_format": "标准JSON格式"
  }
}
```

### 2. 坐标获取工具
```json
{
  "coordinate_tools": {
    "built_in_editor": {
      "description": "MAA内置地图编辑器",
      "access": "通过调试功能开启"
    },
    "debug_screenshots": {
      "description": "MAA调试地图截图",
      "coordinate_overlay": "显示坐标信息"
    },
    "prts_map": {
      "url": "https://map.ark-nights.com/areas",
      "maa_mode": "支持MAA坐标模式",
      "accuracy": "高精度坐标获取"
    }
  }
}
```

## Function Calling 工具设计

```typescript
{
  name: "maa_copilot_enhanced",
  description: "执行MAA作业，支持JSON文件、网站代码、自动编队、连续战斗等功能",
  parameters: {
    source_type: {
      type: "string",
      enum: ["json_file", "website_code", "file_path"],
      description: "作业来源类型：json_file（JSON内容）、website_code（网站代码）、file_path（文件路径）",
      required: true
    },
    source_content: {
      type: "string",
      description: "作业内容，可以是JSON字符串、网站分享代码或文件路径",
      required: true
    },
    auto_formation: {
      type: "boolean",
      description: "是否启用自动编队",
      default: true
    },
    custom_operators: {
      type: "array",
      items: {
        type: "string"
      },
      description: "自定义添加的干员列表",
      default: []
    },
    supplement_low_trust: {
      type: "boolean",
      description: "是否自动补充低信赖度干员",
      default: true
    },
    repeat_count: {
      type: "number",
      description: "重复执行次数",
      default: 1,
      minimum: 1,
      maximum: 999
    },
    stop_on_failure: {
      type: "boolean",
      description": "战斗失败时是否停止",
      default: true
    },
    stop_on_non_3_star: {
      type: "boolean",
      description: "未获得3星时是否停止",
      default: false
    },
    max_retries: {
      type: "number",
      description: "单次战斗最大重试次数",
      default: 3,
      minimum: 0,
      maximum: 10
    }
  }
}
```

## 作业格式规范

### 1. 基础结构
```json
{
  "copilot_format": {
    "title": {
      "type": "string",
      "description": "作业标题",
      "required": true
    },
    "details": {
      "type": "string", 
      "description": "作业详细描述",
      "required": true
    },
    "opers": {
      "type": "array",
      "description": "干员列表",
      "required": true
    },
    "groups": {
      "type": "array",
      "description": "编队分组",
      "required": true
    },
    "actions": {
      "type": "array",
      "description": "操作序列",
      "required": true
    }
  }
}
```

### 2. 干员配置
```json
{
  "operator_config": {
    "name": {
      "type": "string",
      "description": "干员名称"
    },
    "skill": {
      "type": "number",
      "description": "技能编号（1-3）"
    },
    "skill_usage": {
      "type": "string",
      "enum": ["不自动使用", "自动回复后使用", "自动使用"],
      "description": "技能使用策略"
    },
    "requirements": {
      "elite": {
        "type": "number",
        "description": "精英化等级要求"
      },
      "level": {
        "type": "number", 
        "description": "等级要求"
      },
      "skill_level": {
        "type": "number",
        "description": "技能等级要求"
      }
    }
  }
}
```

## 智能匹配算法

### 1. 干员替换策略
```json
{
  "replacement_strategy": {
    "exact_match": {
      "priority": 1,
      "description": "精确匹配指定干员"
    },
    "class_substitute": {
      "priority": 2,
      "description": "同职业同稀有度替换"
    },
    "functional_equivalent": {
      "priority": 3,
      "description": "功能相似干员替换"
    },
    "skill_based": {
      "priority": 4,
      "description": "基于技能类型的替换"
    }
  }
}
```

### 2. 练度评估
```json
{
  "training_assessment": {
    "level_check": {
      "description": "等级要求检查",
      "tolerance": "允许5级以内差异"
    },
    "skill_check": {
      "description": "技能等级检查",
      "critical": "关键技能必须满足要求"
    },
    "elite_check": {
      "description": "精英化要求检查",
      "mandatory": "通常为强制要求"
    }
  }
}
```

## 自然语言理解示例

- "执行这个作业" + JSON内容 → source_type="json_file", source_content=JSON
- "运行作业代码XYZ123" → source_type="website_code", source_content="XYZ123"
- "刷这个关卡10次" → repeat_count=10
- "作业失败就停止" → stop_on_failure=true
- "必须3星才继续" → stop_on_non_3_star=true

## 最佳实践建议

### 1. 作业选择
```json
{
  "selection_guidelines": {
    "author_reputation": "选择知名作者的作业",
    "update_frequency": "选择经常更新的作业",
    "success_rate": "查看成功率和用户反馈",
    "requirement_match": "确保干员练度满足要求"
  }
}
```

### 2. 使用技巧
```json
{
  "usage_tips": {
    "test_run": "新作业先单次测试",
    "backup_plan": "准备备用作业方案",
    "monitor_progress": "监控执行过程",
    "regular_updates": "定期更新作业文件"
  }
}
```

### 3. 错误处理
```json
{
  "error_handling": {
    "format_errors": "检查JSON格式正确性",
    "missing_operators": "确认所需干员已获得",
    "coordinate_errors": "验证地图坐标准确性",
    "version_conflicts": "检查游戏和MAA版本兼容性"
  }
}
```