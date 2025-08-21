//! Function Calling Tools Module - V2 优化版本
//!
//! MAA智能控制Function Calling工具集，提供17个完整的MAA任务类型。
//! V2架构：单队列+优先级，简化模块结构。

pub mod types;
pub mod core_game;
pub mod advanced_automation;
pub mod support_features;  
pub mod system_features;
pub mod handler_v2;

// 重新导出核心类型
pub use types::{FunctionDefinition, FunctionCall, FunctionResponse, TaskContext, GameState};

// 重新导出V2优化版Function Calling处理器
pub use handler_v2::{EnhancedMaaFunctionHandlerV2, create_enhanced_function_handler_v2};