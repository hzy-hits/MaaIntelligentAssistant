# 并行Sub Agent管理

## TMux Session架构
```
maa-main          # 主控制台 - 监控所有session
├── maa-task-system   # Agent 1: 任务系统研究
├── maa-image-rec     # Agent 2: 图像识别研究  
├── maa-infrast       # Agent 3: 基建调度研究
├── maa-battle        # Agent 4: 战斗策略研究
└── maa-ffi           # Agent 5: FFI接口研究
```

## Agent协调机制
- **独立研究** - 每个Agent专注单一模块，避免冲突
- **统一输出** - 所有分析文档存储在docs/maa-research/
- **进度跟踪** - 主session监控各Agent状态
- **成果同步** - 定期更新到CLAUDE.md记忆系统

## Session切换命令
```bash
# 查看所有session
tmux list-sessions

# 切换到指定研究session
tmux attach -t maa-task-system
tmux attach -t maa-image-rec  
tmux attach -t maa-infrast
tmux attach -t maa-battle
tmux attach -t maa-ffi

# 回到主控制台
tmux attach -t maa-main

# 终止所有研究session
tmux kill-session -t maa-task-system
tmux kill-session -t maa-image-rec
tmux kill-session -t maa-infrast  
tmux kill-session -t maa-battle
tmux kill-session -t maa-ffi
```

## 研究任务分配
每个Agent负责：
1. **深度代码分析** - 理解模块实现原理
2. **架构模式提取** - 识别可借鉴的设计思想  
3. **技术文档编写** - 输出详细分析报告
4. **Python方案建议** - 提供具体集成建议

## 协作最佳实践
- **避免交叉** - 各Agent研究范围明确分工
- **定期同步** - 重要发现及时更新到共享记忆
- **成果整合** - 最终合并为统一的Python决策层设计
- **质量保证** - 每份分析文档需包含具体实现建议