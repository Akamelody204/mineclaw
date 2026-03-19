//! Agent 建造者模式
//!
//! 提供流畅的 API 来创建和配置 Agent 实例。

use crate::agent::types::{Agent, AgentCapability, AgentConfig, AgentId, AgentRole, LlmConfig};
use crate::error::{Error, Result};

/// Agent 建造者
///
/// 提供流畅的链式 API 来创建 Agent 实例。
///
/// # 示例
///
/// ```
/// use mineclaw::agent::builder::AgentBuilder;
/// use mineclaw::agent::types::{AgentRole, LlmConfig};
///
/// let agent = AgentBuilder::new()
///     .name("My Agent".to_string())
///     .role(AgentRole::Worker)
///     .llm_config(LlmConfig::new("gpt-4".to_string()))
///     .system_prompt("You are a helpful assistant.".to_string())
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct AgentBuilder {
    name: Option<String>,
    role: Option<AgentRole>,
    capabilities: Vec<AgentCapability>,
    llm_config: Option<LlmConfig>,
    system_prompt: Option<String>,
    nested_depth: Option<u8>,
    parent_orchestrator_id: Option<AgentId>,
}

impl AgentBuilder {
    /// 创建一个新的 AgentBuilder
    pub fn new() -> Self {
        Self {
            name: None,
            role: None,
            capabilities: Vec::new(),
            llm_config: None,
            system_prompt: None,
            nested_depth: None,
            parent_orchestrator_id: None,
        }
    }

    /// 设置 Agent 名称（必填）
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// 设置 Agent 角色（必填）
    pub fn role(mut self, role: AgentRole) -> Self {
        self.role = Some(role);
        self
    }

    /// 添加单个能力标签
    pub fn capability(mut self, capability: AgentCapability) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// 设置多个能力标签
    pub fn capabilities(mut self, capabilities: Vec<AgentCapability>) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// 添加多个能力标签（追加模式）
    pub fn add_capabilities(mut self, mut capabilities: Vec<AgentCapability>) -> Self {
        self.capabilities.append(&mut capabilities);
        self
    }

    /// 设置 LLM 配置（必填）
    pub fn llm_config(mut self, llm_config: LlmConfig) -> Self {
        self.llm_config = Some(llm_config);
        self
    }

    /// 设置系统提示词（必填）
    pub fn system_prompt(mut self, system_prompt: String) -> Self {
        self.system_prompt = Some(system_prompt);
        self
    }

    /// 设置嵌套深度（仅用于 SubOrchestrator）
    pub fn nested_depth(mut self, depth: u8) -> Self {
        self.nested_depth = Some(depth);
        self
    }

    /// 设置父总控 ID（仅用于 SubOrchestrator）
    pub fn parent_orchestrator(mut self, parent_id: AgentId) -> Self {
        self.parent_orchestrator_id = Some(parent_id);
        self
    }

    /// 构建 Agent 实例
    ///
    /// # 返回
    /// 返回创建的 Agent 或错误
    pub fn build(self) -> Result<Agent> {
        // 验证必填字段
        let name = self
            .name
            .ok_or_else(|| Error::AgentInvalidConfig("Agent name is required".to_string()))?;

        let role = self
            .role
            .ok_or_else(|| Error::AgentInvalidConfig("Agent role is required".to_string()))?;

        let llm_config = self
            .llm_config
            .ok_or_else(|| Error::AgentInvalidConfig("LLM config is required".to_string()))?;

        let system_prompt = self
            .system_prompt
            .ok_or_else(|| Error::AgentInvalidConfig("System prompt is required".to_string()))?;

        // 创建配置
        let mut config = AgentConfig::new(name, role, llm_config, system_prompt);

        // 设置能力标签
        config = config.with_capabilities(self.capabilities);

        // 设置嵌套配置（如果有）
        if let Some(depth) = self.nested_depth {
            config = config.with_nested_depth(depth);
        }

        if let Some(parent_id) = self.parent_orchestrator_id {
            config = config.with_parent_orchestrator(parent_id);
        }

        // 创建 Agent（会进行配置验证）
        let agent = Agent::new(config);

        Ok(agent)
    }

    /// 构建 AgentConfig（不创建 Agent 实例）
    ///
    /// 适用于需要先验证配置或稍后创建 Agent 的场景。
    ///
    /// # 返回
    /// 返回创建的 AgentConfig 或错误
    pub fn build_config(self) -> Result<AgentConfig> {
        // 验证必填字段
        let name = self
            .name
            .ok_or_else(|| Error::AgentInvalidConfig("Agent name is required".to_string()))?;

        let role = self
            .role
            .ok_or_else(|| Error::AgentInvalidConfig("Agent role is required".to_string()))?;

        let llm_config = self
            .llm_config
            .ok_or_else(|| Error::AgentInvalidConfig("LLM config is required".to_string()))?;

        let system_prompt = self
            .system_prompt
            .ok_or_else(|| Error::AgentInvalidConfig("System prompt is required".to_string()))?;

        // 创建配置
        let mut config = AgentConfig::new(name, role, llm_config, system_prompt);

        // 设置能力标签
        config = config.with_capabilities(self.capabilities);

        // 设置嵌套配置（如果有）
        if let Some(depth) = self.nested_depth {
            config = config.with_nested_depth(depth);
        }

        if let Some(parent_id) = self.parent_orchestrator_id {
            config = config.with_parent_orchestrator(parent_id);
        }

        // 验证配置
        config.validate()?;

        Ok(config)
    }
}

impl Default for AgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 便捷的 Worker Agent 建造者
///
/// 专门用于快速创建 Worker 类型的 Agent。
pub struct WorkerAgentBuilder {
    builder: AgentBuilder,
}

impl WorkerAgentBuilder {
    /// 创建一个新的 WorkerAgentBuilder
    pub fn new() -> Self {
        Self {
            builder: AgentBuilder::new().role(AgentRole::Worker),
        }
    }

    /// 设置 Agent 名称
    pub fn name(mut self, name: String) -> Self {
        self.builder = self.builder.name(name);
        self
    }

    /// 添加能力标签
    pub fn capability(mut self, capability: AgentCapability) -> Self {
        self.builder = self.builder.capability(capability);
        self
    }

    /// 设置多个能力标签
    pub fn capabilities(mut self, capabilities: Vec<AgentCapability>) -> Self {
        self.builder = self.builder.capabilities(capabilities);
        self
    }

    /// 设置 LLM 配置
    pub fn llm_config(mut self, llm_config: LlmConfig) -> Self {
        self.builder = self.builder.llm_config(llm_config);
        self
    }

    /// 设置系统提示词
    pub fn system_prompt(mut self, system_prompt: String) -> Self {
        self.builder = self.builder.system_prompt(system_prompt);
        self
    }

    /// 构建 Worker Agent
    pub fn build(self) -> Result<Agent> {
        self.builder.build()
    }
}

impl Default for WorkerAgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}
