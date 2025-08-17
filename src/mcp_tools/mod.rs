//! Function Calling Tools Module
//!
//! This module implements Function Calling tools for MAA operations.
//! Provides a simple and direct interface for AI models to control MAA.

mod function_calling;

#[cfg(test)]
mod integration_tests;

// 导出 Function Calling 实现
pub use function_calling::{
    MaaFunctionServer, create_function_server,
    FunctionDefinition, FunctionCall, FunctionResponse
};