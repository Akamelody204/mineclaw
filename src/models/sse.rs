//! SSE 事件定义
//!
//! 用于 Server-Sent Events 流式响应的事件类型。

use serde::Serialize;
use serde_json::Value;

/// SSE 事件类型
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SseEvent {
    /// 助手消息
    AssistantMessage { content: String },
    /// 工具调用
    ToolCall { tool: String, arguments: Value },
    /// 工具结果
    ToolResult { content: String, is_error: bool },
    /// 完成
    Completed,
    /// 错误
    Error { message: String },
}

impl SseEvent {
    /// 创建助手消息事件
    pub fn assistant_message(content: impl Into<String>) -> Self {
        Self::AssistantMessage {
            content: content.into(),
        }
    }

    /// 创建工具调用事件
    pub fn tool_call(tool: impl Into<String>, arguments: Value) -> Self {
        Self::ToolCall {
            tool: tool.into(),
            arguments,
        }
    }

    /// 创建工具结果事件
    pub fn tool_result(content: impl Into<String>, is_error: bool) -> Self {
        Self::ToolResult {
            content: content.into(),
            is_error,
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_assistant_message_serialization() {
        let event = SseEvent::assistant_message("Hello, world!");
        let json = event.to_json().unwrap();
        let parsed: Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["type"], "assistant_message");
        assert_eq!(parsed["content"], "Hello, world!");
    }

    #[test]
    fn test_tool_call_serialization() {
        let args = json!({ "a": 1, "b": 2 });
        let event = SseEvent::tool_call("add", args.clone());
        let json = event.to_json().unwrap();
        let parsed: Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["type"], "tool_call");
        assert_eq!(parsed["tool"], "add");
        assert_eq!(parsed["arguments"], args);
    }

    #[test]
    fn test_tool_result_serialization() {
        let event = SseEvent::tool_result("3", false);
        let json = event.to_json().unwrap();
        let parsed: Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["type"], "tool_result");
        assert_eq!(parsed["content"], "3");
        assert_eq!(parsed["is_error"], false);
    }

    #[test]
    fn test_completed_serialization() {
        let event = SseEvent::completed();
        let json = event.to_json().unwrap();
        let parsed: Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["type"], "completed");
    }

    #[test]
    fn test_error_serialization() {
        let event = SseEvent::error("Something went wrong");
        let json = event.to_json().unwrap();
        let parsed: Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["type"], "error");
        assert_eq!(parsed["message"], "Something went wrong");
    }
}
