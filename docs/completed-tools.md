# MAA 已完成工具文档

本文档记录已经完成实现并测试通过的MAA Function Calling工具。

## 核心游戏功能工具 (已完成)

### 1. maa_startup - 游戏启动管理
**状态**: ✅ 已完成并测试通过  
**文件**: `src/mcp_tools/maa_startup.rs`

```typescript
{
  name: "maa_startup",
  description: "管理游戏启动、账号切换、客户端选择等功能",
  parameters: {
    action: {
      type: "string",
      enum: ["start_game", "switch_account", "check_status"],
      description: "启动操作类型"
    },
    client_type: {
      type: "string", 
      enum: ["Official", "Bilibili", "Txwy", "YoStarEN", "YoStarJP", "YoStarKR"],
      description: "客户端类型"
    },
    account: {
      type: "string",
      description: "账号标识（支持部分匹配）"
    },
    start_emulator: {
      type: "boolean",
      description: "是否尝试启动模拟器"
    }
  }
}
```

**实现特性**:
- 6种客户端类型支持（官服、B服、TxWy、海外服等）
- 智能账号匹配和切换
- 模拟器启动检测和管理
- 完整的启动状态监控

**测试示例**:
```bash
curl -X POST http://localhost:8080/call \
  -H "Content-Type: application/json" \
  -d '{
    "function_call": {
      "name": "maa_startup",
      "arguments": {
        "action": "start_game",
        "client_type": "Official",
        "start_emulator": true
      }
    }
  }'
```

### 2. maa_combat_enhanced - 增强战斗系统
**状态**: ✅ 已完成并测试通过  
**文件**: `src/mcp_tools/maa_combat.rs`

```typescript
{
  name: "maa_combat_enhanced",
  description: "执行自动战斗，支持复杂策略、资源管理、掉落统计等",
  parameters: {
    stage: {
      type: "string",
      description: "关卡代码或自然语言描述（如'狗粮'、'龙门币本'、'1-7'）"
    },
    strategy: {
      type: "object",
      properties: {
        mode: {
          type: "string",
          enum: ["times", "sanity", "material", "infinite"],
          description: "战斗模式"
        },
        target_value: {
          type: "number",
          description: "目标值（次数/理智/材料数量）"
        }
      }
    },
    resources: {
      type: "object", 
      properties: {
        medicine: {
          type: "number",
          description: "理智药剂使用数量"
        },
        stone: {
          type: "number", 
          description: "源石使用数量"
        },
        dr_grandet: {
          type: "boolean",
          description: "Dr.Grandet模式（等待理智恢复1点后再使用源石）"
        }
      }
    }
  }
}
```

**实现特性**:
- 智能关卡名称解析（"狗粮"→"1-7", "龙门币本"→"CE-5"等）
- 4种战斗模式（次数、理智、材料、无限）
- 复杂资源管理（理智药剂、源石、Dr.Grandet模式）
- 掉落统计和数据上报
- 代理连战和失败恢复

**测试示例**:
```bash
curl -X POST http://localhost:8080/call \
  -H "Content-Type: application/json" \
  -d '{
    "function_call": {
      "name": "maa_combat_enhanced",
      "arguments": {
        "stage": "狗粮",
        "strategy": {"mode": "times", "target_value": 10},
        "resources": {"medicine": 3, "stone": 1, "dr_grandet": true}
      }
    }
  }'
```

### 3. maa_recruit_enhanced - 智能招募管理
**状态**: ✅ 已完成并测试通过  
**文件**: `src/mcp_tools/maa_recruit.rs`

```typescript
{
  name: "maa_recruit_enhanced",
  description: "智能公开招募管理，支持标签分析、策略优化、结果预测",
  parameters: {
    mode: {
      type: "string",
      enum: ["auto", "manual", "smart"],
      description: "招募模式：自动、手动确认、智能决策"
    },
    config: {
      type: "object",
      properties: {
        enable: {
          type: "boolean",
          description: "是否启用招募任务"
        },
        refresh: {
          type: "boolean", 
          description: "是否自动刷新招募标签"
        },
        times: {
          type: "number",
          description: "招募次数限制，0表示无限制"
        },
        min_star: {
          type: "number",
          enum: [1, 2, 3, 4, 5, 6],
          description: "最低星级要求"
        },
        skip_robot: {
          type: "boolean",
          description: "是否跳过机器人标签组合"
        }
      }
    },
    timing: {
      type: "object",
      properties: {
        set_time: {
          type: "boolean",
          description: "是否设置招募时间"
        },
        expedite: {
          type: "boolean",
          description: "是否使用加急许可证"
        },
        expedite_times: {
          type: "number",
          description: "加急许可证使用次数限制"
        }
      }
    },
    strategy: {
      type: "object",
      properties: {
        priority_rare: {
          type: "boolean",
          description: "是否优先选择稀有标签组合"
        },
        notify_rare: {
          type: "boolean",
          description: "稀有标签出现时是否发送通知"
        },
        target_operators: {
          type: "array",
          items: {"type": "string"},
          description: "目标干员列表"
        }
      }
    }
  }
}
```

**实现特性**:
- 3种招募模式（自动、手动、智能）
- 分星级招募时间管理（1⭐→01:00:00, 6⭐→08:20:00）
- 加急许可证智能使用
- 稀有标签优先级和通知
- 目标干员定向招募
- 机器人标签过滤

**测试示例**:
```bash
curl -X POST http://localhost:8080/call \
  -H "Content-Type: application/json" \
  -d '{
    "function_call": {
      "name": "maa_recruit_enhanced",
      "arguments": {
        "mode": "smart",
        "config": {"times": 5, "min_star": 4},
        "strategy": {"target_operators": ["银灰", "史尔特尔"]}
      }
    }
  }'
```

## 实现模式和架构

### 统一实现模式
每个工具都遵循相同的架构模式：

1. **参数结构定义** - 使用Rust结构体定义复杂参数
2. **参数解析器** - `parse_arguments()` 方法处理JSON输入
3. **任务执行器** - `execute()` 方法处理核心逻辑
4. **MAA参数构建** - `build_maa_params()` 生成MAA Core格式
5. **自然语言支持** - `parse_natural_language_*()` 函数

### 错误处理机制
- 详细的参数验证和错误信息
- 分层错误处理（解析错误、执行错误、MAA错误）
- 完整的日志追踪和性能监控

### 测试验证
所有工具都通过以下测试：
- ✅ 参数解析正确性测试
- ✅ 默认值处理测试
- ✅ 错误处理测试
- ✅ MAA参数生成测试
- ✅ HTTP API集成测试

---

**文档更新时间**: 2025-08-17  
**已完成工具数量**: 3/16  
**下一个目标**: maa_infrastructure_enhanced