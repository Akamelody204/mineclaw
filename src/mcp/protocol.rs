//! MCP JSON-RPC 2.0 协议消息类型
//!
//! 定义 MCP 协议使用的所有请求、响应和通知结构。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ==================== JSON-RPC 2.0 基础类型 ====================

/// JSON-RPC 请求 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    Number(u64),
    String(String),
}

/// JSON-RPC 错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

/// JSON-RPC 请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: RequestId,
    pub method: String,
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}

/// JSON-RPC 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: RequestId,
    #[serde(default)]
    pub result: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 通知
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}

// ==================== MCP 通用类型 ====================

/// 实现信息（客户端/服务器）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Implementation {
    pub name: String,
    pub version: String,
}

/// 工具定义（复用 models 中的定义，但这里定义协议格式）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProtocolTool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

// ==================== 客户端能力 ====================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClientCapabilities {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sampling: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roots: Option<serde_json::Value>,
}

// ==================== 服务器能力 ====================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerCapabilities {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logging: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompts: Option<serde_json::Value>,
    #[serde(default)]
    pub tools: ToolsCapability,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resources: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolsCapability {
    #[serde(default, rename = "listChanged")]
    pub list_changed: bool,
}

// ==================== Initialize 方法 ====================

/// initialize 请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeRequest {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    #[serde(rename = "clientInfo")]
    pub client_info: Implementation,
}

/// initialize 响应结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResponse {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: Implementation,
}

// ==================== initialized 通知 ====================

/// initialized 通知（无参数）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializedNotification;

// ==================== tools/list 方法 ====================

/// tools/list 请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsRequest {
    #[serde(default)]
    pub cursor: Option<String>,
}

/// tools/list 响应结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsResponse {
    pub tools: Vec<ProtocolTool>,
    #[serde(default, rename = "nextCursor")]
    pub next_cursor: Option<String>,
}

// ==================== tools/call 方法 ====================

/// tools/call 请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolRequest {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// tools/call 响应结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResponse {
    pub content: Vec<ToolResultContent>,
    #[serde(default, rename = "isError")]
    pub is_error: bool,
}

/// 工具结果内容
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolResultContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image {
        #[serde(rename = "mimeType")]
        mime_type: String,
        data: String,
    },
    #[serde(rename = "embeddedResource")]
    EmbeddedResource {
        uri: String,
        #[serde(rename = "mimeType")]
        mime_type: Option<String>,
        text: Option<String>,
        blob: Option<String>,
    },
}

// ==================== 构造函数 ====================

impl JsonRpcRequest {
    /// 创建新的 JSON-RPC 请求
    pub fn new(id: RequestId, method: String, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method,
            params,
        }
    }
}

impl JsonRpcResponse {
    /// 创建成功响应
    pub fn success(id: RequestId, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// 创建错误响应
    pub fn error(
        id: RequestId,
        code: i32,
        message: String,
        data: Option<serde_json::Value>,
    ) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data,
            }),
        }
    }
}

impl JsonRpcNotification {
    /// 创建新的 JSON-RPC 通知
    pub fn new(method: String, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method,
            params,
        }
    }
}

impl InitializeRequest {
    /// 创建初始化请求
    pub fn new(client_name: &str, client_version: &str) -> Self {
        Self {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ClientCapabilities::default(),
            client_info: Implementation {
                name: client_name.to_string(),
                version: client_version.to_string(),
            },
        }
    }
}

impl ListToolsRequest {
    /// 创建工具列表请求
    pub fn new() -> Self {
        Self { cursor: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_jsonrpc_request_serialization() {
        let request = JsonRpcRequest::new(
            RequestId::Number(1),
            "test_method".to_string(),
            Some(json!({"key": "value"})),
        );

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: JsonRpcRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.jsonrpc, "2.0");
        assert_eq!(deserialized.id, RequestId::Number(1));
        assert_eq!(deserialized.method, "test_method");
    }

    #[test]
    fn test_jsonrpc_response_success() {
        let response = JsonRpcResponse::success(
            RequestId::String("req-123".to_string()),
            json!({"result": "ok"}),
        );

        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_jsonrpc_response_error() {
        let response = JsonRpcResponse::error(
            RequestId::Number(1),
            -32601,
            "Method not found".to_string(),
            None,
        );

        assert!(response.result.is_none());
        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32601);
    }

    #[test]
    fn test_initialize_request() {
        let request = InitializeRequest::new("mineclaw", "0.1.0");

        assert_eq!(request.protocol_version, "2024-11-05");
        assert_eq!(request.client_info.name, "mineclaw");
        assert_eq!(request.client_info.version, "0.1.0");

        let serialized = serde_json::to_string(&request).unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(json_value["protocolVersion"], "2024-11-05");
        assert_eq!(json_value["clientInfo"]["name"], "mineclaw");
    }

    #[test]
    fn test_list_tools_response() {
        let tool = ProtocolTool {
            name: "echo".to_string(),
            description: "Echo tool".to_string(),
            input_schema: json!({"type": "object"}),
        };

        let response = ListToolsResponse {
            tools: vec![tool],
            next_cursor: None,
        };

        assert_eq!(response.tools.len(), 1);
        assert_eq!(response.tools[0].name, "echo");

        let serialized = serde_json::to_string(&response).unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(
            json_value["tools"][0]["inputSchema"],
            json!({"type": "object"})
        );
    }

    #[test]
    fn test_request_id_variants() {
        let id_num = RequestId::Number(42);
        let serialized = serde_json::to_string(&id_num).unwrap();
        assert_eq!(serialized, "42");

        let id_str = RequestId::String("test-id".to_string());
        let serialized = serde_json::to_string(&id_str).unwrap();
        assert_eq!(serialized, "\"test-id\"");
    }

    #[test]
    fn test_jsonrpc_notification() {
        let notification = JsonRpcNotification::new("initialized".to_string(), None);

        let serialized = serde_json::to_string(&notification).unwrap();
        let deserialized: JsonRpcNotification = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.jsonrpc, "2.0");
        assert_eq!(deserialized.method, "initialized");
    }

    #[test]
    fn test_call_tool_request() {
        let request = CallToolRequest {
            name: "echo".to_string(),
            arguments: json!({"message": "hello"}),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        assert_eq!(json_value["name"], "echo");
        assert_eq!(json_value["arguments"]["message"], "hello");
    }

    #[test]
    fn test_call_tool_response_text() {
        let response = CallToolResponse {
            content: vec![ToolResultContent::Text {
                text: "Hello world".to_string(),
            }],
            is_error: false,
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        assert_eq!(json_value["content"][0]["type"], "text");
        assert_eq!(json_value["content"][0]["text"], "Hello world");
        assert_eq!(json_value["isError"], false);
    }

    #[test]
    fn test_call_tool_response_image() {
        let response = CallToolResponse {
            content: vec![ToolResultContent::Image {
                mime_type: "image/png".to_string(),
                data: "base64data".to_string(),
            }],
            is_error: false,
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        assert_eq!(json_value["content"][0]["type"], "image");
        assert_eq!(json_value["content"][0]["mimeType"], "image/png");
        assert_eq!(json_value["content"][0]["data"], "base64data");
    }

    #[test]
    fn test_call_tool_response_error() {
        let response = CallToolResponse {
            content: vec![ToolResultContent::Text {
                text: "Error occurred".to_string(),
            }],
            is_error: true,
        };

        assert!(response.is_error);
    }
}
