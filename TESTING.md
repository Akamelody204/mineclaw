# MineClaw 测试指南 (Testing Guide)

本文档旨在为开发人员和测试人员提供 MineClaw 项目的全面测试指导，涵盖单元测试、集成测试、手动验证及自动化脚本使用方法。

## 1. 测试策略概览

MineClaw 采用分层测试策略，确保从底层逻辑到端到端功能的稳定性。

*   **单元测试 (Unit Tests)**: 针对 Rust 模块的独立功能测试（如加密模块、配置解析）。
*   **集成测试 (Integration Tests)**: 针对 API 端点和 MCP 协议交互的测试。
*   **端到端测试 (E2E Tests)**: 模拟真实用户场景，验证完整链路。
*   **手动验证 (Manual Verification)**: 针对特定环境（如不同 Shell）的兼容性检查。

---

## 2. 自动化测试 (Automated Tests)

### 2.1 运行 Rust 测试套件
项目使用标准的 `cargo test` 框架。

```bash
# 运行所有测试
cargo test

# 仅运行特定模块的测试（例如 security）
cargo test security

# 运行集成测试
cargo test --test mcp_integration
```

**关键测试模块：**
*   `src/security.rs`: 验证 AES-256-GCM 加密/解密逻辑。
*   `src/config.rs`: 验证配置文件加载与默认值。
*   `src/mcp/protocol.rs`: 验证 JSON-RPC 序列化/反序列化。

### 2.2 验证脚本 (PowerShell)
我们提供了一个一键验证脚本 `verify_mineclaw.ps1`，用于快速检查服务健康状态。

**前置条件：**
*   确保已编译 Release 版本：`cargo build --release`

**使用方法：**
在 PowerShell 中运行：
```powershell
./verify_mineclaw.ps1
```

**脚本功能：**
1.  启动 `target/release/mineclaw.exe` 后台进程。
2.  检查 `18789` 端口是否处于 `LISTENING` 状态。
3.  发送 HTTP GET 请求到 `/health` 端点。
4.  验证响应状态码是否为 200。
5.  测试完成后自动终止服务进程。

---

## 3. 手动测试流程 (Manual Testing)

### 3.1 启动服务
建议使用 `RUST_LOG` 环境变量启用详细日志，以便观察内部行为。

**PowerShell:**
```powershell
$env:RUST_LOG="debug"
cargo run --release
```

**Bash:**
```bash
RUST_LOG=debug cargo run --release
```

### 3.2 验证 Config API (热更新)
1.  **获取当前配置**：
    ```bash
    curl -v http://127.0.0.1:18789/api/config
    ```
    *预期结果*：返回 JSON 配置，敏感字段（如 `api_key`）应显示为 `******`。

2.  **更新配置**：
    ```bash
    curl -v -X POST http://127.0.0.1:18789/api/config \
      -H "Content-Type: application/json" \
      -d '{"llm": {"temperature": 0.8}}'
    ```
    *预期结果*：返回 HTTP 200，日志显示配置已更新并保存。

### 3.3 验证 Terminal MCP Server
此测试验证 LLM 是否能通过 MCP 协议调用本地 Shell。由于目前尚未集成前端 UI，主要通过观察服务端日志来验证。

1.  确保服务已启动且 Terminal Server 子进程已生成（日志显示 `MCP server process spawned successfully`）。
2.  观察日志中是否有 `Registering tool: execute_command`。
3.  （高级）使用 MCP 调试工具或手动构造 JSON-RPC 请求发送到 Stdin（需开发调试客户端）。

### 3.4 网络连通性测试
在某些受限网络环境（如 VPN、代理、防火墙）下，需特别关注端口绑定。

**测试命令：**
```bash
# 使用系统原生 curl (Windows)
C:\Windows\System32\curl.exe -v http://127.0.0.1:18789/health
```
*   如果返回 `Connection refused`：检查防火墙设置或 VPN 排除列表。
*   如果返回 `404 Not Found`：检查 URL 路径是否正确。

---

## 4. 常见问题排查 (Troubleshooting)

| 现象 | 可能原因 | 解决方案 |
| :--- | :--- | :--- |
| **端口绑定失败 (AddrInUse)** | 端口 18789 被占用 | 运行 `netstat -ano | findstr 18789` 查找 PID 并结束进程，或修改配置文件更换端口。 |
| **Config API 返回 401** | 未提供 Auth Token | (如已启用 Auth) 在 Header 中添加 `Authorization: Bearer <token>`。 |
| **Terminal 命令无输出** | 命令本身无输出或超时 | 检查日志，如果是交互式命令（如 vim），请避免使用。 |
| **乱码或颜色代码** | ANSI 转义序列 | 目前 Terminal Server 已做基础过滤，但部分工具仍可能输出控制字符，建议使用 `| cat` 或重定向到文件。 |

## 5. 提交前检查清单 (Pre-commit Checklist)
- [ ] `cargo test` 全部通过。
- [ ] `./verify_mineclaw.ps1` 运行成功。
- [ ] `cargo clippy` 无严重警告。
- [ ] `cargo fmt` 代码已格式化。
