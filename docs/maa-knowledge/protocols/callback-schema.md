# MAA 回调协议规范

## 协议概述

MAA回调协议定义了自动化执行过程中的事件通知机制，通过回调函数提供实时的状态更新、任务进度、错误信息等反馈。

## 回调函数原型

### 1. 基础签名
```c
void(ASST_CALL* AsstCallback)(int msg, const char* details, void* custom_arg)
```

### 2. 参数说明
```json
{
  "callback_parameters": {
    "msg": {
      "type": "int",
      "description": "消息类型代码",
      "purpose": "标识事件类型"
    },
    "details": {
      "type": "const char*",
      "description": "JSON格式的详细信息",
      "purpose": "提供事件的具体数据"
    },
    "custom_arg": {
      "type": "void*",
      "description": "用户自定义参数",
      "purpose": "传递用户上下文数据"
    }
  }
}
```

## 消息类型分类

### 1. 全局信息 (Global Info)
```json
{
  "global_messages": {
    "initialization": {
      "code": 0,
      "name": "InitializationComplete",
      "description": "MAA初始化完成",
      "details_format": {
        "version": "MAA版本号",
        "timestamp": "初始化时间戳"
      }
    },
    "error": {
      "code": 1,
      "name": "GlobalError",
      "description": "全局错误事件",
      "details_format": {
        "error_code": "错误代码",
        "error_message": "错误描述",
        "timestamp": "错误发生时间"
      }
    }
  }
}
```

### 2. 连接事件
```json
{
  "connection_events": {
    "connection_info": {
      "code": 10000,
      "name": "ConnectionInfo",
      "description": "连接状态信息",
      "details_format": {
        "what": "ConnectFailed|Connected|Disconnect",
        "uuid": "设备UUID",
        "details": {
          "adb_path": "ADB路径",
          "address": "设备地址",
          "screen_size": [1920, 1080],
          "config": "连接配置"
        }
      }
    },
    "uuid_got": {
      "code": 10001,
      "name": "UUIDGot", 
      "description": "获得设备UUID",
      "details_format": {
        "uuid": "设备唯一标识符"
      }
    },
    "resolution_got": {
      "code": 10002,
      "name": "ResolutionGot",
      "description": "获得屏幕分辨率",
      "details_format": {
        "width": 1920,
        "height": 1080
      }
    }
  }
}
```

### 3. 任务链事件
```json
{
  "task_chain_events": {
    "chain_start": {
      "code": 20000,
      "name": "TaskChainStart",
      "description": "任务链开始执行",
      "details_format": {
        "taskchain": "任务链名称",
        "uuid": "执行UUID"
      }
    },
    "chain_completed": {
      "code": 20001,
      "name": "TaskChainCompleted",
      "description": "任务链执行完成",
      "details_format": {
        "taskchain": "任务链名称",
        "uuid": "执行UUID",
        "finished_tasks": ["已完成的任务列表"]
      }
    },
    "chain_error": {
      "code": 20002,
      "name": "TaskChainError",
      "description": "任务链执行错误",
      "details_format": {
        "taskchain": "任务链名称",
        "uuid": "执行UUID",
        "error": "错误信息"
      }
    }
  }
}
```

### 4. 子任务事件
```json
{
  "sub_task_events": {
    "task_start": {
      "code": 30000,
      "name": "SubTaskStart",
      "description": "子任务开始执行",
      "details_format": {
        "subtask": "子任务名称",
        "taskchain": "所属任务链",
        "class": "任务类别"
      }
    },
    "task_completed": {
      "code": 30001,
      "name": "SubTaskCompleted",
      "description": "子任务执行完成",
      "details_format": {
        "subtask": "子任务名称", 
        "taskchain": "所属任务链",
        "result": "执行结果"
      }
    },
    "task_error": {
      "code": 30002,
      "name": "SubTaskError",
      "description": "子任务执行错误",
      "details_format": {
        "subtask": "子任务名称",
        "taskchain": "所属任务链",
        "error": "错误信息"
      }
    },
    "task_extra_info": {
      "code": 30003,
      "name": "SubTaskExtraInfo",
      "description": "子任务额外信息",
      "details_format": {
        "subtask": "子任务名称",
        "what": "信息类型",
        "details": "具体信息内容"
      }
    }
  }
}
```

## 游戏特定事件

### 1. 公开招募事件
```json
{
  "recruit_events": {
    "recruit_tag_detected": {
      "code": 40000,
      "name": "RecruitTagDetected",
      "description": "检测到公招标签",
      "details_format": {
        "tags": ["检测到的标签列表"],
        "level": "稀有度等级",
        "confidence": "识别置信度"
      }
    },
    "recruit_special_tag": {
      "code": 40001,
      "name": "RecruitSpecialTag",
      "description": "检测到特殊标签",
      "details_format": {
        "tag": "特殊标签名称",
        "type": "标签类型(高资/资深等)",
        "recommendation": "推荐操作"
      }
    },
    "recruit_result": {
      "code": 40002,
      "name": "RecruitResult",
      "description": "招募结果",
      "details_format": {
        "operator": "获得的干员",
        "rarity": "稀有度",
        "is_new": "是否为新干员"
      }
    }
  }
}
```

### 2. 基建管理事件
```json
{
  "infrastructure_events": {
    "facility_info": {
      "code": 50000,
      "name": "InfrastructureFacilityInfo",
      "description": "基建设施信息",
      "details_format": {
        "facility": "设施名称",
        "operators": ["当前干员列表"],
        "efficiency": "当前效率",
        "mood": "心情状态"
      }
    },
    "production_completed": {
      "code": 50001,
      "name": "ProductionCompleted", 
      "description": "生产完成",
      "details_format": {
        "facility": "设施名称",
        "product": "生产的物品",
        "quantity": "数量"
      }
    },
    "shift_change": {
      "code": 50002,
      "name": "ShiftChange",
      "description": "换班操作",
      "details_format": {
        "facility": "设施名称",
        "old_operators": ["原干员列表"],
        "new_operators": ["新干员列表"]
      }
    }
  }
}
```

### 3. 战斗相关事件
```json
{
  "battle_events": {
    "stage_drop": {
      "code": 60000,
      "name": "StageDropDetected",
      "description": "检测到关卡掉落",
      "details_format": {
        "stage": "关卡代码",
        "drops": [
          {
            "item": "物品名称",
            "quantity": "数量",
            "rarity": "稀有度"
          }
        ],
        "sanity_cost": "理智消耗"
      }
    },
    "battle_start": {
      "code": 60001,
      "name": "BattleStart",
      "description": "战斗开始",
      "details_format": {
        "stage": "关卡代码",
        "team": ["队伍干员列表"],
        "formation": "编队信息"
      }
    },
    "battle_end": {
      "code": 60002,
      "name": "BattleEnd",
      "description": "战斗结束",
      "details_format": {
        "stage": "关卡代码",
        "result": "Success|Failed|Retreat",
        "stars": "获得星数",
        "time_cost": "耗时"
      }
    }
  }
}
```

### 4. 干员识别事件
```json
{
  "operator_events": {
    "operator_detected": {
      "code": 70000,
      "name": "OperatorDetected",
      "description": "检测到干员信息",
      "details_format": {
        "name": "干员名称",
        "rarity": "稀有度",
        "elite": "精英化等级",
        "level": "等级",
        "skill_level": "技能等级",
        "potential": "潜能等级"
      }
    },
    "operator_box_info": {
      "code": 70001,
      "name": "OperatorBoxInfo",
      "description": "干员仓库信息",
      "details_format": {
        "total_count": "总干员数",
        "new_operators": ["新获得的干员"],
        "updated_operators": ["更新的干员信息"]
      }
    }
  }
}
```

## Web请求事件

### 1. 网络请求
```json
{
  "web_events": {
    "request_start": {
      "code": 80000,
      "name": "WebRequestStart",
      "description": "Web请求开始",
      "details_format": {
        "url": "请求URL",
        "method": "HTTP方法",
        "headers": "请求头信息"
      }
    },
    "request_completed": {
      "code": 80001,
      "name": "WebRequestCompleted",
      "description": "Web请求完成",
      "details_format": {
        "url": "请求URL",
        "status_code": "HTTP状态码",
        "response_size": "响应大小"
      }
    },
    "request_error": {
      "code": 80002,
      "name": "WebRequestError",
      "description": "Web请求错误",
      "details_format": {
        "url": "请求URL",
        "error_code": "错误代码",
        "error_message": "错误信息"
      }
    }
  }
}
```

## 实时状态跟踪

### 1. 进度更新
```json
{
  "progress_tracking": {
    "task_progress": {
      "current_task": "当前执行的任务",
      "completed_tasks": "已完成任务数",
      "total_tasks": "总任务数",
      "progress_percentage": "完成百分比"
    },
    "resource_usage": {
      "sanity_used": "已使用理智",
      "items_obtained": "获得的物品",
      "time_elapsed": "已耗时间"
    }
  }
}
```

### 2. 状态监控
```json
{
  "status_monitoring": {
    "system_status": {
      "connection_stable": "连接是否稳定",
      "performance_metrics": "性能指标",
      "error_count": "错误次数"
    },
    "game_status": {
      "current_scene": "当前游戏场景",
      "sanity_remaining": "剩余理智",
      "inventory_status": "库存状态"
    }
  }
}
```

## Function Calling 回调映射

### 1. 事件到响应的映射
```json
{
  "event_response_mapping": {
    "task_completion": {
      "callback_event": "TaskChainCompleted",
      "function_response": {
        "status": "success",
        "message": "任务执行完成",
        "results": "执行结果详情"
      }
    },
    "error_handling": {
      "callback_event": "TaskChainError",
      "function_response": {
        "status": "error", 
        "error_code": "错误代码",
        "error_message": "错误描述",
        "suggestion": "建议的解决方案"
      }
    },
    "progress_update": {
      "callback_event": "SubTaskCompleted",
      "function_response": {
        "status": "in_progress",
        "progress": "进度信息",
        "current_step": "当前步骤"
      }
    }
  }
}
```

### 2. 实时通知机制
```json
{
  "real_time_notifications": {
    "special_events": {
      "high_priority": ["RecruitSpecialTag", "GlobalError", "ConnectionFailed"],
      "medium_priority": ["StageDropDetected", "OperatorDetected"],
      "low_priority": ["SubTaskStart", "SubTaskCompleted"]
    },
    "notification_format": {
      "timestamp": "事件发生时间",
      "event_type": "事件类型",
      "severity": "严重程度",
      "message": "用户友好的消息",
      "details": "技术细节"
    }
  }
}
```

## 错误处理与重试

### 1. 错误分类
```json
{
  "error_classification": {
    "recoverable_errors": {
      "description": "可恢复的错误",
      "examples": ["网络超时", "临时识别失败", "设备暂时无响应"],
      "handling": "自动重试或用户确认后重试"
    },
    "fatal_errors": {
      "description": "致命错误",
      "examples": ["设备断开", "游戏崩溃", "权限不足"],
      "handling": "停止执行并通知用户"
    },
    "warning_events": {
      "description": "警告事件",
      "examples": ["理智不足", "识别置信度低", "任务超时"],
      "handling": "记录日志并继续执行"
    }
  }
}
```

### 2. 重试机制
```json
{
  "retry_mechanism": {
    "automatic_retry": {
      "max_attempts": 3,
      "retry_delay": "递增延时",
      "backoff_strategy": "指数退避"
    },
    "user_intervention": {
      "prompt_user": "提示用户手动处理",
      "wait_for_confirmation": "等待用户确认",
      "provide_options": "提供多种处理选项"
    }
  }
}
```

## 集成最佳实践

### 1. 回调处理建议
```json
{
  "callback_best_practices": {
    "performance": {
      "non_blocking": "回调处理不应阻塞主线程",
      "efficient_parsing": "高效解析JSON消息",
      "async_processing": "异步处理复杂逻辑"
    },
    "reliability": {
      "error_resilience": "回调处理应具备错误恢复能力",
      "state_consistency": "保持状态一致性",
      "resource_cleanup": "及时清理资源"
    }
  }
}
```

### 2. 日志记录
```json
{
  "logging_strategy": {
    "log_levels": {
      "debug": "详细的调试信息",
      "info": "一般信息事件",
      "warning": "警告级别事件",
      "error": "错误级别事件"
    },
    "log_format": {
      "timestamp": "ISO 8601格式时间戳",
      "level": "日志级别",
      "event_type": "事件类型",
      "message": "日志消息",
      "context": "上下文信息"
    }
  }
}
```