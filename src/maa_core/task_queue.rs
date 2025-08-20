use anyhow::Result;
use serde_json::Value;
use tokio::sync::oneshot;

/// 任务优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    /// 正常优先级：游戏操作任务（战斗、招募等）
    Normal = 1,
    /// 高优先级：即时操作（截图、点击、状态查询）
    High = 10,
}

/// 带优先级的任务包装器
#[derive(Debug)]
pub struct PriorityTask {
    pub priority: TaskPriority,
    pub task: MaaTask,
}

impl PriorityTask {
    pub fn new(task: MaaTask, priority: TaskPriority) -> Self {
        Self { priority, task }
    }
    
    /// 创建高优先级任务（截图、点击等即时操作）
    pub fn high_priority(task: MaaTask) -> Self {
        Self::new(task, TaskPriority::High)
    }
    
    /// 创建普通优先级任务（游戏操作等）
    pub fn normal_priority(task: MaaTask) -> Self {
        Self::new(task, TaskPriority::Normal)
    }
}

impl Ord for PriorityTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // 高优先级任务排在前面（数字越大优先级越高）
        other.priority.cmp(&self.priority)
    }
}

impl PartialOrd for PriorityTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PriorityTask {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for PriorityTask {}

/// MAA任务队列消息定义
/// 
/// 所有MAA操作都通过消息队列发送到专用的MAA线程执行
/// 这确保MAA Core实例的线程安全性和状态一致性
#[derive(Debug)]
pub enum MaaTask {
    /// 游戏启动任务
    Startup {
        client_type: String,
        start_app: bool,
        close_app: bool,
        response_tx: oneshot::Sender<Result<Value>>,
    },
    
    /// 设备连接任务
    Connect {
        address: String,
        response_tx: oneshot::Sender<Result<i32>>,
    },
    
    /// 战斗刷图任务
    Combat {
        stage: String,
        medicine: i32,
        stone: i32,
        times: i32,
        response_tx: oneshot::Sender<Result<Value>>,
    },
    
    /// 公开招募任务
    Recruit {
        max_times: i32,
        expedite: bool,
        skip_robot: bool,
        response_tx: oneshot::Sender<Result<Value>>,
    },
    
    /// 基建任务
    Infrastructure {
        facility: Vec<String>,
        drones: String,
        threshold: f64,
        response_tx: oneshot::Sender<Result<Value>>,
    },
    
    /// 肉鸽任务
    Roguelike {
        theme: String,
        mode: i32,
        starts_count: i32,
        response_tx: oneshot::Sender<Result<Value>>,
    },
    
    /// 作业执行任务
    Copilot {
        filename: String,
        formation: bool,
        response_tx: oneshot::Sender<Result<Value>>,
    },
    
    /// 保全派驻任务
    SssCopilot {
        filename: String,
        loop_times: i32,
        response_tx: oneshot::Sender<Result<Value>>,
    },
    
    /// 生息演算任务
    Reclamation {
        theme: String,
        mode: i32,
        response_tx: oneshot::Sender<Result<Value>>,
    },
    
    /// 领取奖励任务
    Rewards {
        award: bool,
        mail: bool,
        recruit: bool,
        orundum: bool,
        response_tx: oneshot::Sender<Result<Value>>,
    },
    
    /// 信用商店任务
    CreditStore {
        credit_fight: bool,
        response_tx: oneshot::Sender<Result<Value>>,
    },
    
    /// 仓库管理任务
    DepotManagement {
        enable: bool,
        response_tx: oneshot::Sender<Result<Value>>,
    },
    
    /// 干员管理任务
    OperatorBox {
        enable: bool,
        response_tx: oneshot::Sender<Result<Value>>,
    },
    
    /// 游戏关闭任务
    Closedown {
        response_tx: oneshot::Sender<Result<Value>>,
    },
    
    /// 自定义任务
    CustomTask {
        task_type: String,
        params: String,
        response_tx: oneshot::Sender<Result<Value>>,
    },
    
    /// 视频识别任务
    VideoRecognition {
        video_path: String,
        response_tx: oneshot::Sender<Result<Value>>,
    },
    
    /// 系统管理任务
    SystemManagement {
        action: String,
        response_tx: oneshot::Sender<Result<Value>>,
    },
    
    /// 获取MAA状态
    GetStatus {
        response_tx: oneshot::Sender<Result<Value>>,
    },
    
    /// 截图操作
    TakeScreenshot {
        response_tx: oneshot::Sender<Result<Vec<u8>>>,
    },
    
    /// 点击操作
    PerformClick {
        x: i32,
        y: i32,
        response_tx: oneshot::Sender<Result<i32>>,
    },
    
    /// 停止所有任务
    StopAllTasks {
        response_tx: oneshot::Sender<Result<()>>,
    },
}

/// 双优先级MAA任务队列发送器
#[derive(Clone)]
pub struct MaaTaskSender {
    pub high_priority: tokio::sync::mpsc::UnboundedSender<MaaTask>,
    pub normal_priority: tokio::sync::mpsc::UnboundedSender<MaaTask>,
}

/// 双优先级MAA任务队列接收器
pub struct MaaTaskReceiver {
    pub high_priority: tokio::sync::mpsc::UnboundedReceiver<MaaTask>,
    pub normal_priority: tokio::sync::mpsc::UnboundedReceiver<MaaTask>,
}

impl MaaTaskSender {
    /// 发送高优先级任务（截图、点击等即时操作）
    pub fn send_high_priority(&self, task: MaaTask) -> Result<(), tokio::sync::mpsc::error::SendError<MaaTask>> {
        self.high_priority.send(task)
    }
    
    /// 发送普通优先级任务（游戏操作等）
    pub fn send_normal_priority(&self, task: MaaTask) -> Result<(), tokio::sync::mpsc::error::SendError<MaaTask>> {
        self.normal_priority.send(task)
    }
    
    /// 根据任务类型自动选择优先级发送
    pub fn send_auto(&self, task: MaaTask) -> Result<(), tokio::sync::mpsc::error::SendError<MaaTask>> {
        match &task {
            // 高优先级：即时操作
            MaaTask::TakeScreenshot { .. } | 
            MaaTask::PerformClick { .. } | 
            MaaTask::GetStatus { .. } => {
                self.send_high_priority(task)
            },
            // 普通优先级：游戏操作
            _ => {
                self.send_normal_priority(task)
            }
        }
    }
}

/// 创建双优先级MAA任务队列通道
pub fn create_maa_task_channel() -> (MaaTaskSender, MaaTaskReceiver) {
    let (high_tx, high_rx) = tokio::sync::mpsc::unbounded_channel();
    let (normal_tx, normal_rx) = tokio::sync::mpsc::unbounded_channel();
    
    let sender = MaaTaskSender {
        high_priority: high_tx,
        normal_priority: normal_tx,
    };
    
    let receiver = MaaTaskReceiver {
        high_priority: high_rx,
        normal_priority: normal_rx,
    };
    
    (sender, receiver)
}