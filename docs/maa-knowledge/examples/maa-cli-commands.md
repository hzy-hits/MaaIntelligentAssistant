# MAA-CLI 命令测试记录

## 基本信息

### 版本信息
```bash
$ maa version
maa-cli v0.5.4
MaaCore v5.22.3
```

### 目录路径

```bash
# 数据目录
$ maa dir data
/Users/ivena/Library/Application Support/com.loong.maa

# 配置目录
$ maa dir config
/Users/ivena/Library/Application Support/com.loong.maa/config

# 缓存目录
$ maa dir cache
/Users/ivena/Library/Caches/com.loong.maa

# 库文件目录
$ maa dir library
/Users/ivena/Library/Application Support/com.loong.maa/lib

# 日志目录
$ maa dir log
/Users/ivena/Library/Application Support/com.loong.maa/debug

# 热更新目录
$ maa dir hot-update
/Users/ivena/Library/Application Support/com.loong.maa/MaaResource

# 资源目录
$ maa dir resource
/Users/ivena/Library/Application Support/com.loong.maa/resource
```

## 可用命令

### 核心命令概览
```bash
$ maa --help

Commands:
  install      Install maa maa_core and resources
  update       Update maa maa_core and resources
  hot-update   Hot update for resource
  dir          Print path of maa directories
  version      Print version of given component
  run          Run a custom task
  startup      Startup Game and Enter Main Screen
  closedown    Close game client
  fight        Run fight task
  copilot      Run copilot task
  ssscopilot   Run SSSCopilot task
  roguelike    Run rouge-like task
  reclamation  Run Reclamation Algorithm task
  convert      Convert file format between TOML, YAML and JSON
  activity     Show stage activity of given client
  remainder    Get the remainder of given divisor and current date
  cleanup      Clearing the caches of maa-cli and maa core
  list         List all available tasks
  import       Import configuration files
  init         Initialize configurations for maa-cli
  complete     Generate completion script for given shell
  mangen       Generate man page
  help         Print this message or the help of the given subcommand(s)
```

### 任务列表
```bash
$ maa list
No tasks found
```

## 游戏控制命令

### startup - 启动游戏
```bash
$ maa startup --help
# 支持的客户端类型：
# Official, Bilibili, Txwy, YoStarEN, YoStarJP, YoStarKR

# 主要参数：
--addr <ADDR>              # ADB设备地址
--profile <PROFILE>        # 配置文件名
--user-resource           # 使用用户资源
--dry-run                 # 仅解析配置不连接游戏
```

### fight - 战斗任务
```bash
$ maa fight --help
# 主要参数：
[STAGE]                   # 关卡名称，如 1-7
-m, --medicine <NUMBER>   # 理智药剂数量
--stone <NUMBER>          # 源石数量
--times <NUMBER>          # 战斗次数
-D, --drops <ITEM_ID=COUNT> # 掉落物品目标
--series <1-6>            # 代理作战次数
--report-to-penguin       # 上报企鹅物流
--dr-grandet              # 葛朗台模式
```

### copilot - 作业任务
```bash
$ maa copilot --help
# 用于执行作业（自动战斗脚本）
```

### roguelike - 肉鸽任务
```bash
$ maa roguelike --help
# 用于运行肉鸽模式
```

## 管理命令

### init - 初始化配置
```bash
$ maa init --help
# 主要参数：
-n, --name <NAME>         # 配置文件名
-f, --format <FORMAT>     # 配置格式: json, yaml, toml
--force                   # 强制覆盖
```

### 安装和更新
```bash
maa install              # 安装 MaaCore 和资源
maa update               # 更新 MaaCore 和资源
maa hot-update           # 热更新资源
```

## 工具命令

### convert - 格式转换
```bash
maa convert              # 在 TOML, YAML, JSON 之间转换
```

### activity - 活动信息
```bash
maa activity             # 显示当前活动信息
```

### cleanup - 清理缓存
```bash
maa cleanup              # 清理缓存文件
```

## 测试结果总结

1. **maa-cli 已正确安装**：版本 v0.5.4，MaaCore v5.22.3
2. **目录结构完整**：所有必需的目录都存在且正确配置
3. **功能覆盖全面**：支持启动、战斗、作业、肉鸽等所有主要功能
4. **配置系统健全**：支持多配置文件和格式转换
5. **暂无可用任务**：`maa list` 显示无任务，可能需要初始化配置

## 重要发现

- maa-cli 提供了完整的 MAA 功能封装
- 支持多种客户端类型（官服、B服、国际服等）
- 具备详细的参数控制和批处理模式
- 集成了企鹅物流等第三方服务
- 支持作业和肉鸽等高级功能