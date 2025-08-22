# MAA源码研究计划

## 研究目标
深度分析MAA官方源码，提取智能决策机制，为Python决策层设计提供技术基础。

## 并行研究架构
5个独立的Sub Agent同时进行专项研究：

### Agent 1: 任务系统分析
- **Session**: maa-task-system
- **研究重点**: resource/tasks/*.json任务定义、状态机转换逻辑
- **输出**: TASK_SYSTEM.md
- **状态**: ✅ 已完成 (952行分析文档)

### Agent 2: 图像识别系统  
- **Session**: maa-image-rec
- **研究重点**: 模板匹配算法、OCR识别、界面状态判定
- **输出**: IMAGE_RECOGNITION.md
- **状态**: 🔄 进行中

### Agent 3: 基建智能调度
- **Session**: maa-infrast  
- **研究重点**: custom_infrast/排班配置、干员效率计算
- **输出**: INFRAST_SCHEDULING.md
- **状态**: ⏳ 待开始

### Agent 4: 战斗决策系统
- **Session**: maa-battle
- **研究重点**: copilot/作业系统、操作序列、技能释放时机
- **输出**: BATTLE_STRATEGY.md  
- **状态**: ⏳ 待开始

### Agent 5: FFI接口设计
- **Session**: maa-ffi
- **研究重点**: AsstCaller.h接口、回调机制、Python桥接
- **输出**: FFI_INTEGRATION.md
- **状态**: ⏳ 待开始

## 研究环境
- **独立目录**: ~/maa-research/maa-official-study (不污染Git)
- **TMux管理**: 6个session (main + 5个研究)
- **文档输出**: docs/maa-research/

## 预期成果
1. **技术架构理解** - MAA如何实现智能决策
2. **设计模式提取** - 可借鉴的架构思想
3. **Python实现方案** - 具体的集成设计
4. **完整技术文档** - 5份专项分析报告

## 研究时间线
- **Week 1**: 5个Agent并行深度研究
- **Week 2**: 成果整合与Python架构设计  
- **Week 3**: PyO3集成方案实施
- **Week 4**: 测试验证与文档完善