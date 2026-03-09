use crate::error::Result;
use crate::tools::ToolContext;
use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;

use std::path::{Component, Path};
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::Duration;

// ==================== 终端工具参数和结果类型 ====================

/// 运行命令参数
#[derive(Debug, Deserialize)]
pub struct RunCommandParams {
    /// 命令
    pub command: String,
    /// 参数列表
    #[serde(default)]
    pub args: Vec<String>,
    /// 工作目录（可选）
    pub cwd: Option<String>,
    /// 是否流式输出（可选）
    #[serde(default)]
    pub stream_output: bool,
}

/// 运行命令结果
#[derive(Debug, Serialize, Deserialize)]
pub struct RunCommandResult {
    /// 退出码
    pub exit_code: i32,
    /// 标准输出
    pub stdout: String,
    /// 标准错误输出
    pub stderr: String,
    /// 是否被截断
    pub truncated: bool,
}

// ==================== 终端工具实现 ====================

struct RunCommandTool;

impl RunCommandTool {
    pub fn new() -> Self {
        Self
    }

    /// 规范化路径，处理 . 和 .. 等路径遍历字符
    fn normalize_path(&self, raw: &str) -> String {
        let is_absolute = Path::new(raw).has_root();
        let mut components: Vec<&str> = Vec::new();

        for component in Path::new(raw).components() {
            match component {
                Component::CurDir => {}
                Component::ParentDir => {
                    if components.last() == Some(&"..") {
                        components.push("..");
                    } else if !components.is_empty() {
                        components.pop();
                    } else if !is_absolute {
                        components.push("..");
                    }
                }
                Component::Normal(segment) => {
                    if let Some(s) = segment.to_str() {
                        components.push(s);
                    }
                }
                Component::RootDir | Component::Prefix(_) => {}
            }
        }

        let joined = components.join("/");
        if is_absolute {
            format!("/{joined}")
        } else {
            joined
        }
    }

    /// 检查命令是否在黑名单中
    fn is_command_blacklisted(&self, command: &str, context: &ToolContext) -> bool {
        // 硬编码的黑名单
        let hardcoded_blacklist = ["rm -rf /", "mkfs", "dd if=", ":(){ :|:& };:"];

        // 检查硬编码黑名单
        if hardcoded_blacklist
            .iter()
            .any(|pattern| command.contains(pattern))
        {
            return true;
        }

        // 检查配置中的黑名单
        if context
            .config
            .local_tools
            .terminal
            .command_blacklist
            .iter()
            .any(|pattern| command.contains(pattern))
        {
            return true;
        }

        false
    }

    /// 检查工作目录是否被允许
    fn is_working_dir_allowed(&self, cwd: &str, context: &ToolContext) -> bool {
        // 如果没有配置允许的工作目录，则允许所有目录
        if context
            .config
            .local_tools
            .terminal
            .allowed_workspaces
            .is_empty()
        {
            return true;
        }

        // 规范化路径，检测路径遍历
        let normalized_cwd = self.normalize_path(cwd);

        // 同时检查原始路径和规范化路径，防止路径遍历攻击
        let _raw_allowed = context
            .config
            .local_tools
            .terminal
            .allowed_workspaces
            .iter()
            .any(|allowed_dir| cwd.starts_with(allowed_dir));

        // 如果原始路径或规范化路径有任何一个被允许，则允许
        // 更安全的做法是：只有当规范化路径被允许时才允许
        context
            .config
            .local_tools
            .terminal
            .allowed_workspaces
            .iter()
            .any(|allowed_dir| normalized_cwd.starts_with(allowed_dir))
    }

    /// 应用输出过滤规则
    fn apply_output_filters(&self, command: &str, output: &str, context: &ToolContext) -> String {
        // 检查是否有针对该命令的过滤规则
        if let Some(filters) = context.config.local_tools.terminal.filters.get(command) {
            let lines: Vec<&str> = output.lines().collect();
            let filtered_lines: Vec<&str> = lines
                .into_iter()
                .filter(|line| filters.iter().any(|filter| line.contains(filter)))
                .collect();
            return filtered_lines.join("\n");
        }

        output.to_string()
    }
}

#[async_trait]
impl crate::tools::LocalTool for RunCommandTool {
    fn name(&self) -> &str {
        "run_command"
    }

    fn description(&self) -> &str {
        "运行系统命令"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "要执行的命令"
                },
                "args": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "命令参数"
                },
                "cwd": {
                    "type": "string",
                    "description": "工作目录"
                },
                "stream_output": {
                    "type": "boolean",
                    "description": "是否流式输出"
                }
            },
            "required": ["command"],
            "additionalProperties": false
        })
    }

    async fn call(&self, arguments: Value, context: ToolContext) -> Result<Value> {
        let params: RunCommandParams = serde_json::from_value(arguments)?;

        // 检查命令是否被黑名单
        let full_command = if params.args.is_empty() {
            params.command.clone()
        } else {
            format!("{} {}", params.command, params.args.join(" "))
        };

        if self.is_command_blacklisted(&full_command, &context) {
            return Err(crate::error::Error::LocalToolExecution {
                tool: "run_command".to_string(),
                message: "Command is in blacklist".to_string(),
            });
        }

        // 检查工作目录是否被允许
        if let Some(cwd) = &params.cwd
            && !self.is_working_dir_allowed(cwd, &context)
        {
            return Err(crate::error::Error::LocalToolExecution {
                tool: "run_command".to_string(),
                message: format!("Working directory not allowed: {}", cwd),
            });
        }

        // 执行命令
        let mut command = Command::new(&params.command);
        if !params.args.is_empty() {
            command.args(&params.args);
        }
        if let Some(cwd) = &params.cwd {
            command.current_dir(cwd);
        }

        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let child = command.spawn()?;
        let child_id = child.id().expect("Failed to get child process id");

        let timeout = Duration::from_secs(context.config.local_tools.terminal.timeout_seconds);

        let output_result = tokio::select! {
            output = child.wait_with_output() => output,
            _ = tokio::time::sleep(timeout) => {
                // 超时，尝试终止进程
                if let Ok(mut cmd) = tokio::process::Command::new("taskkill")
                    .arg("/F")
                    .arg("/PID")
                    .arg(format!("{}", child_id))
                    .spawn()
                {
                    let _ = cmd.wait().await;
                }
                return Err(crate::error::Error::LocalToolExecution {
                    tool: "run_command".to_string(),
                    message: format!(
                        "Command timed out after {} seconds",
                        context.config.local_tools.terminal.timeout_seconds
                    ),
                });
            }
        };

        let output = match output_result {
            Ok(output) => output,
            Err(e) => {
                return Err(crate::error::Error::LocalToolExecution {
                    tool: "run_command".to_string(),
                    message: e.to_string(),
                });
            }
        };

        // 处理输出
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let max_output_bytes = context.config.local_tools.terminal.max_output_bytes;
        let mut truncated = false;

        let (processed_stdout, processed_stderr) = if stdout.len() + stderr.len() > max_output_bytes
        {
            truncated = true;
            let mut processed_stdout = stdout;
            let mut processed_stderr = stderr;

            // 截断输出以确保总长度不超过限制
            while processed_stdout.len() + processed_stderr.len() > max_output_bytes {
                if processed_stdout.len() > processed_stderr.len() {
                    processed_stdout.pop();
                } else {
                    processed_stderr.pop();
                }
            }

            (processed_stdout, processed_stderr)
        } else {
            (stdout, stderr)
        };

        // 应用输出过滤规则
        let filtered_stdout =
            self.apply_output_filters(&params.command, &processed_stdout, &context);

        let result = RunCommandResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: filtered_stdout,
            stderr: processed_stderr,
            truncated,
        };

        Ok(serde_json::to_value(result)?)
    }
}

// ==================== 终端工具管理器 ====================

pub struct TerminalTool;

impl Default for TerminalTool {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalTool {
    pub fn new() -> Self {
        Self
    }

    /// 注册所有终端工具
    pub fn register_all(registry: &mut crate::tools::registry::LocalToolRegistry) {
        registry.register(std::sync::Arc::new(RunCommandTool::new()));
    }
}

// ==================== 测试 ====================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::models::Session;
    use crate::tools::LocalTool;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_run_command() {
        let config = Arc::new(Config::default());
        let session = Session::new();
        let context = ToolContext::new(session, config);

        let tool = RunCommandTool::new();
        let args = json!({
            "command": "echo",
            "args": ["Hello", "World"],
            "cwd": ".",
            "stream_output": false
        });

        let result = tool.call(args, context).await.unwrap();
        let run_result: RunCommandResult = serde_json::from_value(result).unwrap();

        assert_eq!(run_result.exit_code, 0);
        assert!(run_result.stdout.contains("Hello World"));
        assert!(run_result.stderr.is_empty());
        assert!(!run_result.truncated);
    }
}
