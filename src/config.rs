use crate::encryption::EncryptionManager;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{info, warn};

/// Checkpoint 配置
#[derive(Debug, Deserialize, Clone)]
pub struct CheckpointConfig {
    /// 是否启用 checkpoint
    #[serde(default = "default_checkpoint_enabled")]
    pub enabled: bool,
    /// Checkpoint 存储目录（agentfs 路径）
    #[serde(default = "default_checkpoint_directory")]
    pub checkpoint_directory: String,
}

fn default_checkpoint_enabled() -> bool {
    true
}

fn default_checkpoint_directory() -> String {
    ".checkpoints".to_string()
}

impl Default for CheckpointConfig {
    fn default() -> Self {
        Self {
            enabled: default_checkpoint_enabled(),
            checkpoint_directory: default_checkpoint_directory(),
        }
    }
}

/// 终端工具配置
#[derive(Debug, Deserialize, Clone)]
pub struct TerminalConfig {
    /// 是否启用终端工具
    #[serde(default = "default_terminal_enabled")]
    pub enabled: bool,
    /// 最大输出字节数
    #[serde(default = "default_terminal_max_output_bytes")]
    pub max_output_bytes: usize,
    /// 超时秒数
    #[serde(default = "default_terminal_timeout_seconds")]
    pub timeout_seconds: u64,
    /// 最大并发进程数
    #[serde(default = "default_terminal_max_concurrent_processes")]
    pub max_concurrent_processes: usize,
    /// 允许的工作目录
    #[serde(default)]
    pub allowed_workspaces: Vec<String>,
    /// 命令黑名单
    #[serde(default)]
    pub command_blacklist: Vec<String>,
    /// 命令正则表达式黑名单
    #[serde(default)]
    pub command_blacklist_regex: Vec<String>,
    /// 始终允许的命令正则表达式
    #[serde(default)]
    pub always_allow_regex: Vec<String>,
    /// 需要确认的命令正则表达式
    #[serde(default)]
    pub always_confirm_regex: Vec<String>,
    /// 后台任务存活时间 (分钟) (Phase EX3)
    #[serde(default = "default_background_task_ttl_minutes")]
    pub background_task_ttl_minutes: u64,

    /// 编译后的黑名单正则
    #[serde(skip)]
    pub compiled_blacklist: Vec<Regex>,
    /// 编译后的始终允许正则
    #[serde(skip)]
    pub compiled_always_allow: Vec<Regex>,
    /// 编译后的始终确认正则
    #[serde(skip)]
    pub compiled_always_confirm: Vec<Regex>,
    /// 过滤规则
    #[serde(default)]
    pub filters: HashMap<String, Vec<String>>,
}

fn default_terminal_enabled() -> bool {
    true
}

fn default_terminal_max_output_bytes() -> usize {
    1048576 // 1MB
}

fn default_terminal_timeout_seconds() -> u64 {
    300
}

fn default_background_task_ttl_minutes() -> u64 {
    30
}

fn default_terminal_max_concurrent_processes() -> usize {
    4
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            enabled: default_terminal_enabled(),
            max_output_bytes: default_terminal_max_output_bytes(),
            timeout_seconds: default_terminal_timeout_seconds(),
            max_concurrent_processes: default_terminal_max_concurrent_processes(),
            allowed_workspaces: Vec::new(),
            command_blacklist: Vec::new(),
            command_blacklist_regex: Vec::new(),
            always_allow_regex: Vec::new(),
            always_confirm_regex: Vec::new(),
            background_task_ttl_minutes: default_background_task_ttl_minutes(),
            compiled_blacklist: Vec::new(),
            compiled_always_allow: Vec::new(),
            compiled_always_confirm: Vec::new(),
            filters: HashMap::new(),
        }
    }
}

/// 本地工具配置
#[derive(Debug, Deserialize, Clone, Default)]
pub struct LocalToolsConfig {
    #[serde(default)]
    pub terminal: TerminalConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub llm: LlmConfig,
    #[serde(default)]
    pub mcp: Option<McpConfig>,
    #[serde(default)]
    pub filesystem: FilesystemConfig,
    #[serde(default)]
    pub local_tools: LocalToolsConfig,
    #[serde(default)]
    pub checkpoint: CheckpointConfig,
    #[serde(default = "default_agentfs_db_path")]
    pub agentfs_db_path: String,
    pub encryption: Option<EncryptionConfig>,
    /// 命名模型配置档案映射（key 为档案名称，如 "gpt-4o"、"claude-3-opus"）
    /// 可选：为空时系统行为与之前一致，所有 Agent 使用 Config.llm
    #[serde(default)]
    pub models: HashMap<String, ModelProfile>,
    /// 默认模型档案名称，"default" 特指 Config.llm 段
    #[serde(default = "default_default_model")]
    pub default_model: String,
}

fn default_agentfs_db_path() -> String {
    "data/mineclaw.db".to_string()
}

fn default_default_model() -> String {
    "default".to_string()
}

#[derive(Debug, Deserialize, Clone)]
pub struct EncryptionConfig {
    // 加密密钥通过环境变量 MINECLAW_ENCRYPTION_KEY 提供，不需要在文件中配置
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LlmConfig {
    pub provider: String,
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f64,
}

/// 模型配置档案
///
/// 定义一个命名的 LLM 模型配置，包含连接信息和元数据。
/// 字段 `api_key`、`base_url`、`max_tokens`、`temperature` 为 Optional，
/// 未指定时从 `Config.llm`（默认配置）继承。
#[derive(Debug, Deserialize, Clone)]
pub struct ModelProfile {
    /// LLM 提供商（如 "openai"、"anthropic"）
    pub provider: String,
    /// API Key（可选，为 None 时从 Config.llm 继承）
    pub api_key: Option<String>,
    /// Base URL（可选，为 None 时从 Config.llm 继承）
    pub base_url: Option<String>,
    /// 模型名称（如 "gpt-4o"、"claude-3-opus-20240229"）
    pub model: String,
    /// 上下文窗口大小（token 数），如 128000
    pub context_window: Option<u32>,
    /// 每 1000 input token 的成本（USD）
    pub cost_per_1k_input: Option<f64>,
    /// 每 1000 output token 的成本（USD）
    pub cost_per_1k_output: Option<f64>,
    /// 能力等级：如 "flagship"、"standard"、"budget"
    pub capability_tier: Option<String>,
    /// 最大输出 token 数（可选，为 None 时从 Config.llm 继承）
    pub max_tokens: Option<u32>,
    /// 温度参数（可选，为 None 时从 Config.llm 继承）
    pub temperature: Option<f64>,
}

/// 完全解析后的模型配置
///
/// 所有字段已填充（无 Optional），可直接用于创建 LlmProvider。
/// 由 `Config::resolve_model_profile()` 生成。
#[derive(Debug, Clone)]
pub struct ResolvedModelProfile {
    /// LLM 提供商
    pub provider: String,
    /// API Key（已解析，非空）
    pub api_key: String,
    /// Base URL（已解析）
    pub base_url: String,
    /// 模型名称
    pub model: String,
    /// 最大输出 token 数
    pub max_tokens: u32,
    /// 温度参数
    pub temperature: f64,
    /// 上下文窗口大小（可能未配置）
    pub context_window: Option<u32>,
    /// 每 1000 input token 的成本（可能未配置）
    pub cost_per_1k_input: Option<f64>,
    /// 每 1000 output token 的成本（可能未配置）
    pub cost_per_1k_output: Option<f64>,
    /// 能力等级（可能未配置）
    pub capability_tier: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct McpConfig {
    pub enabled: bool,
    #[serde(default)]
    pub servers: Vec<McpServerConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FilesystemConfig {
    #[serde(default = "default_max_read_bytes")]
    pub max_read_bytes: usize,
    #[serde(default)]
    pub allowed_directories: Vec<String>,
}

fn default_max_read_bytes() -> usize {
    16384
}

impl Default for FilesystemConfig {
    fn default() -> Self {
        Self {
            max_read_bytes: default_max_read_bytes(),
            allowed_directories: Vec::new(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 18789,
            },
            llm: LlmConfig {
                provider: "openai".to_string(),
                api_key: "".to_string(),
                base_url: "https://api.openai.com/v1".to_string(),
                model: "gpt-4o".to_string(),
                max_tokens: 2048,
                temperature: 0.7,
            },
            mcp: None,
            filesystem: FilesystemConfig::default(),
            local_tools: LocalToolsConfig::default(),
            checkpoint: CheckpointConfig::default(),
            agentfs_db_path: default_agentfs_db_path(),
            encryption: None,
            models: HashMap::new(),
            default_model: default_default_model(),
        }
    }
}

impl Config {
    pub fn load() -> crate::error::Result<Self> {
        let config_path = Self::get_config_path()?;

        let mut settings = config::Config::builder();

        let default_config = Config::default();
        settings = settings
            .set_default("server.host", default_config.server.host)
            .map_err(crate::error::Error::Config)?
            .set_default("server.port", default_config.server.port)
            .map_err(crate::error::Error::Config)?
            .set_default("llm.provider", default_config.llm.provider)
            .map_err(crate::error::Error::Config)?
            .set_default("llm.api_key", default_config.llm.api_key)
            .map_err(crate::error::Error::Config)?
            .set_default("llm.base_url", default_config.llm.base_url)
            .map_err(crate::error::Error::Config)?
            .set_default("llm.model", default_config.llm.model)
            .map_err(crate::error::Error::Config)?
            .set_default("llm.max_tokens", default_config.llm.max_tokens)
            .map_err(crate::error::Error::Config)?
            .set_default("llm.temperature", default_config.llm.temperature)
            .map_err(crate::error::Error::Config)?
            .set_default("agentfs_db_path", default_config.agentfs_db_path)
            .map_err(crate::error::Error::Config)?
            .set_default(
                "local_tools.terminal.enabled",
                default_config.local_tools.terminal.enabled,
            )
            .map_err(crate::error::Error::Config)?
            .set_default(
                "local_tools.terminal.max_output_bytes",
                default_config.local_tools.terminal.max_output_bytes as i64,
            )
            .map_err(crate::error::Error::Config)?
            .set_default(
                "local_tools.terminal.timeout_seconds",
                default_config.local_tools.terminal.timeout_seconds,
            )
            .map_err(crate::error::Error::Config)?
            .set_default(
                "local_tools.terminal.max_concurrent_processes",
                default_config.local_tools.terminal.max_concurrent_processes as i64,
            )
            .map_err(crate::error::Error::Config)?;

        if config_path.exists() {
            settings = settings.add_source(config::File::from(config_path.clone()));
        }

        let settings = settings
            .add_source(config::Environment::with_prefix("MINECLAW").separator("__"))
            .build()?;

        let mut config = settings.try_deserialize::<Config>()?;

        // 检查环境变量中是否有加密密钥
        let encryption_key_env = std::env::var("MINECLAW_ENCRYPTION_KEY").ok();

        // 处理 API Key
        if config.llm.api_key.starts_with("encrypted:") {
            // 情况1：已经是加密的 API Key，需要解密
            let key = encryption_key_env.ok_or_else(|| {
                crate::error::Error::Config(config::ConfigError::Message(
                    "Encrypted API Key detected but MINECLAW_ENCRYPTION_KEY is missing, please make sure it is in your env".to_string(),
                ))
            })?;

            let manager = EncryptionManager::new(&key).map_err(|e| {
                crate::error::Error::Config(config::ConfigError::Message(format!(
                    "Invalid encryption key: {}",
                    e
                )))
            })?;

            let cipher_text = config.llm.api_key.trim_start_matches("encrypted:");
            let plain_text = manager.decrypt(cipher_text).map_err(|e| {
                crate::error::Error::Config(config::ConfigError::Message(format!(
                    "Failed to decrypt LLM API Key: {}",
                    e
                )))
            })?;

            info!("Successfully decrypted LLM API Key");
            config.llm.api_key = plain_text;
        } else if !config.llm.api_key.is_empty() {
            // 情况2：明文 API Key
            if let Some(key) = encryption_key_env {
                // 有加密密钥，自动加密并写回配置文件
                match EncryptionManager::new(&key) {
                    Ok(manager) => match manager.encrypt(&config.llm.api_key) {
                        Ok(encrypted) => {
                            info!("API Key encrypted successfully");

                            // 尝试写回配置文件
                            if config_path.exists() {
                                match Self::update_config_with_encrypted_key(
                                    &config_path,
                                    &encrypted,
                                ) {
                                    Ok(_) => {
                                        info!("Config file updated with encrypted API Key");
                                    }
                                    Err(e) => {
                                        warn!("Failed to update config file: {}", e);
                                        info!(
                                            "To store it securely, update your config file with:"
                                        );
                                        info!("llm.api_key = \"encrypted:{}\"", encrypted);
                                    }
                                }
                            } else {
                                info!(
                                    "Config file not found. To store it securely, create a config file with:"
                                );
                                info!("llm.api_key = \"encrypted:{}\"", encrypted);
                            }
                        }
                        Err(e) => {
                            warn!("Failed to encrypt API Key: {}", e);
                        }
                    },
                    Err(e) => {
                        warn!("Invalid encryption key in environment variable: {}", e);
                    }
                }
            } else {
                // 没有加密密钥，发出警告
                warn!(
                    "API Key is stored in plaintext. For better security, set MINECLAW_ENCRYPTION_KEY environment variable and encrypt your API Key."
                );
            }
        }

        // 编译正则表达式（用于终端工具）
        let mut compiled_blacklist = Vec::new();
        for regex_str in &config.local_tools.terminal.command_blacklist_regex {
            match Regex::new(regex_str) {
                Ok(re) => compiled_blacklist.push(re),
                Err(e) => {
                    warn!("Failed to compile blacklist regex '{}': {}", regex_str, e);
                }
            }
        }
        config.local_tools.terminal.compiled_blacklist = compiled_blacklist;

        let mut compiled_always_allow = Vec::new();
        for regex_str in &config.local_tools.terminal.always_allow_regex {
            match Regex::new(regex_str) {
                Ok(re) => compiled_always_allow.push(re),
                Err(e) => {
                    warn!(
                        "Failed to compile always_allow regex '{}': {}",
                        regex_str, e
                    );
                }
            }
        }
        config.local_tools.terminal.compiled_always_allow = compiled_always_allow;

        let mut compiled_always_confirm = Vec::new();
        for regex_str in &config.local_tools.terminal.always_confirm_regex {
            match Regex::new(regex_str) {
                Ok(re) => compiled_always_confirm.push(re),
                Err(e) => {
                    warn!(
                        "Failed to compile always_confirm regex '{}': {}",
                        regex_str, e
                    );
                }
            }
        }
        config.local_tools.terminal.compiled_always_confirm = compiled_always_confirm;

        Ok(config)
    }

    /// 更新配置文件，将 API Key 替换为加密版本
    fn update_config_with_encrypted_key(
        config_path: &PathBuf,
        encrypted_key: &str,
    ) -> crate::error::Result<()> {
        use toml_edit::{DocumentMut, value};

        let content = fs::read_to_string(config_path).map_err(|e| {
            crate::error::Error::Config(config::ConfigError::Message(format!(
                "Failed to read config file: {}",
                e
            )))
        })?;

        let mut doc = content.parse::<DocumentMut>().map_err(|e| {
            crate::error::Error::Config(config::ConfigError::Message(format!(
                "Failed to parse config file: {}",
                e
            )))
        })?;

        // 更新 llm.api_key
        if let Some(llm) = doc.get_mut("llm").and_then(|t| t.as_table_mut()) {
            llm["api_key"] = value(format!("encrypted:{}", encrypted_key));
        } else {
            return Err(crate::error::Error::Config(config::ConfigError::Message(
                "Config file missing [llm] section".to_string(),
            )));
        }

        // 写回文件
        fs::write(config_path, doc.to_string()).map_err(|e| {
            crate::error::Error::Config(config::ConfigError::Message(format!(
                "Failed to write config file: {}",
                e
            )))
        })?;

        Ok(())
    }

    fn get_config_path() -> crate::error::Result<PathBuf> {
        Ok(PathBuf::from("config/mineclaw.toml"))
    }

    /// 解析模型档案为完全填充的配置
    ///
    /// - `"default"` 或与 `default_model` 相同的名称 → 使用 `Config.llm` 段
    /// - 其他名称 → 从 `models` 中查找，缺失字段从 `Config.llm` 继承
    /// - 找不到 → 返回 `Error::ModelProfileNotFound`
    pub fn resolve_model_profile(
        &self,
        profile_name: &str,
    ) -> crate::error::Result<ResolvedModelProfile> {
        if profile_name == "default" || profile_name == self.default_model {
            // 特殊名称 "default" 或与 default_model 相同 → 直接使用 llm 段
            return Ok(ResolvedModelProfile {
                provider: self.llm.provider.clone(),
                api_key: self.llm.api_key.clone(),
                base_url: self.llm.base_url.clone(),
                model: self.llm.model.clone(),
                max_tokens: self.llm.max_tokens,
                temperature: self.llm.temperature,
                context_window: None,
                cost_per_1k_input: None,
                cost_per_1k_output: None,
                capability_tier: None,
            });
        }

        // 从 models 中查找命名档案
        let profile = self.models.get(profile_name).ok_or_else(|| {
            crate::error::Error::ModelProfileNotFound(format!(
                "Model profile '{}' not found. Available: default{}",
                profile_name,
                if self.models.is_empty() {
                    String::new()
                } else {
                    format!(
                        ", {}",
                        self.models.keys().cloned().collect::<Vec<_>>().join(", ")
                    )
                }
            ))
        })?;

        // 解析：profile 字段优先，缺失时从 llm 段继承
        Ok(ResolvedModelProfile {
            provider: profile.provider.clone(),
            api_key: profile
                .api_key
                .clone()
                .unwrap_or_else(|| self.llm.api_key.clone()),
            base_url: profile
                .base_url
                .clone()
                .unwrap_or_else(|| self.llm.base_url.clone()),
            model: profile.model.clone(),
            max_tokens: profile.max_tokens.unwrap_or(self.llm.max_tokens),
            temperature: profile.temperature.unwrap_or(self.llm.temperature),
            context_window: profile.context_window,
            cost_per_1k_input: profile.cost_per_1k_input,
            cost_per_1k_output: profile.cost_per_1k_output,
            capability_tier: profile.capability_tier.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();

        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 18789);
        assert_eq!(config.llm.provider, "openai");
        assert!(config.mcp.is_none());
    }

    #[test]
    fn test_mcp_config_deserialization() {
        let toml_content = r#"
[server]
host = "127.0.0.1"
port = 18789

[llm]
provider = "openai"
api_key = "test-key"
base_url = "https://api.openai.com/v1"
model = "gpt-4o"
max_tokens = 2048
temperature = 0.7

[mcp]
enabled = true

[[mcp.servers]]
name = "filesystem"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/test"]
env = { "TEST_ENV" = "value" }
"#;

        let config: Config = toml::from_str(toml_content).unwrap();

        assert!(config.mcp.is_some());
        let mcp = config.mcp.unwrap();
        assert!(mcp.enabled);
        assert_eq!(mcp.servers.len(), 1);

        let server = &mcp.servers[0];
        assert_eq!(server.name, "filesystem");
        assert_eq!(server.command, "npx");
        assert_eq!(
            server.args,
            vec!["-y", "@modelcontextprotocol/server-filesystem", "/test"]
        );
        assert_eq!(server.env.get("TEST_ENV"), Some(&"value".to_string()));
    }

    #[test]
    fn test_mcp_config_without_servers() {
        let toml_content = r#"
[server]
host = "127.0.0.1"
port = 18789

[llm]
provider = "openai"
api_key = "test-key"
base_url = "https://api.openai.com/v1"
model = "gpt-4o"
max_tokens = 2048
temperature = 0.7

[mcp]
enabled = false
"#;

        let config: Config = toml::from_str(toml_content).unwrap();

        assert!(config.mcp.is_some());
        let mcp = config.mcp.unwrap();
        assert!(!mcp.enabled);
        assert!(mcp.servers.is_empty());
    }

    #[test]
    fn test_mcp_config_without_env_and_args() {
        let toml_content = r#"
[server]
host = "127.0.0.1"
port = 18789

[llm]
provider = "openai"
api_key = "test-key"
base_url = "https://api.openai.com/v1"
model = "gpt-4o"
max_tokens = 2048
temperature = 0.7

[mcp]
enabled = true

[[mcp.servers]]
name = "simple"
command = "echo"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();

        let mcp = config.mcp.unwrap();
        let server = &mcp.servers[0];
        assert!(server.args.is_empty());
        assert!(server.env.is_empty());
    }

    #[test]
    fn test_model_profile_config_deserialization() {
        let toml_content = r#"
[server]
host = "127.0.0.1"
port = 18789

[llm]
provider = "openai"
api_key = "sk-default-key"
base_url = "https://api.openai.com/v1"
model = "gpt-4o"
max_tokens = 2048
temperature = 0.7

[models.gpt-4o]
provider = "openai"
model = "gpt-4o"
context_window = 128000
cost_per_1k_input = 0.005
cost_per_1k_output = 0.015
capability_tier = "flagship"

[models.claude-3-opus]
provider = "anthropic"
model = "claude-3-opus-20240229"
api_key = "sk-ant-custom"
base_url = "https://api.anthropic.com"
context_window = 200000
cost_per_1k_input = 0.015
cost_per_1k_output = 0.075
capability_tier = "flagship"
max_tokens = 4096
temperature = 0.5

[models.gpt-4o-mini]
provider = "openai"
model = "gpt-4o-mini"
context_window = 128000
cost_per_1k_input = 0.00015
cost_per_1k_output = 0.0006
capability_tier = "budget"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();

        // 验证默认字段
        assert_eq!(config.default_model, "default");
        assert_eq!(config.models.len(), 3);

        // 验证 gpt-4o profile
        let gpt4o = &config.models["gpt-4o"];
        assert_eq!(gpt4o.provider, "openai");
        assert_eq!(gpt4o.model, "gpt-4o");
        assert_eq!(gpt4o.context_window, Some(128000));
        assert_eq!(gpt4o.cost_per_1k_input, Some(0.005));
        assert_eq!(gpt4o.capability_tier, Some("flagship".to_string()));
        // 未指定 api_key → None（继承）
        assert!(gpt4o.api_key.is_none());
        assert!(gpt4o.base_url.is_none());
        assert!(gpt4o.max_tokens.is_none());

        // 验证 claude-3-opus profile（有自定义 api_key、base_url、max_tokens、temperature）
        let claude = &config.models["claude-3-opus"];
        assert_eq!(claude.provider, "anthropic");
        assert_eq!(claude.api_key, Some("sk-ant-custom".to_string()));
        assert_eq!(
            claude.base_url,
            Some("https://api.anthropic.com".to_string())
        );
        assert_eq!(claude.max_tokens, Some(4096));
        assert_eq!(claude.temperature, Some(0.5));

        // 验证 budget profile
        let mini = &config.models["gpt-4o-mini"];
        assert_eq!(mini.capability_tier, Some("budget".to_string()));
    }

    #[test]
    fn test_config_without_models_is_backward_compatible() {
        let toml_content = r#"
[server]
host = "127.0.0.1"
port = 18789

[llm]
provider = "openai"
api_key = "test-key"
base_url = "https://api.openai.com/v1"
model = "gpt-4o"
max_tokens = 2048
temperature = 0.7
"#;

        let config: Config = toml::from_str(toml_content).unwrap();

        // models 应为空 HashMap
        assert!(config.models.is_empty());
        // default_model 应为 "default"
        assert_eq!(config.default_model, "default");
    }

    #[test]
    fn test_resolve_model_profile_default() {
        let config = Config::default();

        // "default" 应解析为 llm 段
        let resolved = config.resolve_model_profile("default").unwrap();
        assert_eq!(resolved.provider, "openai");
        assert_eq!(resolved.model, "gpt-4o");
        assert_eq!(resolved.max_tokens, 2048);
        assert_eq!(resolved.temperature, 0.7);
        // 元数据应为 None
        assert!(resolved.context_window.is_none());
        assert!(resolved.capability_tier.is_none());
    }

    #[test]
    fn test_resolve_model_profile_named() {
        let mut config = Config::default();
        config.llm.api_key = "sk-shared-key".to_string();
        config.llm.base_url = "https://api.openai.com/v1".to_string();

        // 添加一个命名的 model profile
        config.models.insert(
            "gpt-4o-mini".to_string(),
            ModelProfile {
                provider: "openai".to_string(),
                api_key: None,  // 继承
                base_url: None, // 继承
                model: "gpt-4o-mini".to_string(),
                context_window: Some(128000),
                cost_per_1k_input: Some(0.00015),
                cost_per_1k_output: Some(0.0006),
                capability_tier: Some("budget".to_string()),
                max_tokens: Some(1024),
                temperature: Some(0.3),
            },
        );

        let resolved = config.resolve_model_profile("gpt-4o-mini").unwrap();
        assert_eq!(resolved.provider, "openai");
        assert_eq!(resolved.model, "gpt-4o-mini");
        // api_key 从 llm 段继承
        assert_eq!(resolved.api_key, "sk-shared-key");
        // base_url 从 llm 段继承
        assert_eq!(resolved.base_url, "https://api.openai.com/v1");
        // profile 自己的值
        assert_eq!(resolved.max_tokens, 1024);
        assert_eq!(resolved.temperature, 0.3);
        assert_eq!(resolved.context_window, Some(128000));
        assert_eq!(resolved.capability_tier, Some("budget".to_string()));
    }

    #[test]
    fn test_resolve_model_profile_not_found() {
        let config = Config::default();

        let result = config.resolve_model_profile("nonexistent");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("nonexistent"));
        assert!(err.contains("not found"));
    }

    #[test]
    fn test_resolve_model_profile_custom_api_key_override() {
        let mut config = Config::default();
        config.llm.api_key = "sk-shared-key".to_string();

        config.models.insert(
            "claude".to_string(),
            ModelProfile {
                provider: "anthropic".to_string(),
                api_key: Some("sk-ant-exclusive".to_string()), // 覆盖
                base_url: Some("https://api.anthropic.com".to_string()), // 覆盖
                model: "claude-3-opus-20240229".to_string(),
                context_window: None,
                cost_per_1k_input: None,
                cost_per_1k_output: None,
                capability_tier: None,
                max_tokens: None,  // 继承 llm 段
                temperature: None, // 继承 llm 段
            },
        );

        let resolved = config.resolve_model_profile("claude").unwrap();
        // api_key 和 base_url 使用 profile 自己的
        assert_eq!(resolved.api_key, "sk-ant-exclusive");
        assert_eq!(resolved.base_url, "https://api.anthropic.com");
        // max_tokens 和 temperature 从 llm 段继承
        assert_eq!(resolved.max_tokens, 2048);
        assert_eq!(resolved.temperature, 0.7);
    }
}
