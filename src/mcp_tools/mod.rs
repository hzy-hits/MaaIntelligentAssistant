//! Function Calling Tools Module
//!
//! This module implements Function Calling tools for MAA operations.
//! Provides a simple and direct interface for AI models to control MAA.

mod function_calling;
mod enhanced_tools;
mod maa_startup;
mod maa_combat;
mod maa_recruit;
mod maa_infrastructure;
mod maa_roguelike;
mod maa_copilot_enhanced;
mod maa_other_tools;
mod maa_atomic_operations;
mod atomic_function_server;
mod maa_custom_tasks;

#[cfg(test)]
mod integration_tests;

// 导出基础 Function Calling 实现
pub use function_calling::{
    MaaFunctionServer, create_function_server,
    FunctionDefinition, FunctionCall, FunctionResponse
};

// 导出增强 Function Calling 实现 (16个MAA任务)
pub use enhanced_tools::{
    EnhancedMaaFunctionServer, create_enhanced_function_server
};

// 导出原子级 Function Calling 实现 (11个底层操作)
pub use atomic_function_server::{
    AtomicMaaFunctionServer, create_atomic_function_server
};