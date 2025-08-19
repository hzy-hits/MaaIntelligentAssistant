//! Function Calling Tools Module
//!
//! MAA智能控制Function Calling工具集，提供16个完整的MAA任务类型。
//! 重构后按功能分类为清晰的模块结构。

pub mod types;
pub mod queue_client;
pub mod core_game;
pub mod advanced_automation;
pub mod support_features;  
pub mod system_features;
pub mod server;
pub mod context;

// 重新导出核心类型
pub use types::{FunctionDefinition, FunctionCall, FunctionResponse, TaskContext, GameState};

// 重新导出服务器
pub use server::{EnhancedMaaFunctionServer, create_enhanced_function_server};

// 重新导出上下文管理
pub use context::{ContextManager, GLOBAL_CONTEXT, record_operation, generate_recommendations, suggest_task_chain, update_game_state, check_reminders};