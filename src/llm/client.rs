use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

use crate::config::LlmConfig;
use crate::error::{Error, Result};
use crate::models::{Message, MessageRole, Tool, ToolCall};

// ==================== LLM Response Types ====================

/// LLM 响应，可以包含文本和/或工具调用
#[derive(Debug, Clone)]
pub struct LlmResponse {
    /// 文本内容（可能为空）
    pub text: Option<String>,
    /// 工具调用列表（可能为空）
    pub tool_calls: Vec<ToolCall>,
}

impl LlmResponse {
    /// 创建只有文本的响应
    pub fn text(text: String) -> Self {
        Self {
            text: Some(text),
            tool_calls: Vec::new(),
        }
    }

    /// 创建只有工具调用的响应
    pub fn tool_calls(tool_calls: Vec<ToolCall>) -> Self {
        Self {
            text: None,
            tool_calls,
        }
    }

    /// 创建同时包含文本和工具调用的响应
    pub fn text_with_tool_calls(text: String, tool_calls: Vec<ToolCall>) -> Self {
        Self {
            text: Some(text),
            tool_calls,
        }
    }

    /// 检查是否有工具调用
    pub fn has_tool_calls(&self) -> bool {
        !self.tool_calls.is_empty()
    }

    /// 检查是否有文本内容
    pub fn has_text(&self) -> bool {
        self.text.is_some()
    }
}

// ==================== Chat Tool Types (OpenAI Format) ====================

/// OpenAI 格式的工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatTool {
    pub r#type: String,
    pub function: ChatToolFunction,
}

/// OpenAI 格式的工具函数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// OpenAI 格式的工具调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatToolCall {
    pub id: String,
    pub r#type: String,
    pub function: ChatToolCallFunction,
}

/// OpenAI 格式的工具调用函数部分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatToolCallFunction {
    pub name: String,
    pub arguments: String,
}

// ==================== LLM Provider Trait ====================

#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// 简单的文本聊天（向后兼容）
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String>;

    /// 支持工具调用的聊天
    async fn chat_with_tools(
        &self,
        messages: Vec<ChatMessage>,
        tools: Vec<ChatTool>,
    ) -> Result<LlmResponse>;
}

// ==================== Chat Message Types ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ChatToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

// ==================== Message Conversion ====================

impl From<(MessageRole, String)> for ChatMessage {
    fn from((role, content): (MessageRole, String)) -> Self {
        let role_str = match role {
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::System => "system",
            MessageRole::ToolCall => "assistant",
            MessageRole::ToolResult => "tool",
        };
        Self {
            role: role_str.to_string(),
            content: Some(content),
            tool_calls: None,
            tool_call_id: None,
        }
    }
}

impl ChatMessage {
    /// 从 Message 转换为 ChatMessage
    pub fn from_message(message: &Message) -> Self {
        let role_str = match message.role {
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::System => "system",
            MessageRole::ToolCall => "assistant",
            MessageRole::ToolResult => "tool",
        };

        // 重要：确保 content 字段始终存在
        // 某些 API（如火山引擎）要求所有消息都必须有 content 字段
        let content = if message.content.is_empty() {
            // 对于空内容，使用一个默认值而不是 None
            match message.role {
                MessageRole::ToolCall => Some("".to_string()),
                MessageRole::ToolResult => Some("".to_string()),
                _ => Some("".to_string()),
            }
        } else {
            Some(message.content.clone())
        };

        let tool_calls = message.tool_calls.as_ref().map(|calls| {
            calls
                .iter()
                .map(|call| ChatToolCall {
                    id: call.id.clone(),
                    r#type: "function".to_string(),
                    function: ChatToolCallFunction {
                        name: call.name.clone(),
                        arguments: call.arguments.to_string(),
                    },
                })
                .collect()
        });

        let tool_call_id = message.tool_result.as_ref().map(|r| r.tool_call_id.clone());

        Self {
            role: role_str.to_string(),
            content,
            tool_calls,
            tool_call_id,
        }
    }

    /// 从 Tool 转换为 ChatTool
    pub fn tool_to_chat_tool(tool: &Tool) -> ChatTool {
        ChatTool {
            r#type: "function".to_string(),
            function: ChatToolFunction {
                name: tool.name.clone(),
                description: tool.description.clone(),
                parameters: tool.input_schema.clone(),
            },
        }
    }

    /// 从 ChatToolCall 转换为 ToolCall
    pub fn chat_tool_call_to_tool_call(chat_call: &ChatToolCall) -> Result<ToolCall> {
        let arguments = serde_json::from_str(&chat_call.function.arguments).map_err(|e| {
            Error::Llm(format!("Failed to parse tool arguments: {}", e))
        })?;

        Ok(ToolCall {
            id: chat_call.id.clone(),
            name: chat_call.function.name.clone(),
            arguments,
        })
    }
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ChatTool>>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<ChatToolCall>>,
}

pub struct OpenAiProvider {
    config: LlmConfig,
    client: Client,
}

impl OpenAiProvider {
    pub fn new(config: LlmConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String> {
        let response = self.chat_with_tools(messages, vec![]).await?;
        match response.text {
            Some(text) => Ok(text),
            None => Err(Error::Llm("No text response from LLM".into())),
        }
    }

    async fn chat_with_tools(
        &self,
        messages: Vec<ChatMessage>,
        tools: Vec<ChatTool>,
    ) -> Result<LlmResponse> {
        info!(
            "LLM chat_with_tools request: model={}, message_count={}, tool_count={}",
            self.config.model,
            messages.len(),
            tools.len()
        );

        let request = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages,
            temperature: self.config.temperature,
            tools: if tools.is_empty() { None } else { Some(tools) },
        };

        // 调试：打印序列化后的请求
        if let Ok(json) = serde_json::to_string_pretty(&request) {
            debug!("Request JSON:\n{}", json);
        }

        let url = format!("{}/chat/completions", self.config.base_url);

        debug!("Sending request to LLM API: {}", url);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Llm(format!(
                "LLM request failed: {} - {}",
                status, text
            )));
        }

        info!("LLM API response received, status: {}", response.status());

        let completion: ChatCompletionResponse = response.json().await?;

        info!("LLM response parsed, choices: {}", completion.choices.len());

        let choice = completion
            .choices
            .first()
            .ok_or_else(|| Error::Llm("No response from LLM".into()))?;

        let message = &choice.message;

        // 解析工具调用
        let tool_calls = if let Some(chat_tool_calls) = &message.tool_calls {
            debug!("LLM returned {} tool calls", chat_tool_calls.len());
            let mut calls = Vec::new();
            for chat_call in chat_tool_calls {
                let call = ChatMessage::chat_tool_call_to_tool_call(chat_call)?;
                calls.push(call);
            }
            calls
        } else {
            Vec::new()
        };

        // 返回响应（可以同时包含文本和工具调用）
        Ok(LlmResponse {
            text: message.content.clone(),
            tool_calls,
        })
    }
}

pub fn create_provider(config: LlmConfig) -> Arc<dyn LlmProvider> {
    match config.provider.to_lowercase().as_str() {
        "openai" => Arc::new(OpenAiProvider::new(config)),
        _ => Arc::new(OpenAiProvider::new(config)),
    }
}
