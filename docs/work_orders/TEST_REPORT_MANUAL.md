# MineClaw 手动测试报告 (Manual Test Report)

## 1. 测试概览
*   **测试阶段**: Phase 2 验收测试
*   **测试日期**: 2026-03-05
*   **测试人员**: Cryptocho (Agent) & User
*   **测试环境**: Windows 11, PowerShell, Git Bash

## 2. 测试项目

### 2.1 基础服务启动
*   **操作**: 运行 `cargo run` 或执行编译后的 `mineclaw.exe`。
*   **结果**:
    *   ✅ 服务成功绑定 `127.0.0.1:18789`。
    *   ✅ 控制台输出 `Health check: http://127.0.0.1:18789/health`。
    *   ✅ 无 panic，无严重错误日志。

### 2.2 健康检查 API
*   **操作**: 使用 curl 访问 `/health`。
*   **命令**: `curl -v http://127.0.0.1:18789/health`
*   **结果**:
    *   ✅ 返回 HTTP 200 OK。
    *   ✅ 响应体为 `OK`。
    *   ✅ 解决了之前因 `0.0.0.0` 绑定导致的防火墙拦截问题。

### 2.3 Terminal MCP Server 集成
*   **操作**: 触发 LLM 调用终端工具。
*   **场景**:
    *   查询当前目录 (`ls` / `dir`)。
    *   查询 Shell 信息 (`get_shell_info`)。
*   **结果**:
    *   ✅ 主进程成功启动 `terminal_server.exe` 子进程。
    *   ✅ Stdio 通信正常，JSON-RPC 消息收发无误。
    *   ✅ 工具列表成功加载 (`execute_command`, `get_shell_info`)。
    *   ✅ 能够正确识别 Windows 环境并调用 `cmd.exe` 或 `powershell.exe`。

### 2.4 安全性测试
*   **操作**: 执行产生大量输出的命令。
*   **结果**:
    *   ✅ 输出超过 32KB 时自动截断。
    *   ✅ 返回结果包含 `[System Warning]` 提示。

## 3. 遗留问题与风险
*   **交互式命令**: 目前不支持 `vim`, `top` 等交互式程序，执行会挂起。已在文档中标记为已知限制。
*   **Shell 差异**: Git Bash 下的 curl 命令参数解析可能存在问题，建议使用 PowerShell 原生 curl 或标准 `curl.exe`。

## 4. 结论
系统核心功能运行稳定，关键路径（API -> LLM -> MCP -> Terminal）已打通。符合发布标准。
