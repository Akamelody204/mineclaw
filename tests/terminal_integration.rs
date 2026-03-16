//! 终端工具集成测试
//!
//! 测试终端工具与系统其他组件的集成

use mineclaw::config::{Config, LocalToolsConfig, TerminalConfig};
use mineclaw::models::Session;
use mineclaw::tools::{LocalToolRegistry, ToolContext, terminal::TerminalTool};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_terminal_tool_registration() {
    // 测试终端工具是否能正确注册到工具注册表
    let mut registry = LocalToolRegistry::new();

    // 注册终端工具
    TerminalTool::register_all(&mut registry);

    // 验证工具是否存在于注册表中
    assert!(registry.has_tool("run_command"));

    // 获取工具定义
    let tool = registry.get_tool("run_command");
    assert!(tool.is_some());

    let tool = tool.unwrap();
    assert_eq!(tool.name, "run_command");
    assert_eq!(
        tool.description,
        "Execute a terminal command and return the output. Restricted to safe commands and specific workspaces."
    );
}

#[tokio::test]
async fn test_terminal_tool_basic_execution() {
    // 测试基本的命令执行功能
    let mut registry = LocalToolRegistry::new();
    TerminalTool::register_all(&mut registry);

    let config = Arc::new(Config::default());
    let session = Session::new();
    let context = ToolContext::new(session, config);

    // 准备测试参数
    let args = json!({
        "command": "echo",
        "args": ["Hello", "from", "integration", "test"],
        "cwd": ".",
        "stream_output": false
    });

    // 调用工具
    let result = registry.call_tool("run_command", args, context).await;

    // 验证结果
    assert!(result.is_ok());

    let result_value = result.unwrap();
    let exit_code = result_value["exit_code"].as_i64();
    let stdout = result_value["stdout"].as_str();

    assert_eq!(exit_code, Some(0));
    assert!(stdout.is_some());
    assert!(stdout.unwrap().contains("Hello from integration test"));
}

#[tokio::test]
async fn test_terminal_tool_config_loading() {
    // 测试终端工具配置是否能正确加载
    let mut config = Config::default();

    // 修改一些配置值
    config.local_tools.terminal.max_output_bytes = 2048;
    config.local_tools.terminal.timeout_seconds = 10;
    config.local_tools.terminal.command_blacklist = vec!["test_cmd".to_string()];

    // 验证配置值
    assert_eq!(config.local_tools.terminal.max_output_bytes, 2048);
    assert_eq!(config.local_tools.terminal.timeout_seconds, 10);
    assert_eq!(config.local_tools.terminal.command_blacklist.len(), 1);
    assert!(
        config
            .local_tools
            .terminal
            .command_blacklist
            .contains(&"test_cmd".to_string())
    );
}

#[tokio::test]
async fn test_terminal_tool_with_arguments() {
    // 测试带参数的命令执行
    let mut registry = LocalToolRegistry::new();
    TerminalTool::register_all(&mut registry);

    let config = Arc::new(Config::default());
    let session = Session::new();
    let context = ToolContext::new(session, config);

    // 准备带参数的测试
    let args = json!({
        "command": "echo",
        "args": ["Argument", "1", "Argument", "2"],
        "stream_output": false
    });

    // 调用工具
    let result = registry.call_tool("run_command", args, context).await;

    // 验证结果
    assert!(result.is_ok());

    let result_value = result.unwrap();
    let stdout = result_value["stdout"].as_str().unwrap();

    assert!(stdout.contains("Argument 1"));
    assert!(stdout.contains("Argument 2"));
}

#[tokio::test]
async fn test_terminal_tool_blacklist() {
    // 测试命令黑名单功能
    let mut registry = LocalToolRegistry::new();
    TerminalTool::register_all(&mut registry);

    // 创建带有黑名单的配置
    let mut config = Config::default();
    config.local_tools.terminal.command_blacklist = vec!["echo".to_string()];
    let config_arc = Arc::new(config);

    let session = Session::new();
    let context = ToolContext::new(session, config_arc);

    // 准备测试参数（使用黑名单中的命令）
    let args = json!({
        "command": "echo",
        "args": ["This should be blocked"],
        "stream_output": false
    });

    // 调用工具 - 应该被拒绝
    let result = registry.call_tool("run_command", args, context).await;

    // 验证结果 - 应该返回错误
    assert!(result.is_err());
}

#[tokio::test]
async fn test_terminal_tool_default_config() {
    // 测试默认配置是否正确
    let terminal_config = TerminalConfig::default();

    // 验证默认值
    assert!(terminal_config.enabled);
    assert_eq!(terminal_config.max_output_bytes, 1048576); // 1MB
    assert_eq!(terminal_config.timeout_seconds, 300); // 5分钟
    assert!(terminal_config.allowed_workspaces.is_empty());
    assert!(terminal_config.command_blacklist.is_empty());
    assert!(terminal_config.filters.is_empty());

    // 验证 LocalToolsConfig 默认值
    let local_tools_config = LocalToolsConfig::default();
    assert!(local_tools_config.terminal.enabled);
}

#[tokio::test]
async fn test_terminal_tool_list_tools() {
    // 测试工具列表功能
    let mut registry = LocalToolRegistry::new();

    // 注册前应该是空的
    let tools_before = registry.list_tools();
    assert!(!tools_before.iter().any(|t| t.name == "run_command"));

    // 注册终端工具
    TerminalTool::register_all(&mut registry);

    // 注册后应该包含终端工具
    let tools_after = registry.list_tools();
    assert!(tools_after.iter().any(|t| t.name == "run_command"));

    // 验证工具信息
    let terminal_tool = tools_after
        .iter()
        .find(|t| t.name == "run_command")
        .unwrap();
    assert_eq!(terminal_tool.name, "run_command");
    assert_eq!(
        terminal_tool.description,
        "Execute a terminal command and return the output. Restricted to safe commands and specific workspaces."
    );
}
