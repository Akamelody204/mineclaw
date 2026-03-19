//! MCP stdio 传输层
//!
//! 负责通过 stdio 与 MCP 服务器进程通信。

use crate::error::{Error, Result};
use async_trait::async_trait;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tracing::{debug, error, info};

// ==================== Transport trait ====================

/// 传输层 trait，定义发送和接收消息的接口
#[async_trait]
pub trait Transport: Send + Sync {
    /// 发送一条 JSON 消息
    async fn send(&mut self, message: &str) -> Result<()>;

    /// 接收一条 JSON 消息
    async fn receive(&mut self) -> Result<String>;

    /// 关闭传输连接
    async fn close(&mut self) -> Result<()>;
}

// ==================== StdioTransport ====================

/// 基于 stdio 的传输层实现
pub struct StdioTransport {
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    stdout_lines: Option<tokio::io::Lines<BufReader<ChildStdout>>>,
}

impl StdioTransport {
    /// 创建一个新的 stdio 传输层并启动子进程
    pub async fn spawn(
        command: &str,
        args: &[String],
        env: &std::collections::HashMap<String, String>,
    ) -> Result<Self> {
        info!(command, ?args, "Spawning MCP server process");

        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        // 设置环境变量
        for (key, value) in env {
            cmd.env(key, value);
        }

        let mut child = cmd.spawn().map_err(|e| {
            error!(error = %e, "Failed to spawn MCP server");
            Error::Mcp(format!("Failed to spawn server: {}", e))
        })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| Error::Mcp("Failed to capture stdin".to_string()))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| Error::Mcp("Failed to capture stdout".to_string()))?;

        let stdout_lines = BufReader::new(stdout).lines();

        info!("MCP server process spawned successfully");

        Ok(Self {
            child: Some(child),
            stdin: Some(stdin),
            stdout_lines: Some(stdout_lines),
        })
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn send(&mut self, message: &str) -> Result<()> {
        debug!(message, "Sending message to MCP server");

        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| Error::Mcp("Transport is closed".to_string()))?;

        stdin.write_all(message.as_bytes()).await.map_err(|e| {
            error!(error = %e, "Failed to write to stdin");
            Error::Mcp(format!("Write failed: {}", e))
        })?;

        stdin.write_all(b"\n").await.map_err(|e| {
            error!(error = %e, "Failed to write newline");
            Error::Mcp(format!("Write failed: {}", e))
        })?;

        stdin.flush().await.map_err(|e| {
            error!(error = %e, "Failed to flush stdin");
            Error::Mcp(format!("Flush failed: {}", e))
        })?;

        debug!("Message sent successfully");
        Ok(())
    }

    async fn receive(&mut self) -> Result<String> {
        let lines = self
            .stdout_lines
            .as_mut()
            .ok_or_else(|| Error::Mcp("Transport is closed".to_string()))?;

        debug!("Waiting for message from MCP server");

        match lines.next_line().await {
            Ok(Some(line)) => {
                if line.trim().is_empty() {
                    debug!("Received empty line, skipping");
                    self.receive().await
                } else {
                    debug!(line, "Received message from MCP server");
                    Ok(line)
                }
            }
            Ok(None) => {
                error!("MCP server stdout closed");
                Err(Error::Mcp("Server closed connection".to_string()))
            }
            Err(e) => {
                error!(error = %e, "Failed to read from stdout");
                Err(Error::Mcp(format!("Read failed: {}", e)))
            }
        }
    }

    async fn close(&mut self) -> Result<()> {
        info!("Closing MCP server transport");

        // Drop stdin first to signal EOF
        self.stdin.take();
        self.stdout_lines.take();

        if let Some(mut child) = self.child.take() {
            // Try graceful shutdown first
            match child.try_wait() {
                Ok(Some(status)) => {
                    info!(%status, "MCP server already exited");
                }
                Ok(None) => {
                    // Send SIGTERM (Unix) or terminate (Windows)
                    #[cfg(unix)]
                    {
                        use tokio::signal::unix::{SignalKind, signal};
                        if let Ok(mut sigterm) = signal(SignalKind::terminate()) {
                            let _ = child.start_kill();
                            tokio::select! {
                                _ = sigterm.recv() => {},
                                _ = tokio::time::sleep(tokio::time::Duration::from_secs(5)) => {
                                    let _ = child.kill().await;
                                }
                            }
                        }
                    }
                    #[cfg(not(unix))]
                    {
                        let _ = child.kill().await;
                    }

                    let status = child.wait().await;
                    info!(?status, "MCP server exited");
                }
                Err(e) => {
                    error!(error = %e, "Failed to wait for MCP server");
                    let _ = child.kill().await;
                }
            }
        }

        Ok(())
    }
}

impl Drop for StdioTransport {
    fn drop(&mut self) {
        // 如果还没关闭，尝试清理
        if self.child.is_some() {
            // 因为 drop 不是 async，我们只能尽力而为
            if let Some(mut child) = self.child.take() {
                let _ = child.start_kill();
            }
        }
    }
}
