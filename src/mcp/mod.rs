//! MCP (Model Context Protocol) 集成模块
//!
//! 此模块包含与 MCP 服务器通信的客户端实现。

pub mod client;
pub mod executor;
pub mod protocol;
pub mod registry;
pub mod server;
pub mod transport;

pub use client::McpClient;
pub use executor::{ExecutionResult, ToolExecutor};
pub use protocol::*;
pub use registry::ToolRegistry;
pub use server::{McpServerHandle, McpServerManager, ServerStatus};
pub use transport::{Transport, StdioTransport};
