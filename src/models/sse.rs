//! SSE 事件定义
//!
//! 用于 Server-Sent Events 流式响应的事件类型。

use serde::Serialize;
use serde_json::Value;

/// SSE 事件类型
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SseEvent {
    /// 会话开始
    SessionStarted { session_id: String },
    /// 助手消息
    AssistantMessage {
        agent_id: Option<String>,
        content: String,
    },
    /// Agent 被创建 (agent_spawned)
    AgentSpawned {
        agent_id: String,
        role: String,
        parent_id: Option<String>,
        depth: usize,
    },
    /// Agent 状态变更 (agent_status)
    AgentStatus {
        agent_id: String,
        status: String, // idle, thinking, typing, searching, panicked, celebrating
    },
    /// 工具调用
    ToolCall {
        agent_id: Option<String>,
        tool: String,
        arguments: Value,
    },
    /// 工具结果
    ToolResult {
        agent_id: Option<String>,
        content: String,
        is_error: bool,
    },
    /// 工单更新 (work_order_update)
    WorkOrderUpdate {
        order_id: String,
        from: String,
        to: String,
        status: String, // assigned, completed, failed
    },
    /// 上下通管理告警 (cma_alert)
    CmaAlert {
        level: String, // info, warning, error
        message: String,
        agent_id: Option<String>,
    },
    /// 完成
    Completed,
    /// 错误
    Error { message: String },
}

impl SseEvent {
    /// 创建会话开始事件
    pub fn session_started(session_id: impl Into<String>) -> Self {
        Self::SessionStarted {
            session_id: session_id.into(),
        }
    }

    /// 创建助手消息事件
    pub fn assistant_message(agent_id: Option<String>, content: impl Into<String>) -> Self {
        Self::AssistantMessage {
            agent_id,
            content: content.into(),
        }
    }

    /// 创建 Agent 被创建事件
    pub fn agent_spawned(
        agent_id: impl Into<String>,
        role: impl Into<String>,
        parent_id: Option<String>,
        depth: usize,
    ) -> Self {
        Self::AgentSpawned {
            agent_id: agent_id.into(),
            role: role.into(),
            parent_id,
            depth,
        }
    }

    /// 创建 Agent 状态变更事件
    pub fn agent_status(agent_id: impl Into<String>, status: impl Into<String>) -> Self {
        Self::AgentStatus {
            agent_id: agent_id.into(),
            status: status.into(),
        }
    }

    /// 创建工具调用事件
    pub fn tool_call(agent_id: Option<String>, tool: impl Into<String>, arguments: Value) -> Self {
        Self::ToolCall {
            agent_id,
            tool: tool.into(),
            arguments,
        }
    }

    /// 创建工具结果事件
    pub fn tool_result(
        agent_id: Option<String>,
        content: impl Into<String>,
        is_error: bool,
    ) -> Self {
        Self::ToolResult {
            agent_id,
            content: content.into(),
            is_error,
        }
    }

    /// 创建工单更新事件
    pub fn work_order_update(
        order_id: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
        status: impl Into<String>,
    ) -> Self {
        Self::WorkOrderUpdate {
            order_id: order_id.into(),
            from: from.into(),
            to: to.into(),
            status: status.into(),
        }
    }

    /// 创建 CMA 告警事件
    pub fn cma_alert(
        level: impl Into<String>,
        message: impl Into<String>,
        agent_id: Option<String>,
    ) -> Self {
        Self::CmaAlert {
            level: level.into(),
            message: message.into(),
            agent_id,
        }
    }

    /// 创建完成事件
    pub fn completed() -> Self {
        Self::Completed
    }

    /// 创建错误事件
    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
        }
    }

    /// 序列化为 JSON 字符串
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}
