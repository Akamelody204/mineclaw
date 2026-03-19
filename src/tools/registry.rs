//! 本地工具注册表
//!
//! 管理所有本地工具，提供工具查找和调用功能。

use super::{LocalTool, ToolContext};
use crate::error::{Error, Result};
use crate::models::Tool;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;

// ==================== LocalToolRegistry ====================

/// 本地工具注册表
pub struct LocalToolRegistry {
    tools: HashMap<String, Arc<dyn LocalTool>>,
}

impl LocalToolRegistry {
    /// 创建一个新的本地工具注册表
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// 注册一个工具
    pub fn register(&mut self, tool: Arc<dyn LocalTool>) {
        let name = tool.name().to_string();
        debug!(tool_name = %name, "Registering local tool");
        self.tools.insert(name, tool);
    }

    /// 获取所有工具列表
    pub fn list_tools(&self) -> Vec<Tool> {
        self.tools
            .values()
            .map(|t| Tool {
                name: t.name().to_string(),
                description: t.description().to_string(),
                input_schema: t.input_schema(),
            })
            .collect()
    }

    /// 检查工具是否存在
    pub fn has_tool(&self, tool_name: &str) -> bool {
        self.tools.contains_key(tool_name)
    }

    /// 获取工具定义
    pub fn get_tool(&self, tool_name: &str) -> Option<Tool> {
        self.tools.get(tool_name).map(|t| Tool {
            name: t.name().to_string(),
            description: t.description().to_string(),
            input_schema: t.input_schema(),
        })
    }

    /// 调用工具
    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: Value,
        context: ToolContext,
    ) -> Result<Value> {
        let tool = self
            .tools
            .get(tool_name)
            .ok_or_else(|| Error::LocalToolNotFound(tool_name.to_string()))?;

        tool.call(arguments, context)
            .await
            .map_err(|e| Error::LocalToolExecution {
                tool: tool_name.to_string(),
                message: e.to_string(),
            })
    }

    /// 清空注册表
    pub fn clear(&mut self) {
        self.tools.clear();
    }
}

impl Default for LocalToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
