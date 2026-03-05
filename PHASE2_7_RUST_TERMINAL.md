# MineClaw Phase 2.7 - Rust Terminal Server Implementation

## 概述
Phase 2.7 专注于将 MineClaw 的核心 Terminal MCP Server 从 Node.js/TypeScript 原型重构为高性能、无依赖的 Rust 实现。这一举措消除了对外部 Node.js 运行时的依赖，提高了系统的便携性和执行效率，并增强了安全性。

## 主要变更

### 1. 核心架构重构
*   **移除 Node.js 依赖**：完全移除了 `terminal-mcp-server.js` 及其相关 `package.json` 依赖。
*   **Rust 原生实现**：新建 `src/bin/terminal_server.rs`，作为一个独立的二进制程序编译和运行。
*   **MCP 协议集成**：
    *   实现了标准的 JSON-RPC 2.0 协议。
    *   基于 `stdio` 进行进程间通信。
    *   内置 `execute_command` 和 `get_shell_info` 两个核心工具。

### 2. 功能增强与安全
*   **跨平台 Shell 支持**：
    *   自动检测宿主机操作系统（Windows/Linux/macOS）。
    *   智能选择 Shell 执行器（cmd.exe, PowerShell, Bash）。
    *   支持用户自定义 Shell 路径。
*   **输出截断保护**：
    *   为防止海量日志输出导致内存溢出或上下文超限，实现了 **32KB 输出截断**。
    *   当 `stdout` 或 `stderr` 超过限制时，自动截断并追加系统提示，引导 AI 使用 `head`/`grep` 等工具。
*   **网络与绑定修复**：
    *   修复了 Windows 环境下 `0.0.0.0` 绑定被防火墙拦截的问题，统一强制绑定到 `127.0.0.1`。
    *   增加了详细的启动与绑定日志，便于排查“静默失败”问题。

### 3. 验证与测试
*   **自动化测试脚本**：
    *   创建了 `verify_mineclaw.ps1` PowerShell 脚本。
    *   自动化流程：启动服务 -> 检测端口监听 -> 发送健康检查请求 -> 验证响应 -> 自动清理进程。
*   **手动验证**：
    *   验证了 `curl` 命令在不同 Shell 环境下的兼容性问题。
    *   确认了 `netstat` 端口占用情况。

## 项目结构更新
```text
mineclaw/
├── src/
│   ├── bin/
│   │   └── terminal_server.rs  # [NEW] Rust 实现的 Terminal Server
│   └── main.rs                 # 更新 MCP 配置以调用新二进制
├── verify_mineclaw.ps1         # [NEW] 自动化验证脚本
├── .gitignore                  # [UPDATE] 忽略临时脚本与配置
└── terminal-mcp-server.js      # [DEPRECATED] 待移除的旧 JS 实现
```

## 下一步计划 (Phase 3)
*   **遗留代码清理**：彻底删除旧的 JS/TS 代码和相关配置文件。
*   **交互式会话支持**：探索基于 PTY 的交互式命令执行（如支持 `top`, `vim` 等 TUI 应用）。
*   **更细粒度的权限控制**：为不同工具设置执行权限白名单。
