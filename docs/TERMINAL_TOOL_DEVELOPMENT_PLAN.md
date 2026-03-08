# 终端工具开发计划

## 1. 项目概述

### 1.1 项目背景
本项目需要实现一个终端工具，提供命令执行功能，支持输出限制、自定义过滤规则、命令黑名单机制、工作目录限制、超时控制等功能。

### 1.2 参考实现
本项目参考 Zed 编辑器的终端命令执行工具核心代码（位于 `D:\mineclaw\参考\terminal_command_execution_tool\` 目录下），包括：
- 命令执行流程
- 超时控制
- 工作目录校验
- 命令黑白名单
- TOML 配置解析
- 输出大小限制

### 1.3 技术栈
- **语言**: Rust
- **异步运行时**: Tokio
- **配置解析**: Serde + Toml
- **其他依赖**: tracing, thiserror

## 2. 功能需求

### 2.1 核心功能
- [x] 命令执行：支持执行任意系统命令
- [x] 输出限制：配置最大输出字节数，超过则截断
- [x] 超时控制：配置执行超时时间，超时则终止命令
- [x] 工作目录限制：配置允许的工作目录，防止越权访问
- [x] 命令黑名单：配置禁止执行的命令，防止恶意操作
- [x] 过滤规则：配置输出过滤规则，仅显示感兴趣的信息

### 2.2 配置选项
```toml
# config/mineclaw.toml
[server]
host = "127.0.0.1"
port = 18789

[llm]
provider = "volcengine"
api_key = "encrypted:tkv5uXC5iiqmjGMwWyCyWjdvVjA31ubmiLQjE1u5PRsc6Lcw6HAHWqEViCyR"
base_url = "https://ark.cn-beijing.volces.com/api/coding/v3"
model = "ark-code-latest"
max_tokens = 256000
temperature = 0.7

[mcp]
enabled = false

[local_tools]
[local_tools.terminal]
max_output_bytes = 65536
timeout_seconds = 300
allowed_workspaces = []
command_blacklist = [
    "rm", "del", "format", "notepad", "calc", "shutdown", "reboot"
]

[local_tools.terminal.filters]
```

### 2.3 错误处理
- 命令执行失败
- 命令超时
- 命令被黑名单拒绝
- 工作目录不被允许
- 输出截断

## 3. 技术方案

### 3.1 项目结构

```
src/
├── config.rs          # 配置解析和管理
├── error.rs           # 错误类型定义
├── main.rs            # 服务器入口点
├── tools/
│   ├── mod.rs         # 工具模块定义
│   ├── registry.rs    # 工具注册表
│   ├── terminal.rs    # 终端工具实现
│   └── ...            # 其他工具
```

### 3.2 核心数据结构

```rust
// src/config.rs
#[derive(Debug, Deserialize, Clone)]
pub struct TerminalConfig {
    pub enabled: bool,
    pub max_output_bytes: usize,
    pub timeout_seconds: u64,
    pub allowed_workspaces: Vec<String>,
    pub command_blacklist: Vec<String>,
    pub filters: HashMap<String, Vec<String>>,
}

// src/tools/terminal.rs
#[derive(Debug, Deserialize)]
pub struct RunCommandParams {
    pub command: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
    pub stream_output: bool,
}

#[derive(Debug, Serialize)]
pub struct RunCommandResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub truncated: bool,
}
```

### 3.3 执行流程

```
RunCommandTool.call()
    -> 检查命令是否在黑名单
    -> 检查工作目录是否被允许
    -> 创建子进程执行命令
    -> 异步等待命令完成或超时
    -> 处理命令输出（截断、过滤）
    -> 返回结果
```

## 4. 实现步骤

### 4.1 配置解析（已完成）

**任务**: 实现 `TerminalConfig` 结构体和配置解析逻辑

**文件**: `src/config.rs`

**主要变更**:
1. 添加 `TerminalConfig` 结构体
2. 添加默认配置值
3. 添加配置文件解析支持

### 4.2 终端工具实现（已完成）

**任务**: 实现 `RunCommandTool` 结构体和命令执行逻辑

**文件**: `src/tools/terminal.rs`

**主要变更**:
1. 添加 `RunCommandParams` 和 `RunCommandResult` 数据结构
2. 实现命令执行核心逻辑
3. 添加黑名单检查功能
4. 添加工作目录验证功能
5. 添加输出截断和过滤功能
6. 添加测试用例

### 4.3 工具注册（已完成）

**任务**: 实现终端工具的注册和初始化逻辑

**文件**: `src/tools/mod.rs` 和 `src/main.rs`

**主要变更**:
1. 在 `mod.rs` 中添加终端工具引用
2. 在 `main.rs` 中添加终端工具注册代码

### 4.4 测试（已完成）

**任务**: 实现终端工具的测试用例

**文件**: `src/tools/terminal.rs`

**主要变更**:
1. 添加 `test_run_command` 测试用例
2. 测试基本的命令执行功能

## 5. 测试计划

### 5.1 功能测试

**测试命令**:
```bash
cargo run -- --config config/mineclaw.toml
```

**测试场景**:
1. 执行简单命令（如 `echo "Hello World"`）
2. 执行带参数的命令（如 `ls -la`）
3. 执行超时命令（如 `sleep 10` 当超时设置为 5 秒）
4. 执行黑名单命令（如 `rm -rf /`）
5. 执行在不允许的工作目录中的命令
6. 执行产生大量输出的命令（如 `cat /dev/random`）
7. 执行符合过滤规则的命令（如 `cargo build`）

### 5.2 性能测试

**测试命令**:
```bash
cargo run --release -- --config config/mineclaw.toml
```

**测试场景**:
1. 同时执行多个命令
2. 执行长时间运行的命令
3. 执行产生大量输出的命令

## 6. 风险管理

### 6.1 安全风险

- **命令注入**: 确保命令和参数被正确解析，防止注入攻击
- **权限提升**: 限制命令的执行权限，防止权限提升
- **资源消耗**: 限制命令的执行时间和输出大小，防止资源耗尽

### 6.2 技术风险

- **平台兼容性**: 确保命令执行在 Windows、Linux 和 macOS 上都能正常工作
- **异步处理**: 确保异步执行逻辑正确，防止资源泄漏
- **错误处理**: 确保所有错误场景都能正确处理

### 6.3 实施风险

- **配置文件**: 确保配置文件格式正确，防止解析错误
- **依赖更新**: 确保依赖库的版本兼容，防止兼容性问题

## 7. 后续扩展

### 7.1 功能扩展

- 支持命令输出流式传输
- 支持命令历史记录
- 支持命令别名
- 支持命令完成

### 7.2 性能优化

- 优化命令执行效率
- 优化输出处理效率
- 优化异步处理

### 7.3 安全增强

- 支持命令白名单
- 支持用户权限管理
- 支持命令审计

## 8. 总结

本计划详细描述了终端工具的功能需求、技术方案和实现步骤。我们将参考 Zed 编辑器的终端命令执行工具核心代码，实现一个功能强大、安全可靠的终端工具，支持命令执行、输出限制、超时控制、工作目录限制、命令黑名单和过滤规则等功能。

我们将遵循软件工程最佳实践，确保代码质量和可维护性，并通过全面的测试覆盖确保工具的正确性和稳定性。