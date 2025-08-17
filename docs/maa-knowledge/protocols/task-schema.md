# MAA 任务协议规范

## 协议概述

MAA任务协议是基于JSON的任务配置系统，支持复杂的自动化任务序列定义，包含多种任务类型、识别算法和动作执行机制。

## 核心概念

### 1. 任务类型分类
```json
{
  "task_types": {
    "template_task": {
      "prefix": "@",
      "description": "模板任务，可被其他任务继承",
      "example": "@BaseClick",
      "usage": "定义通用的任务模板"
    },
    "base_task": {
      "prefix": "",
      "description": "基础任务，具体的执行单元",
      "example": "StartGame",
      "usage": "实际执行的任务"
    },
    "virtual_task": {
      "prefix": "#",
      "description": "虚拟任务，运行时动态生成",
      "example": "#DynamicClick",
      "usage": "动态任务生成"
    }
  }
}
```

### 2. 任务表达式
```json
{
  "task_expressions": {
    "inheritance": {
      "symbol": "@",
      "description": "继承模板任务的属性",
      "example": "Task@Template"
    },
    "virtual": {
      "symbol": "#",
      "description": "创建虚拟任务",
      "example": "#VirtualTask"
    },
    "append": {
      "symbol": "*",
      "description": "在任务名称后添加内容",
      "example": "Task*Suffix"
    },
    "prepend": {
      "symbol": "+",
      "description": "在任务名称前添加内容",
      "example": "+PrefixTask"
    },
    "replace": {
      "symbol": "^",
      "description": "替换任务名称中的内容",
      "example": "Task^Old^New"
    }
  }
}
```

## 任务配置结构

### 1. 基础参数
```json
{
  "basic_parameters": {
    "TaskName": {
      "type": "string",
      "description": "任务唯一标识符",
      "required": true,
      "example": "StartButton"
    },
    "algorithm": {
      "type": "string",
      "enum": ["MatchTemplate", "OcrDetect", "FeatureMatch", "JustReturn", "Hash"],
      "description": "识别算法类型",
      "required": true
    },
    "action": {
      "type": "string", 
      "enum": ["DoNothing", "Click", "Swipe", "Key", "Input", "StartApp", "StopApp"],
      "description": "执行的动作类型",
      "default": "DoNothing"
    },
    "next": {
      "type": "array",
      "items": {"type": "string"},
      "description": "后续执行的任务列表",
      "default": []
    }
  }
}
```

### 2. 识别参数
```json
{
  "recognition_parameters": {
    "roi": {
      "type": "array",
      "items": {"type": "number"},
      "description": "识别区域 [x, y, width, height]",
      "example": [0, 0, 1920, 1080]
    },
    "template": {
      "type": "string",
      "description": "模板图片文件名",
      "example": "start_button.png"
    },
    "text": {
      "type": "array",
      "items": {"type": "string"},
      "description": "OCR识别的目标文字",
      "example": ["开始", "Start"]
    },
    "threshold": {
      "type": "number",
      "description": "识别阈值 (0.0-1.0)",
      "default": 0.8,
      "minimum": 0.0,
      "maximum": 1.0
    }
  }
}
```

### 3. 动作参数
```json
{
  "action_parameters": {
    "target": {
      "type": "array",
      "items": {"type": "number"},
      "description": "动作目标坐标 [x, y]",
      "example": [960, 540]
    },
    "begin": {
      "type": "array", 
      "items": {"type": "number"},
      "description": "滑动起始坐标 [x, y]",
      "example": [100, 500]
    },
    "end": {
      "type": "array",
      "items": {"type": "number"},
      "description": "滑动结束坐标 [x, y]",
      "example": [900, 500]
    },
    "input_text": {
      "type": "string",
      "description": "要输入的文字内容",
      "example": "username"
    }
  }
}
```

## 识别算法详解

### 1. 图像匹配算法
```json
{
  "match_template": {
    "algorithm": "MatchTemplate",
    "required_params": ["template"],
    "optional_params": ["threshold", "roi", "green_mask"],
    "description": "基于模板图片的匹配识别",
    "use_cases": ["按钮识别", "图标检测", "界面状态判断"]
  }
}
```

### 2. OCR文字识别
```json
{
  "ocr_detect": {
    "algorithm": "OcrDetect", 
    "required_params": ["text"],
    "optional_params": ["roi", "expected", "replace"],
    "description": "基于OCR的文字识别",
    "use_cases": ["文字按钮", "状态文字", "数值读取"]
  }
}
```

### 3. 特征点匹配
```json
{
  "feature_match": {
    "algorithm": "FeatureMatch",
    "required_params": ["template"],
    "optional_params": ["threshold", "roi"],
    "description": "基于特征点的图像匹配",
    "use_cases": ["复杂场景识别", "角度变化适应"]
  }
}
```

### 4. 直接执行
```json
{
  "just_return": {
    "algorithm": "JustReturn",
    "required_params": [],
    "description": "跳过识别直接执行动作",
    "use_cases": ["固定操作", "延时等待", "条件分支"]
  }
}
```

## 动作类型详解

### 1. 点击动作
```json
{
  "click_action": {
    "action": "Click",
    "parameters": {
      "target": "点击坐标，可以是固定坐标或识别到的位置",
      "target_offset": "相对于识别位置的偏移量",
      "pre_delay": "点击前延时（毫秒）",
      "post_delay": "点击后延时（毫秒）"
    }
  }
}
```

### 2. 滑动动作
```json
{
  "swipe_action": {
    "action": "Swipe",
    "parameters": {
      "begin": "滑动起始坐标",
      "end": "滑动结束坐标",
      "duration": "滑动持续时间（毫秒）",
      "pre_delay": "滑动前延时",
      "post_delay": "滑动后延时"
    }
  }
}
```

### 3. 输入动作
```json
{
  "input_action": {
    "action": "Input",
    "parameters": {
      "input_text": "要输入的文字内容",
      "pre_delay": "输入前延时",
      "post_delay": "输入后延时"
    }
  }
}
```

## 高级功能

### 1. 任务继承
```json
{
  "task_inheritance": {
    "base_template": {
      "@BaseClick": {
        "algorithm": "MatchTemplate",
        "action": "Click",
        "pre_delay": 500,
        "post_delay": 1000
      }
    },
    "derived_task": {
      "StartButton@BaseClick": {
        "template": "start_button.png",
        "target": [960, 540]
      }
    }
  }
}
```

### 2. 条件分支
```json
{
  "conditional_execution": {
    "condition_check": {
      "CheckState": {
        "algorithm": "MatchTemplate",
        "template": "loading.png",
        "next": ["WaitLoading", "StartGame"]
      }
    },
    "branch_logic": {
      "description": "根据识别结果执行不同的后续任务",
      "success_path": "识别成功时的任务序列",
      "failure_path": "识别失败时的任务序列"
    }
  }
}
```

### 3. 循环控制
```json
{
  "loop_control": {
    "max_times": {
      "type": "number",
      "description": "最大执行次数",
      "default": 20
    },
    "timeout": {
      "type": "number",
      "description": "超时时间（毫秒）",
      "default": 10000
    },
    "interrupt_tasks": {
      "type": "array",
      "description": "中断任务列表",
      "example": ["ErrorDialog", "NetworkError"]
    }
  }
}
```

## 配置文件结构

### 1. 单文件配置
```json
{
  "single_file_config": {
    "filename": "task_config.json",
    "structure": {
      "TaskName1": {"algorithm": "...", "action": "..."},
      "TaskName2": {"algorithm": "...", "action": "..."}
    }
  }
}
```

### 2. 多文件配置
```json
{
  "multi_file_config": {
    "directory_structure": {
      "tasks/": {
        "main.json": "主要任务定义",
        "templates.json": "模板任务定义", 
        "common.json": "通用任务定义"
      }
    },
    "loading_order": "按文件名字典序加载"
  }
}
```

## 运行时机制

### 1. 任务执行流程
```json
{
  "execution_flow": [
    "1. 加载任务配置文件",
    "2. 解析任务依赖关系",
    "3. 执行起始任务",
    "4. 进行图像/文字识别",
    "5. 执行对应动作",
    "6. 根据结果选择后续任务",
    "7. 重复步骤4-6直到任务完成"
  ]
}
```

### 2. 错误处理
```json
{
  "error_handling": {
    "recognition_failure": {
      "description": "识别失败时的处理",
      "options": ["重试", "跳过", "终止", "执行备用任务"]
    },
    "action_failure": {
      "description": "动作执行失败时的处理",
      "options": ["重试", "记录日志", "触发异常处理流程"]
    },
    "timeout_handling": {
      "description": "超时处理机制",
      "behavior": "超时后终止当前任务，执行cleanup任务"
    }
  }
}
```

## Function Calling 映射策略

### 1. 任务类型映射
```json
{
  "task_mapping": {
    "simple_tasks": {
      "description": "简单任务直接映射到Function Calling参数",
      "example": "点击按钮 → Click action + template"
    },
    "complex_tasks": {
      "description": "复杂任务序列映射到任务链",
      "example": "完整流程 → 多个任务的组合"
    },
    "dynamic_tasks": {
      "description": "动态任务需要运行时生成",
      "example": "条件分支 → 基于状态的任务选择"
    }
  }
}
```

### 2. 参数转换规则
```json
{
  "parameter_conversion": {
    "coordinate_system": {
      "source": "MAA绝对坐标",
      "target": "相对坐标或命名区域",
      "conversion": "基于屏幕分辨率和ROI"
    },
    "template_resources": {
      "source": "本地图片文件",
      "target": "资源标识符",
      "conversion": "文件名到资源ID的映射"
    },
    "text_recognition": {
      "source": "OCR文字数组",
      "target": "自然语言描述",
      "conversion": "多语言文字到语义的转换"
    }
  }
}
```