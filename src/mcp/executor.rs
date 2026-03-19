//! MCP 工具执行器
//!
//! 提供工具调用的执行功能，包含超时控制和错误处理。

use crate::error::{Error, Result};
use crate::mcp::protocol::{CallToolResponse, ToolResultContent};
use crate::mcp::server::McpServerManager;
use serde_json::Value;
use std::time::Duration;
use tracing::{debug, error, info};

// ==================== ToolExecutor ====================

/// 工具执行器
#[derive(Clone)]
pub struct ToolExecutor {
    /// 默认超时时间（毫秒）
    default_timeout: Duration,
}

impl ToolExecutor {
    /// 创建一个新的工具执行器
    pub fn new() -> Self {
        Self {
            default_timeout: Duration::from_secs(30),
        }
    }

    /// 设置默认超时时间
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    /// 执行工具调用
    pub async fn execute(
        &self,
        server_manager: &mut McpServerManager,
        tool_name: &str,
        arguments: Value,
    ) -> Result<ExecutionResult> {
        info!(tool_name = %tool_name, "Executing tool");

        // 查找工具所在的服务器 - 先 clone 出来以避免借用问题
        let server_name = server_manager
            .find_tool_server(tool_name)
            .map(|s| s.to_string())
            .ok_or_else(|| {
                error!(tool_name = %tool_name, "Tool not found");
                Error::McpToolNotFound(tool_name.to_string())
            })?;

        debug!(server_name = %server_name, tool_name = %tool_name, "Found tool server");

        // 执行工具调用（带超时）
        let response = tokio::time::timeout(
            self.default_timeout,
            self.execute_without_timeout(server_manager, &server_name, tool_name, arguments),
        )
        .await
        .map_err(|_| {
            error!(tool_name = %tool_name, "Tool execution timed out");
            Error::McpToolExecution {
                tool: tool_name.to_string(),
                message: "Execution timed out".to_string(),
            }
        })??;

        Ok(response)
    }

    /// 执行工具调用（不带超时）
    async fn execute_without_timeout(
        &self,
        server_manager: &mut McpServerManager,
        server_name: &str,
        tool_name: &str,
        arguments: Value,
    ) -> Result<ExecutionResult> {
        let response = server_manager
            .call_tool(server_name, tool_name, arguments)
            .await?;

        // 转换为执行结果
        let result = ExecutionResult::from_response(response, tool_name.to_string());

        if result.is_error {
            error!(tool_name = %tool_name, "Tool execution returned error");
        } else {
            debug!(tool_name = %tool_name, "Tool execution succeeded");
        }

        Ok(result)
    }
}

impl Default for ToolExecutor {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== ExecutionResult ====================

/// 工具执行结果
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// 工具名称
    pub tool_name: String,
    /// 是否为错误
    pub is_error: bool,
    /// 文本内容（合并所有 text 类型的内容）
    pub text_content: String,
    /// 原始响应内容
    pub raw_content: Vec<ToolResultContent>,
}

impl ExecutionResult {
    /// 从响应创建执行结果
    pub fn from_response(response: CallToolResponse, tool_name: String) -> Self {
        let mut text_content = String::new();

        for content in &response.content {
            if let ToolResultContent::Text { text } = content {
                if !text_content.is_empty() {
                    text_content.push('\n');
                }
                text_content.push_str(text);
            }
        }

        Self {
            tool_name,
            is_error: response.is_error,
            text_content,
            raw_content: response.content,
        }
    }

    /// 获取错误消息（如果是错误）
    pub fn error_message(&self) -> Option<&str> {
        if self.is_error {
            Some(&self.text_content)
        } else {
            None
        }
    }
}
