# MAA基建智能调度系统深度分析

## 概述

MAA (MaaAssistantArknights) 基建调度系统是一个高度精细化的干员管理与效率优化系统，通过复杂的算法实现基建设施的智能化运营。本文档基于MAA官方代码库的深入研究，详细分析其核心调度算法、效率计算机制以及智能决策逻辑。

## 1. 基建布局系统

### 1.1 布局策略分类

MAA支持多种基建布局策略，每种都有不同的优化目标：

#### 243布局（极限效率）
```json
{
    "title": "243极限效率，一天三换",
    "buildingType": "243", 
    "planTimes": "3班",
    "plans": [
        {
            "name": "16H最高效率长班",
            "description": "请手动把年放加工站，下次在16小时后换班"
        },
        {
            "name": "4H次高效替补2班", 
            "description": "下次在4小时后换班"
        }
    ]
}
```

**特点**：
- 2个贸易站 + 4个制造站 + 3个发电站
- 最大化生产效率，要求频繁换班
- 适合高强度管理的玩家

#### 153布局（平衡发展）
```json
{
    "buildingType": "153",
    "scheduleType": {
        "planTimes": 4,
        "trading": 1,
        "manufacture": 5, 
        "power": 3,
        "dormitory": 4
    }
}
```

**特点**：
- 1个贸易站 + 5个制造站 + 3个发电站
- 制造效率极大化，龙门币获取相对较少
- 适合专注某类资源生产

#### 333布局（合成玉特化）
```json
{
    "title": "333_layout_for_Orundum_3_times_a_day",
    "buildingType": "333"
}
```

**特点**：
- 3个贸易站 + 3个制造站 + 3个发电站
- 专门用于合成玉生产
- 平衡资源获取和管理难度

### 1.2 房间类型系统

```cpp
enum class InfrastRoomType 
{
    Control,      // 控制中枢 - 心情恢复中心
    Manufacture,  // 制造站 - 生产各类物品
    Trading,      // 贸易站 - 处理订单获取龙门币
    Power,        // 发电站 - 提供电力支持
    Reception,    // 会客室 - 线索收集与交换
    Office,       // 办公室 - 线索处理
    Dormitory,    // 宿舍 - 干员休息恢复
    Processing,   // 加工站 - 特殊材料处理
    Training      // 训练室 - 技能训练
};
```

### 1.3 配置文件结构

```json
{
    "facility": {
        "maxNumOfOpers": 3,
        "products": ["Battle Record", "Pure Gold", "Dualchip"],
        "skills": {
            "skill_id": {
                "desc": ["技能描述"],
                "efficient": {"Battle Record": 0.35, "Pure Gold": 0.25},
                "efficient_regex": {"product_reg": "regex_pattern"},
                "name": ["技能名称"],
                "template": "skill_template.png",
                "maxNum": 2
            }
        },
        "skillsGroup": [
            {
                "desc": "技能组描述",
                "conditions": {"power_plant_count": 3},
                "necessary": [/* 必选技能组合 */],
                "optional": [/* 可选技能组合 */],
                "allow_external": false
            }
        ]
    }
}
```

## 2. 干员效率计算系统

### 2.1 技能效率核心算法

```cpp
struct Skill {
    std::string id;
    std::vector<std::string> names;
    std::string desc;
    std::unordered_map<std::string, double> efficient;        // 直接效率值
    std::unordered_map<std::string, std::string> efficient_regex; // 动态效率正则
    int max_num = INT_MAX;  // 技能数量限制
};

struct SkillsComb {
    std::unordered_set<Skill> skills;
    std::unordered_map<std::string, double> efficient;
    std::unordered_map<std::string, std::string> efficient_regex;
    
    // 构造函数中的效率累加逻辑
    SkillsComb(std::unordered_set<Skill> skill_vec) {
        skills = std::move(skill_vec);
        for (const auto& s : skills) {
            // 直接效率累加
            for (const auto& [key, value] : s.efficient) {
                efficient[key] += value;
            }
            // 正则表达式效率累加
            for (const auto& [key, reg] : s.efficient_regex) {
                efficient_regex[key] += "+(" + reg + ")";
            }
        }
    }
};
```

### 2.2 效率计算优先级

1. **正则表达式效率**：动态计算，支持复杂的条件判断
2. **固定数值效率**：直接数值加成
3. **技能组合效率**：多技能协同效应
4. **设施条件加成**：基于设施配置的额外加成

### 2.3 心情系统建模

```cpp
enum class SmileyType {
    Invalid = -1,
    Rest,        // 休息完成，绿色笑脸（高效率）
    Work,        // 工作中，黄色笑脸（正常效率）
    Distract     // 注意力涣散，红色哭脸（低效率）
};

struct Oper {
    Smiley smiley;
    double mood_ratio = 0;  // 心情百分比 [0,1]
    Doing doing = Doing::Invalid;
    bool selected = false;
    std::unordered_set<Skill> skills;
};
```

**心情影响机制**：
- 心情值直接影响工作效率
- 心情阈值(`mood_threshold`)决定是否需要轮换
- 宿舍分配算法优化心情恢复速度

### 2.4 协同效应计算

```cpp
struct SkillsGroup {
    std::string desc;
    std::unordered_map<std::string, int> conditions;  // 激活条件
    std::vector<SkillsComb> necessary;                // 必选技能
    std::vector<SkillsComb> optional;                 // 可选技能
    bool allow_external = false;                      // 允许外部干员补充
};
```

**协同效应机制**：
1. **必选技能检查**：确保核心技能存在
2. **可选技能匹配**：最大化可选技能覆盖
3. **条件验证**：检查设施条件（如发电站数量）
4. **效率累积**：计算总体效率提升

## 3. 智能调度策略

### 3.1 调度模式架构

```cpp
class InfrastTask final : public InterfaceTask {
    enum class Mode {
        Default = 0,     // 默认模式
        Custom = 10000,  // 自定义配置模式
        Rotation = 20000 // 轮换模式
    };
    
private:
    std::shared_ptr<InfrastMfgTask> m_mfg_task_ptr;
    std::shared_ptr<InfrastTradeTask> m_trade_task_ptr;
    std::shared_ptr<InfrastDormTask> m_dorm_task_ptr;
    // ... 其他设施任务指针
};
```

### 3.2 自动排班决策逻辑

#### 3.2.1 干员选择算法

```cpp
bool select_custom_opers(std::vector<std::string>& partial_result) {
    // 1. 心情筛选
    if (mood_ratio < m_mood_threshold) return false;
    
    // 2. 技能匹配
    for (const auto& required_skill : required_skills) {
        if (!oper.skills.contains(required_skill)) return false;
    }
    
    // 3. 优先级排序
    std::sort(candidates.begin(), candidates.end(), 
              [](const Oper& a, const Oper& b) {
                  return calculate_efficiency(a) > calculate_efficiency(b);
              });
    
    return true;
}
```

#### 3.2.2 时间轮换策略

```json
{
    "period": [
        ["22:00", "23:59"],  // 时间段1
        ["00:00", "06:00"]   // 时间段2（跨天处理）
    ],
    "duration": 360,         // 工作持续时长（分钟）
    "order": "pre"           // 执行顺序：pre/post
}
```

**轮换触发条件**：
1. **时间触发**：达到预设时间段
2. **心情触发**：平均心情低于阈值
3. **效率触发**：效率下降超过阈值
4. **手动触发**：用户主动请求

### 3.3 优先级调度算法

```cpp
struct CustomRoomConfig {
    enum class Product {
        BattleRecord,    // 经验书
        PureGold,        // 赤金  
        Dualchip,        // 双芯片
        OriginiumShard,  // 源石碎片
        LMD,             // 龙门币
        Orundum          // 合成玉
    };
    
    std::vector<std::string> names;      // 指定干员
    std::vector<std::string> candidates; // 候选干员
    bool autofill = false;               // 自动填充
    Product product;                     // 目标产品
    bool sort = false;                   // 是否排序
};
```

**优先级计算公式**：
```
priority = base_efficiency × mood_factor × skill_synergy × facility_bonus
```

## 4. 无人机管理系统

### 4.1 无人机分配策略

```cpp
struct CustomDronesConfig {
    enum class Order { Pre, Post };
    
    int index = 0;           // 目标设施索引
    Order order = Order::Pre; // 执行时机
};
```

**分配规则**：
```json
{
    "drones": {
        "enable": true,
        "room": "manufacture",  // trading/manufacture
        "index": 1,            // 设施编号 [1,5]
        "rule": "all",         // 使用规则（预留）
        "order": "pre"         // pre: 换班前 / post: 换班后
    }
}
```

### 4.2 订单优先级系统

1. **贸易站订单**：
   - 高价值订单优先
   - 接近完成的订单优先
   - 稀有材料订单优先

2. **制造站生产**：
   - 当前急需材料优先
   - 高效率干员配置优先
   - 生产周期短的物品优先

### 4.3 资源分配平衡算法

```cpp
void optimize_drone_allocation() {
    // 1. 计算各设施收益率
    for (auto& facility : facilities) {
        facility.roi = calculate_return_on_investment(facility);
    }
    
    // 2. 动态调整分配策略
    std::priority_queue<Facility> facility_queue;
    for (auto& facility : facilities) {
        facility_queue.push(facility);
    }
    
    // 3. 按收益率分配无人机
    while (!drones.empty() && !facility_queue.empty()) {
        auto best_facility = facility_queue.top();
        assign_drone(drones.front(), best_facility);
        // 更新状态并重新排序
    }
}
```

## 5. 心情和疲劳管理

### 5.1 心情恢复机制

```cpp
class InfrastDormTask {
private:
    double m_mood_threshold = 0.3;  // 心情阈值
    bool m_notstationed_enabled = false;    // 未进驻干员
    bool m_trust_enabled = false;           // 信赖值优先
    
public:
    bool assign_operators_to_dormitory() {
        // 1. 筛选低心情干员
        auto low_mood_opers = filter_by_mood(all_operators, m_mood_threshold);
        
        // 2. 优先级排序
        std::sort(low_mood_opers.begin(), low_mood_opers.end(),
                  [](const Oper& a, const Oper& b) {
                      if (a.mood_ratio != b.mood_ratio) 
                          return a.mood_ratio < b.mood_ratio;
                      return a.trust_value < b.trust_value;  // 信赖值考虑
                  });
        
        // 3. 分配到最优宿舍
        for (auto& oper : low_mood_opers) {
            auto best_dorm = find_best_dormitory(oper);
            assign_to_dormitory(oper, best_dorm);
        }
    }
};
```

### 5.2 疲劳度建模

```cpp
enum class Doing {
    Invalid = -1,
    Nothing,    // 空闲状态
    Resting,    // 休息中（宿舍）
    Working     // 工作中（生产设施）
};

struct FatigueModel {
    double base_consumption = 1.0;      // 基础心情消耗
    double work_multiplier = 1.5;       // 工作状态乘数
    double skill_modifier = 0.0;        // 技能修正
    double facility_bonus = 0.0;        // 设施加成
    
    double calculate_hourly_mood_change(const Oper& oper) {
        if (oper.doing == Doing::Resting) {
            return base_recovery_rate + facility_bonus;
        } else if (oper.doing == Doing::Working) {
            return -(base_consumption * work_multiplier - skill_modifier);
        }
        return 0.0;
    }
};
```

### 5.3 宿舍分配优化

**宿舍效果加成系统**：
```json
{
    "dormitory": [
        {
            "operators": ["爱丽丝"],     // 指定干员（心情恢复加成）
            "sort": true,               // 按顺序排列
            "autofill": true           // 自动填充其余位置
        }
    ]
}
```

**分配策略**：
1. **心情优先**：最低心情干员优先进入宿舍
2. **技能协同**：宿舍技能干员组合优化
3. **信赖培养**：低信赖干员获得额外休息时间
4. **未进驻管理**：未进驻干员的心情维护

## 6. Python集成建议

### 6.1 智能调度引擎架构

```python
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass
from enum import Enum

class FacilityType(Enum):
    CONTROL = "control"
    MANUFACTURE = "manufacture"
    TRADING = "trading"
    POWER = "power"
    DORMITORY = "dormitory"
    RECEPTION = "reception"
    OFFICE = "office"
    PROCESSING = "processing"

@dataclass
class OperatorSkill:
    id: str
    name: str
    description: str
    efficiency: Dict[str, float]  # product -> efficiency value
    efficiency_regex: Dict[str, str]  # dynamic efficiency patterns
    max_count: int = float('inf')

@dataclass 
class Operator:
    name: str
    face_hash: str  # 区分同技能不同干员
    mood_ratio: float  # [0, 1]
    skills: List[OperatorSkill]
    trust_level: int
    is_working: bool = False
    facility: Optional[str] = None
    
    def calculate_efficiency(self, product: str, facility_type: FacilityType) -> float:
        """计算该干员在特定设施生产特定产品的效率"""
        total_efficiency = 0.0
        for skill in self.skills:
            if product in skill.efficiency:
                total_efficiency += skill.efficiency[product]
        return total_efficiency * self.mood_ratio

class InfrastScheduler:
    def __init__(self):
        self.operators: List[Operator] = []
        self.facilities: Dict[str, Dict] = {}
        self.mood_threshold: float = 0.3
        self.scheduling_rules: Dict = {}
    
    def load_infrast_config(self, config_path: str):
        """加载基建配置文件"""
        with open(config_path, 'r', encoding='utf-8') as f:
            config = json.load(f)
        
        for facility_type, facility_config in config.items():
            self.facilities[facility_type] = {
                'max_operators': facility_config.get('maxNumOfOpers', 3),
                'products': facility_config.get('products', []),
                'skills': self._parse_skills(facility_config.get('skills', {})),
                'skill_groups': self._parse_skill_groups(facility_config.get('skillsGroup', []))
            }
    
    def optimize_operator_assignment(self, facility_type: FacilityType, 
                                   target_product: str) -> List[Operator]:
        """优化干员分配"""
        available_operators = self._get_available_operators()
        
        # 1. 筛选适合的干员
        suitable_operators = []
        for op in available_operators:
            if op.mood_ratio >= self.mood_threshold:
                efficiency = op.calculate_efficiency(target_product, facility_type)
                if efficiency > 0:
                    suitable_operators.append((op, efficiency))
        
        # 2. 按效率排序
        suitable_operators.sort(key=lambda x: x[1], reverse=True)
        
        # 3. 技能组合优化
        optimal_team = self._find_optimal_skill_combination(
            suitable_operators, facility_type, target_product
        )
        
        return optimal_team
    
    def _find_optimal_skill_combination(self, candidates: List[Tuple[Operator, float]],
                                      facility_type: FacilityType, 
                                      target_product: str) -> List[Operator]:
        """寻找最优技能组合"""
        max_operators = self.facilities[facility_type.value]['max_operators']
        skill_groups = self.facilities[facility_type.value]['skill_groups']
        
        best_combination = []
        best_efficiency = 0.0
        
        # 使用动态规划或贪心算法寻找最优组合
        # 考虑技能协同效应和约束条件
        for group in skill_groups:
            combination = self._evaluate_skill_group(candidates, group, max_operators)
            efficiency = self._calculate_group_efficiency(combination, target_product)
            
            if efficiency > best_efficiency:
                best_efficiency = efficiency
                best_combination = combination
        
        return best_combination[:max_operators]
    
    def schedule_shift_rotation(self, custom_config: Dict) -> Dict[str, List[str]]:
        """执行换班调度"""
        schedule_result = {}
        
        for facility_name, room_configs in custom_config.get('rooms', {}).items():
            facility_schedule = []
            
            for room_config in room_configs:
                if room_config.get('skip', False):
                    continue
                
                # 自定义干员优先
                if 'operators' in room_config and room_config['operators']:
                    facility_schedule.extend(room_config['operators'])
                
                # 自动填充
                elif room_config.get('autofill', False):
                    auto_operators = self._auto_fill_operators(
                        facility_name, 
                        room_config.get('product', ''),
                        len(room_config.get('operators', []))
                    )
                    facility_schedule.extend(auto_operators)
            
            schedule_result[facility_name] = facility_schedule
        
        return schedule_result
    
    def manage_mood_and_rest(self) -> Dict[str, List[str]]:
        """心情和休息管理"""
        # 1. 筛选需要休息的干员
        tired_operators = [op for op in self.operators 
                          if op.mood_ratio < self.mood_threshold and op.is_working]
        
        # 2. 优化宿舍分配
        dormitory_assignment = self._optimize_dormitory_assignment(tired_operators)
        
        # 3. 计算替换方案
        replacement_schedule = self._calculate_replacement_schedule(tired_operators)
        
        return {
            'dormitory_assignment': dormitory_assignment,
            'replacement_schedule': replacement_schedule
        }
    
    def _optimize_dormitory_assignment(self, tired_operators: List[Operator]) -> Dict[str, List[str]]:
        """优化宿舍分配算法"""
        dormitory_config = self.facilities.get('dormitory', {})
        max_dorm_capacity = dormitory_config.get('max_operators', 5)
        
        # 按心情值排序，最疲劳的优先休息
        tired_operators.sort(key=lambda op: op.mood_ratio)
        
        assignment = {}
        dorm_index = 0
        
        for op in tired_operators:
            dorm_name = f'dormitory_{dorm_index}'
            if dorm_name not in assignment:
                assignment[dorm_name] = []
            
            if len(assignment[dorm_name]) < max_dorm_capacity:
                assignment[dorm_name].append(op.name)
            else:
                dorm_index += 1
                dorm_name = f'dormitory_{dorm_index}'
                assignment[dorm_name] = [op.name]
        
        return assignment
```

### 6.2 效率监控和分析模块

```python
class EfficiencyAnalyzer:
    def __init__(self, scheduler: InfrastScheduler):
        self.scheduler = scheduler
        self.efficiency_history: List[Dict] = []
    
    def analyze_current_efficiency(self) -> Dict:
        """分析当前基建效率"""
        facility_efficiency = {}
        
        for facility_type, facility_config in self.scheduler.facilities.items():
            working_operators = self._get_working_operators(facility_type)
            
            if not working_operators:
                facility_efficiency[facility_type] = {
                    'total_efficiency': 0.0,
                    'operator_count': 0,
                    'average_mood': 0.0
                }
                continue
            
            total_eff = sum(op.calculate_efficiency('default', 
                                                 FacilityType(facility_type)) 
                           for op in working_operators)
            avg_mood = sum(op.mood_ratio for op in working_operators) / len(working_operators)
            
            facility_efficiency[facility_type] = {
                'total_efficiency': total_eff,
                'operator_count': len(working_operators),
                'average_mood': avg_mood,
                'efficiency_per_operator': total_eff / len(working_operators)
            }
        
        return facility_efficiency
    
    def predict_optimal_schedule(self, time_horizon: int = 24) -> Dict:
        """预测最优调度方案"""
        # 基于历史数据和当前状态预测
        current_state = self.analyze_current_efficiency()
        mood_decay_model = self._build_mood_decay_model()
        
        # 模拟未来24小时的心情变化
        predicted_schedule = {}
        for hour in range(time_horizon):
            hour_schedule = self._simulate_hour_schedule(hour, mood_decay_model)
            predicted_schedule[f'hour_{hour}'] = hour_schedule
        
        return predicted_schedule
    
    def generate_optimization_report(self) -> str:
        """生成优化建议报告"""
        analysis = self.analyze_current_efficiency()
        
        report = "# 基建效率分析报告\n\n"
        
        for facility, metrics in analysis.items():
            report += f"## {facility.upper()}设施\n"
            report += f"- 总效率: {metrics['total_efficiency']:.2f}\n"
            report += f"- 干员数量: {metrics['operator_count']}\n"
            report += f"- 平均心情: {metrics['average_mood']:.2f}\n"
            report += f"- 人均效率: {metrics['efficiency_per_operator']:.2f}\n\n"
            
            # 优化建议
            if metrics['average_mood'] < 0.5:
                report += "**建议**: 心情偏低，建议安排休息\n"
            
            if metrics['efficiency_per_operator'] < 0.3:
                report += "**建议**: 效率偏低，建议调整干员配置\n"
            
            report += "\n"
        
        return report
```

### 6.3 与Rust架构的集成接口

```python
import json
from typing import Any
import asyncio
import aiohttp

class MAAIntegrationClient:
    def __init__(self, maa_server_url: str = "http://localhost:8080"):
        self.server_url = maa_server_url
        self.scheduler = InfrastScheduler()
    
    async def execute_infrast_task(self, config: Dict[str, Any]) -> Dict:
        """执行基建任务"""
        function_call = {
            "name": "maa_infrastructure_enhanced",
            "arguments": {
                "enable": True,
                "mode": config.get("mode", 1),  # 0=默认, 10000=自定义
                "facility": config.get("facilities", ["Mfg", "Trade", "Power"]),
                "drones": config.get("drones", "_NotUse"),
                "threshold": config.get("mood_threshold", 0.3),
                "replenish": config.get("replenish", False),
                "filename": config.get("custom_config_path", ""),
                "plan_index": config.get("plan_index", 0)
            }
        }
        
        async with aiohttp.ClientSession() as session:
            async with session.post(
                f"{self.server_url}/call",
                json={"function_call": function_call}
            ) as response:
                return await response.json()
    
    async def get_current_status(self) -> Dict:
        """获取当前MAA状态"""
        async with aiohttp.ClientSession() as session:
            async with session.get(f"{self.server_url}/status") as response:
                return await response.json()
    
    def optimize_and_execute(self, layout_type: str = "243") -> Dict:
        """优化并执行基建调度"""
        # 1. 加载对应布局配置
        config_path = f"custom_infrast/{layout_type}_layout_3_times_a_day.json"
        with open(config_path, 'r', encoding='utf-8') as f:
            layout_config = json.load(f)
        
        # 2. Python层优化
        optimized_config = self.scheduler.optimize_layout_config(layout_config)
        
        # 3. 执行MAA调用
        result = asyncio.run(self.execute_infrast_task({
            "mode": 10000,  # Custom mode
            "custom_config_path": config_path,
            "plan_index": 0,
            "mood_threshold": 0.3
        }))
        
        return {
            "optimization_result": optimized_config,
            "execution_result": result
        }

# 使用示例
if __name__ == "__main__":
    # 初始化调度器
    scheduler = InfrastScheduler()
    scheduler.load_infrast_config("infrast.json")
    
    # 创建集成客户端
    client = MAAIntegrationClient()
    
    # 执行优化调度
    result = client.optimize_and_execute("243")
    print("调度结果:", result)
    
    # 生成分析报告
    analyzer = EfficiencyAnalyzer(scheduler)
    report = analyzer.generate_optimization_report()
    print(report)
```

## 7. 核心算法总结

### 7.1 调度决策流程

```
1. 配置加载 → 解析JSON配置文件，构建设施和技能数据
2. 状态分析 → 检测当前干员心情、工作状态、设施状态
3. 需求评估 → 根据目标产品和时间要求制定调度需求
4. 干员筛选 → 基于心情阈值和技能匹配筛选可用干员
5. 组合优化 → 使用贪心/动态规划寻找最优技能组合
6. 执行分配 → 将干员分配到对应设施并启动生产
7. 监控反馈 → 持续监控效率和心情变化，触发重新调度
```

### 7.2 效率最大化策略

1. **技能协同最大化**：优先选择具有协同效应的技能组合
2. **心情管理优化**：平衡工作效率和心情消耗，避免频繁换班
3. **时间窗口利用**：根据不同时间段的需求调整生产重点
4. **资源分配均衡**：确保各类资源生产的合理分配

### 7.3 性能优化要点

- **缓存机制**：缓存效率计算结果，避免重复计算
- **增量更新**：仅在状态变化时重新计算受影响的部分
- **并行处理**：多设施调度任务可并行执行
- **预测优化**：基于历史数据预测最优调度时机

## 结论

MAA的基建调度系统展现了复杂系统设计的精妙之处，通过多层次的算法设计实现了高效的资源管理和智能决策。其核心价值在于：

1. **数据驱动**：基于详细的技能和效率数据进行精确计算
2. **智能优化**：结合多种算法寻找最优解决方案  
3. **灵活配置**：支持高度自定义的调度策略
4. **实时响应**：能够根据状态变化动态调整策略

Python集成方案为该系统提供了更好的扩展性和分析能力，可以在MAA强大的底层能力基础上构建更智能的决策层，实现真正的"智能基建管理系统"。

---

*本分析基于MAA官方代码库v5.x版本，涵盖了基建调度系统的核心算法和实现细节。*