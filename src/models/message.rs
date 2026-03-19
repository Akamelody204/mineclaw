use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::agent::AgentId;

/// 工具定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// 工具调用
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// 工具结果
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub content: String,
    pub is_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    ToolCall,
    ToolResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub session_id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
    /// 工具调用列表（仅当 role == ToolCall 时有值）
    pub tool_calls: Option<Vec<ToolCall>>,
    /// 工具结果（仅当 role == ToolResult 时有值）
    pub tool_result: Option<ToolResult>,
    /// 关联的 checkpoint ID（可选）
    pub checkpoint_id: Option<String>,
    /// 发送该消息的 Agent ID（可选，None 表示用户/系统消息）
    pub agent_id: Option<AgentId>,
}

impl Message {
    pub fn new(session_id: Uuid, role: MessageRole, content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id,
            role,
            content,
            timestamp: Utc::now(),
            metadata: None,
            tool_calls: None,
            tool_result: None,
            checkpoint_id: None,
            agent_id: None,
        }
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn with_tool_calls(mut self, tool_calls: Vec<ToolCall>) -> Self {
        self.tool_calls = Some(tool_calls);
        self
    }

    pub fn with_tool_result(mut self, tool_result: ToolResult) -> Self {
        self.tool_result = Some(tool_result);
        self
    }

    pub fn with_checkpoint_id(mut self, checkpoint_id: String) -> Self {
        self.checkpoint_id = Some(checkpoint_id);
        self
    }

    pub fn with_agent_id(mut self, agent_id: AgentId) -> Self {
        self.agent_id = Some(agent_id);
        self
    }
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub session_id: Option<Uuid>,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct SendMessageResponse {
    pub message_id: Uuid,
    pub session_id: Uuid,
    pub assistant_response: String,
}

#[derive(Debug, Serialize)]
pub struct ListMessagesResponse {
    pub messages: Vec<Message>,
}

// ==================== 管理 API 数据结构 ====================

/// 工具信息（用于列表 API）
#[derive(Debug, Clone, Serialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub server_name: String,
    pub input_schema: serde_json::Value,
}

/// 工具列表响应
#[derive(Debug, Serialize)]
pub struct ListToolsResponse {
    pub tools: Vec<ToolInfo>,
}

/// MCP 服务器信息
#[derive(Debug, Clone, Serialize)]
pub struct McpServerInfo {
    pub name: String,
    pub status: crate::mcp::ServerStatus,
    pub tool_count: usize,
    pub uptime_seconds: Option<u64>,
    pub last_health_check: Option<DateTime<Utc>>,
}

/// MCP 服务器列表响应
#[derive(Debug, Serialize)]
pub struct ListMcpServersResponse {
    pub servers: Vec<McpServerInfo>,
}

/// 重启 MCP 服务器响应
#[derive(Debug, Serialize)]
pub struct RestartMcpServerResponse {
    pub success: bool,
    pub message: String,
}
