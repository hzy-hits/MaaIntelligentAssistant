//! Legacy Code Module
//!
//! 这个模块包含了旧版本的实现和未使用的代码，保留用于参考和可能的未来重构。
//! 这些代码不会被编译到最终的二进制文件中。

#![allow(dead_code)]
#![allow(unused_imports)]

// Legacy MCP Gateway implementation
pub mod mcp_gateway {
    //! 旧版本的MCP网关实现
    //! 这个实现在架构重构后不再使用，但保留用于参考
    include!("mcp_gateway/mod.rs");
}

// Legacy individual MCP tools
pub mod maa_command {
    //! 旧版本的MAA命令工具实现
    include!("maa_command.rs");
}

pub mod maa_copilot {
    //! 旧版本的MAA作业工具实现  
    include!("maa_copilot.rs");
}

pub mod maa_operators {
    //! 旧版本的MAA干员工具实现
    include!("maa_operators.rs");
}

pub mod maa_status {
    //! 旧版本的MAA状态工具实现
    include!("maa_status.rs");
}

// Legacy RMCP server implementations
pub mod rmcp_server {
    //! 旧版本的RMCP服务器实现
    include!("rmcp_server.rs");
}

pub mod rmcp_tools {
    //! 旧版本的RMCP工具实现
    include!("rmcp_tools.rs");
}