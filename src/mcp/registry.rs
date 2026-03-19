//! MCP 工具注册表
//!
//! 管理多个 MCP 服务器的工具，提供工具查找和路由功能。

use crate::models::Tool;
use std::collections::HashMap;
use tracing::{debug, warn};

// ==================== ToolRegistry ====================

/// 工具注册表
pub struct ToolRegistry {
    /// 工具名称到服务器名称的映射
    /// 注意：如果有同名工具，只会保留最后注册的那个
    tool_to_server: HashMap<String, String>,
    /// 服务器名称到工具列表的映射
    server_tools: HashMap<String, Vec<Tool>>,
}

impl ToolRegistry {
    /// 创建一个新的工具注册表
    pub fn new() -> Self {
        Self {
            tool_to_server: HashMap::new(),
            server_tools: HashMap::new(),
        }
    }

    /// 注册一个服务器的所有工具
    pub fn register_server(&mut self, server_name: String, tools: Vec<Tool>) {
        debug!(server_name = %server_name, tool_count = tools.len(), "Registering server tools");

        // 更新服务器到工具的映射
        self.server_tools.insert(server_name.clone(), tools.clone());

        // 更新工具到服务器的映射
        for tool in tools {
            if let Some(existing_server) = self
                .tool_to_server
                .insert(tool.name.clone(), server_name.clone())
            {
                warn!(
                    tool_name = %tool.name,
                    old_server = %existing_server,
                    new_server = %server_name,
                    "Tool name conflict, overwriting"
                );
            }
        }
    }

    /// 注销一个服务器的所有工具
    pub fn unregister_server(&mut self, server_name: &str) {
        debug!(server_name = %server_name, "Unregistering server tools");

        if let Some(tools) = self.server_tools.remove(server_name) {
            for tool in tools {
                if let Some(registered_server) = self.tool_to_server.get(&tool.name)
                    && registered_server == server_name
                {
                    self.tool_to_server.remove(&tool.name);
                }
            }
        }
    }

    /// 查找工具所在的服务器
    pub fn find_server(&self, tool_name: &str) -> Option<&str> {
        self.tool_to_server.get(tool_name).map(|s| s.as_str())
    }

    /// 获取所有工具列表
    pub fn all_tools(&self) -> Vec<(String, Tool)> {
        let mut result = Vec::new();
        for (server_name, tools) in &self.server_tools {
            for tool in tools {
                result.push((server_name.clone(), tool.clone()));
            }
        }
        result
    }

    /// 获取指定服务器的工具列表
    pub fn server_tools(&self, server_name: &str) -> Option<&[Tool]> {
        self.server_tools.get(server_name).map(|v| v.as_slice())
    }

    /// 检查工具是否存在
    pub fn has_tool(&self, tool_name: &str) -> bool {
        self.tool_to_server.contains_key(tool_name)
    }

    /// 获取工具定义
    pub fn get_tool(&self, tool_name: &str) -> Option<&Tool> {
        let server_name = self.tool_to_server.get(tool_name)?;
        let tools = self.server_tools.get(server_name)?;
        tools.iter().find(|t| t.name == tool_name)
    }

    /// 清空注册表
    pub fn clear(&mut self) {
        self.tool_to_server.clear();
        self.server_tools.clear();
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
