use anyhow::Result;
use serde_json::Value;
use tokio::sync::oneshot;

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

/// MAA任务队列发送器类型
pub type MaaTaskSender = tokio::sync::mpsc::UnboundedSender<MaaTask>;

/// MAA任务队列接收器类型  
pub type MaaTaskReceiver = tokio::sync::mpsc::UnboundedReceiver<MaaTask>;

/// 创建MAA任务队列通道
pub fn create_maa_task_channel() -> (MaaTaskSender, MaaTaskReceiver) {
    tokio::sync::mpsc::unbounded_channel()
}