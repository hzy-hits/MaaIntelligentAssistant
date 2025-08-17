# MAA 集成指南

## 集成概述

MAA集成指南提供了完整的API接口和配置方法，支持多种任务类型的自动化执行，包含实例配置、任务管理、参数设置等核心功能。

## 核心API接口

### 1. 任务管理API
```c
{
  "task_management": {
    "AsstAppendTask": {
      "description": "添加任务到执行队列",
      "signature": "AsstTaskId AsstAppendTask(AsstHandle handle, const char* type, const char* params)",
      "parameters": {
        "handle": "MAA实例句柄",
        "type": "任务类型",
        "params": "JSON格式的任务参数"
      },
      "return": "任务ID，用于后续操作"
    },
    "AsstSetTaskParams": {
      "description": "动态修改任务参数",
      "signature": "AsstBool AsstSetTaskParams(AsstHandle handle, AsstTaskId id, const char* params)",
      "parameters": {
        "handle": "MAA实例句柄",
        "id": "任务ID",
        "params": "新的JSON参数"
      },
      "return": "操作是否成功"
    }
  }
}
```

### 2. 实例配置API
```c
{
  "instance_configuration": {
    "AsstSetInstanceOption": {
      "description": "设置实例级别配置选项",
      "signature": "AsstBool AsstSetInstanceOption(AsstHandle handle, AsstInstanceOptionKey key, const char* value)",
      "parameters": {
        "handle": "MAA实例句柄",
        "key": "配置项键名",
        "value": "配置值"
      },
      "supported_options": {
        "touch_mode": "触摸模式设置",
        "deployment_pause": "部署暂停控制",
        "adb_lite_enabled": "ADB轻量模式",
        "kill_adb_on_exit": "退出时终止ADB"
      }
    }
  }
}
```

## 支持的任务类型

### 1. StartUp - 游戏启动
```json
{
  "startup_task": {
    "type": "StartUp",
    "description": "自动启动游戏和账号切换",
    "parameters": {
      "client_type": {
        "type": "string",
        "description": "客户端类型",
        "options": ["Official", "Bilibili", "txwy", "YoStarEN", "YoStarJP", "YoStarKR"],
        "default": "Official"
      },
      "start_game_enabled": {
        "type": "boolean",
        "description": "是否启动游戏",
        "default": true
      },
      "account_name": {
        "type": "string",
        "description": "账号名称（用于切换账号）",
        "default": ""
      }
    }
  }
}
```

### 2. Fight - 自动战斗
```json
{
  "fight_task": {
    "type": "Fight",
    "description": "自动战斗系统",
    "parameters": {
      "stage": {
        "type": "string",
        "description": "关卡代码",
        "example": "1-7"
      },
      "medicine": {
        "type": "number",
        "description": "理智药剂使用数量",
        "default": 0
      },
      "stone": {
        "type": "number",
        "description": "源石使用数量",
        "default": 0
      },
      "times": {
        "type": "number",
        "description": "战斗次数",
        "default": 1
      },
      "drops": {
        "type": "object",
        "description": "掉落物品统计配置",
        "properties": {
          "enable": "是否启用掉落统计",
          "upload_penguin": "是否上传企鹅物流",
          "upload_yituliu": "是否上传一图流"
        }
      }
    }
  }
}
```

### 3. Recruit - 公开招募
```json
{
  "recruit_task": {
    "type": "Recruit",
    "description": "公开招募自动化",
    "parameters": {
      "refresh": {
        "type": "boolean",
        "description": "是否刷新标签",
        "default": false
      },
      "select": {
        "type": "array",
        "description": "指定选择的标签组合",
        "items": {"type": "number"},
        "example": [3, 4]
      },
      "confirm": {
        "type": "array",
        "description": "确认选择的标签",
        "items": {"type": "number"},
        "example": [3, 4]
      },
      "times": {
        "type": "number",
        "description": "招募次数",
        "default": 0
      },
      "set_time": {
        "type": "boolean",
        "description": "是否设置招募时间",
        "default": true
      },
      "expedite": {
        "type": "boolean",
        "description": "是否使用加急许可",
        "default": false
      }
    }
  }
}
```

### 4. Infrast - 基建管理
```json
{
  "infrast_task": {
    "type": "Infrast",
    "description": "基建设施管理",
    "parameters": {
      "facility": {
        "type": "array",
        "description": "要管理的设施列表",
        "items": {"type": "string"},
        "options": ["Mfg", "Trade", "Power", "Control", "Reception", "Office", "Dorm"]
      },
      "drones": {
        "type": "string",
        "description": "无人机使用策略",
        "options": ["_NotUse", "Money", "SyntheticJade", "CombatRecord", "PureGold", "OriginStone", "Chip"],
        "default": "_NotUse"
      },
      "threshold": {
        "type": "number",
        "description": "心情阈值",
        "default": 0.3,
        "range": [0.0, 1.0]
      },
      "replenish": {
        "type": "boolean",
        "description": "是否自动补充源石",
        "default": false
      }
    }
  }
}
```

### 5. Roguelike - 肉鸽模式
```json
{
  "roguelike_task": {
    "type": "Roguelike",
    "description": "集成战略自动化",
    "parameters": {
      "theme": {
        "type": "string",
        "description": "主题选择",
        "options": ["Phantom", "Mizuki", "Sami", "Sarkaz"],
        "default": "Phantom"
      },
      "mode": {
        "type": "number",
        "description": "模式选择",
        "options": [0, 1, 2, 3, 4],
        "default": 0
      },
      "starts_count": {
        "type": "number",
        "description": "开局干员数量",
        "default": 2
      },
      "investment_enabled": {
        "type": "boolean",
        "description": "是否启用投资系统",
        "default": true
      },
      "investments_count": {
        "type": "number",
        "description": "投资次数",
        "default": 999
      },
      "stop_when_investment_full": {
        "type": "boolean",
        "description": "投资满时是否停止",
        "default": false
      }
    }
  }
}
```

### 6. Copilot - 作业执行
```json
{
  "copilot_task": {
    "type": "Copilot",
    "description": "执行作业文件",
    "parameters": {
      "filename": {
        "type": "string",
        "description": "作业文件路径",
        "example": "./copilot.json"
      },
      "formation": {
        "type": "boolean",
        "description": "是否自动编队",
        "default": false
      }
    }
  }
}
```

## 实例配置选项

### 1. 触摸模式配置
```json
{
  "touch_mode_options": {
    "minitouch": {
      "description": "使用minitouch协议",
      "compatibility": "最佳兼容性",
      "performance": "标准性能"
    },
    "maatouch": {
      "description": "使用maatouch协议",
      "compatibility": "良好兼容性",
      "performance": "优化性能"
    },
    "adb": {
      "description": "使用原生ADB输入",
      "compatibility": "通用兼容性",
      "performance": "较低性能"
    }
  }
}
```

### 2. ADB配置选项
```json
{
  "adb_configuration": {
    "adb_lite_enabled": {
      "description": "启用ADB轻量模式",
      "benefits": ["减少资源占用", "提高响应速度"],
      "default": false
    },
    "kill_adb_on_exit": {
      "description": "退出时终止ADB进程",
      "purpose": "避免进程残留",
      "default": false
    },
    "adb_path": {
      "description": "自定义ADB路径",
      "format": "绝对路径",
      "example": "/usr/local/bin/adb"
    }
  }
}
```

## 多客户端支持

### 1. 客户端类型
```json
{
  "supported_clients": {
    "Official": {
      "name": "官方服",
      "region": "中国大陆",
      "language": "简体中文"
    },
    "Bilibili": {
      "name": "B服",
      "region": "中国大陆",
      "language": "简体中文"
    },
    "txwy": {
      "name": "应用宝/TapTap",
      "region": "中国大陆",
      "language": "简体中文"
    },
    "YoStarEN": {
      "name": "英文国际服",
      "region": "国际",
      "language": "英文"
    },
    "YoStarJP": {
      "name": "日文服",
      "region": "日本",
      "language": "日文"
    },
    "YoStarKR": {
      "name": "韩文服",
      "region": "韩国",
      "language": "韩文"
    }
  }
}
```

### 2. 多语言配置
```json
{
  "multi_language_support": {
    "resource_switching": {
      "description": "根据客户端类型自动切换资源",
      "automatic": true
    },
    "text_recognition": {
      "description": "支持多语言OCR识别",
      "languages": ["简体中文", "英文", "日文", "韩文"]
    },
    "ui_elements": {
      "description": "适配不同语言的UI元素",
      "adaptive": true
    }
  }
}
```

## 集成最佳实践

### 1. 初始化流程
```json
{
  "initialization_workflow": [
    "1. 创建MAA实例",
    "2. 设置回调函数",
    "3. 加载资源文件",
    "4. 连接设备",
    "5. 配置实例选项",
    "6. 开始添加任务"
  ]
}
```

### 2. 任务链管理
```json
{
  "task_chain_management": {
    "sequential_execution": {
      "description": "任务按添加顺序依次执行",
      "example": "StartUp → Fight → Infrast → Recruit"
    },
    "conditional_execution": {
      "description": "根据前置任务结果决定后续任务",
      "implementation": "通过回调函数判断任务结果"
    },
    "parallel_execution": {
      "description": "某些任务可以并行执行",
      "note": "需要确保任务间无冲突"
    }
  }
}
```

### 3. 错误处理策略
```json
{
  "error_handling_strategy": {
    "task_failure": {
      "detection": "通过回调函数检测任务失败",
      "response": "记录错误信息，决定是否重试或跳过"
    },
    "connection_loss": {
      "detection": "监控连接状态回调",
      "response": "尝试重新连接，必要时重启任务"
    },
    "resource_shortage": {
      "detection": "监控游戏资源状态",
      "response": "调整任务参数或暂停执行"
    }
  }
}
```

## Function Calling 集成映射

### 1. API到Function Calling的映射
```json
{
  "api_function_mapping": {
    "direct_mapping": {
      "AsstAppendTask": "直接映射到Function Calling的tool execution",
      "task_types": "映射到tool parameters中的task_type字段",
      "task_params": "映射到tool parameters中的具体参数"
    },
    "abstraction_layer": {
      "natural_language": "将自然语言转换为具体的任务参数",
      "parameter_validation": "验证参数的合法性和完整性",
      "smart_defaults": "为缺失的参数提供智能默认值"
    }
  }
}
```

### 2. 回调到响应的转换
```json
{
  "callback_response_conversion": {
    "real_time_updates": {
      "progress_tracking": "将任务进度转换为Function Calling的进度响应",
      "status_updates": "实时更新Function Calling的执行状态"
    },
    "result_formatting": {
      "success_response": "格式化成功响应，包含执行结果",
      "error_response": "格式化错误响应，提供用户友好的错误信息",
      "intermediate_results": "提供中间结果和进度信息"
    }
  }
}
```

### 3. 配置管理
```json
{
  "configuration_management": {
    "default_configs": {
      "description": "为不同Function Calling工具提供默认配置",
      "customization": "允许用户自定义常用配置"
    },
    "profile_management": {
      "description": "支持多配置文件管理",
      "use_cases": ["不同账号", "不同游戏模式", "不同自动化策略"]
    }
  }
}
```

## 部署和运维

### 1. 部署架构
```json
{
  "deployment_architecture": {
    "standalone_mode": {
      "description": "单机部署模式",
      "components": ["MAA Core", "Function Calling Server", "Resource Files"],
      "suitable_for": "个人用户和小规模使用"
    },
    "distributed_mode": {
      "description": "分布式部署模式",
      "components": ["Controller Service", "Worker Nodes", "Shared Storage"],
      "suitable_for": "多用户和高并发场景"
    }
  }
}
```

### 2. 监控和维护
```json
{
  "monitoring_maintenance": {
    "health_checks": {
      "connection_status": "定期检查设备连接状态",
      "resource_usage": "监控系统资源使用情况",
      "task_success_rate": "统计任务成功率"
    },
    "log_management": {
      "structured_logging": "使用结构化日志格式",
      "log_rotation": "定期轮转日志文件",
      "centralized_collection": "集中收集和分析日志"
    },
    "performance_optimization": {
      "resource_optimization": "优化内存和CPU使用",
      "task_scheduling": "优化任务调度策略",
      "cache_management": "有效管理缓存数据"
    }
  }
}
```