//! 终端工具功能测试
//!
//! 测试终端工具的各种功能场景

use mineclaw::config::Config;
use mineclaw::models::Session;
use mineclaw::tools::{LocalToolRegistry, ToolContext, terminal::TerminalTool};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_functional_simple_command() {
    // TC-FUNC-001: 简单命令执行测试
    let mut registry = LocalToolRegistry::new();
    TerminalTool::register_all(&mut registry);

    let config = Arc::new(Config::default());
    let session = Session::new();
    let context = ToolContext::new(session, config);

    // 准备测试参数
    let args = json!({
        "command": "echo",
        "args": ["Hello", "World"],
        "cwd": ".",
        "stream_output": false
    });

    // 调用工具
    let result = registry.call_tool("run_command", args, context).await;

    // 验证结果
    assert!(result.is_ok());
    let result_value = result.unwrap();
    assert_eq!(result_value["exit_code"].as_i64(), Some(0));
    assert!(
        result_value["stdout"]
            .as_str()
            .unwrap()
            .contains("Hello World")
    );
    assert_eq!(result_value["stderr"].as_str().unwrap(), "");
    assert_eq!(result_value["truncated"].as_bool(), Some(false));
}

#[tokio::test]
async fn test_functional_command_with_arguments() {
    // TC-FUNC-002: 带参数的命令执行测试
    let mut registry = LocalToolRegistry::new();
    TerminalTool::register_all(&mut registry);

    let config = Arc::new(Config::default());
    let session = Session::new();
    let context = ToolContext::new(session, config);

    // 准备带多个参数的测试
    let args = json!({
        "command": "echo",
        "args": ["Argument", "1", "Argument", "2", "Argument", "3"],
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
    assert!(stdout.contains("Argument 3"));
}

#[tokio::test]
async fn test_functional_output_truncation() {
    // TC-FUNC-003: 输出限制测试
    let mut registry = LocalToolRegistry::new();
    TerminalTool::register_all(&mut registry);

    // 创建较小输出限制的配置
    let mut config = Config::default();
    config.local_tools.terminal.max_output_bytes = 20; // 很小的限制
    let config_arc = Arc::new(config);

    let session = Session::new();
    let context = ToolContext::new(session, config_arc);

    // 准备会产生大量输出的测试
    let args = json!({
        "command": "echo",
        "args": ["This is a very long output that should be truncated"],
        "stream_output": false
    });

    // 调用工具
    let result = registry.call_tool("run_command", args, context).await;

    // 验证结果 - 输出应该被截断
    assert!(result.is_ok());
    let result_value = result.unwrap();
    assert_eq!(result_value["truncated"].as_bool(), Some(true));

    // 验证输出长度不超过限制
    let stdout = result_value["stdout"].as_str().unwrap();
    let stderr = result_value["stderr"].as_str().unwrap();
    assert!(stdout.len() + stderr.len() <= 20);
}

#[tokio::test]
async fn test_functional_command_blacklist() {
    // TC-FUNC-006: 命令黑名单测试
    let mut registry = LocalToolRegistry::new();
    TerminalTool::register_all(&mut registry);

    // 创建带有黑名单的配置
    let mut config = Config::default();
    config.local_tools.terminal.command_blacklist = vec!["rm".to_string(), "del".to_string()];
    let config_arc = Arc::new(config);

    let session = Session::new();
    let context = ToolContext::new(session, config_arc);

    // 测试1: 使用黑名单中的命令
    let args1 = json!({
        "command": "rm",
        "args": ["-rf", "/"],
        "stream_output": false
    });

    let result1 = registry
        .call_tool("run_command", args1, context.clone())
        .await;
    assert!(result1.is_err()); // 应该被拒绝

    // 测试2: 使用不在黑名单中的命令
    let args2 = json!({
        "command": "echo",
        "args": ["This should work"],
        "stream_output": false
    });

    let result2 = registry.call_tool("run_command", args2, context).await;
    assert!(result2.is_ok()); // 应该成功
}

#[tokio::test]
async fn test_functional_working_directory_restriction() {
    // TC-FUNC-005: 工作目录限制测试
    let mut registry = LocalToolRegistry::new();
    TerminalTool::register_all(&mut registry);

    // 创建带有工作目录限制的配置
    let mut config = Config::default();
    config.local_tools.terminal.allowed_workspaces = vec!["./test".to_string()];
    let config_arc = Arc::new(config);

    let session = Session::new();
    let context = ToolContext::new(session, config_arc);

    // 测试1: 使用不在允许列表中的工作目录
    let args1 = json!({
        "command": "echo",
        "args": ["Hello"],
        "cwd": "./forbidden",
        "stream_output": false
    });

    let result1 = registry
        .call_tool("run_command", args1, context.clone())
        .await;
    assert!(result1.is_err()); // 应该被拒绝

    // 测试2: 如果没有配置限制，应该允许所有目录
    let mut registry2 = LocalToolRegistry::new();
    TerminalTool::register_all(&mut registry2);

    let mut config2 = Config::default();
    config2.local_tools.terminal.allowed_workspaces = vec![]; // 空列表，允许所有目录
    let config_arc2 = Arc::new(config2);
    let session2 = Session::new();
    let context2 = ToolContext::new(session2, config_arc2);

    let args2 = json!({
        "command": "echo",
        "args": ["Hello"],
        "cwd": ".",
        "stream_output": false
    });

    let result2 = registry2.call_tool("run_command", args2, context2).await;
    assert!(result2.is_ok()); // 应该成功
}

#[tokio::test]
async fn test_functional_output_filters() {
    // TC-FUNC-007: 输出过滤测试
    let mut registry = LocalToolRegistry::new();
    TerminalTool::register_all(&mut registry);

    // 创建带有过滤规则的配置
    let mut config = Config::default();
    config.local_tools.terminal.filters.insert(
        "echo".to_string(),
        vec!["Hello".to_string()], // 只显示包含Hello的行
    );
    let config_arc = Arc::new(config);

    let session = Session::new();
    let context = ToolContext::new(session, config_arc);

    // 准备测试
    let args = json!({
        "command": "echo",
        "args": ["Hello World"],
        "stream_output": false
    });

    // 调用工具
    let result = registry.call_tool("run_command", args, context).await;

    // 验证结果
    assert!(result.is_ok());
    let result_value = result.unwrap();
    let stdout = result_value["stdout"].as_str().unwrap();

    // 由于我们的过滤规则是匹配"Hello"，输出应该包含Hello
    assert!(stdout.contains("Hello"));
}

#[tokio::test]
async fn test_functional_hardcoded_blacklist() {
    // 测试硬编码黑名单
    let mut registry = LocalToolRegistry::new();
    TerminalTool::register_all(&mut registry);

    let config = Arc::new(Config::default());
    let session = Session::new();
    let context = ToolContext::new(session, config);

    // 测试硬编码黑名单中的命令
    let args = json!({
        "command": "rm",
        "args": ["-rf", "/"],
        "stream_output": false
    });

    let result = registry.call_tool("run_command", args, context).await;
    assert!(result.is_err()); // 应该被硬编码黑名单拒绝
}

#[tokio::test]
async fn test_functional_command_without_arguments() {
    // 测试不带参数的命令
    let mut registry = LocalToolRegistry::new();
    TerminalTool::register_all(&mut registry);

    let config = Arc::new(Config::default());
    let session = Session::new();
    let context = ToolContext::new(session, config);

    // 准备不带参数的测试
    let args = json!({
        "command": "echo",
        "stream_output": false
    });

    // 调用工具
    let result = registry.call_tool("run_command", args, context).await;

    // 验证结果
    assert!(result.is_ok());
    let result_value = result.unwrap();
    assert_eq!(result_value["exit_code"].as_i64(), Some(0));
}

#[tokio::test]
async fn test_functional_empty_command() {
    // 测试空命令参数
    let mut registry = LocalToolRegistry::new();
    TerminalTool::register_all(&mut registry);

    let config = Arc::new(Config::default());
    let session = Session::new();
    let context = ToolContext::new(session, config);

    // 测试缺少必需参数
    let args = json!({}); // 没有command参数

    // 调用工具
    let result = registry.call_tool("run_command", args, context).await;

    // 应该返回错误，因为缺少必需的command参数
    assert!(result.is_err());
}
