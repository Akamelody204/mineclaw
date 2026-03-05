# MineClaw Phase 3 实现计划

## 概述

Phase 3: 本地工具集与安全增强。在这一阶段，我们将实现核心的本地工具集，包括终端工具、文件工具，并集成 agentfs 实现 checkpoint 功能，同时增强 API Key 的安全性。

---

## 目标

- ✅ API Key 加密存储
- ✅ 终端工具（带输出限制和过滤）
- ✅ 文件读写工具（仅指定的 9 个工具）
- ✅ agentfs checkpoint 集成
- ✅ 命令黑名单（而非白名单）

---

## 实现功能

### 1. API Key 加密存储
- **功能描述**: 将配置中的 API Key 使用加密方式存储，避免明文泄露
- **实现细节**:
  - 使用 AES-GCM 256 位加密
  - 加密密钥通过环境变量提供（`MINECLAW_ENCRYPTION_KEY`）
  - 支持加密/解密配置文件中的敏感字段
  - 提供密钥生成工具
  - 配置文件中标记为 `encrypted_` 前缀的字段会被自动解密

- **数据结构**:
  ```rust
  // src/encryption.rs
  pub struct EncryptionManager {
      key: [u8; 32],
  }

  impl EncryptionManager {
      pub fn new(key: &str) -> Result<Self>;
      pub fn encrypt(&self, plaintext: &str) -> Result<String>;
      pub fn decrypt(&self, ciphertext: &str) -> Result<String>;
      pub fn generate_key() -> String;
  }
  ```

- **配置扩展**:
  ```toml
  # config/mineclaw.toml
  [llm]
  api_key = "encrypted:base64encodedencrypteddata"
  ```

### 2. 终端工具
- **功能描述**: 提供命令执行工具，支持输出限制和自定义过滤规则
- **工具名称**: `run_command`
- **实现细节**:
  - 最大输出文本限制（默认 64KB，可配置）
  - 支持用户自定义过滤规则（配置文件）
  - 命令黑名单机制（而非白名单）
  - 工作目录限制
  - 超时控制（默认 300 秒）
  - 支持实时输出流式传输（通过 SSE）

- **内置黑名单**:
  - `rm -rf /`
  - `mkfs`
  - `dd if=`
  - `:(){ :|:& };:`
  - 等危险命令

- **过滤规则配置**:
  ```toml
  # config/mineclaw.toml
  [terminal]
  max_output_bytes = 65536  # 64KB
  timeout_seconds = 300
  allowed_workspaces = ["/path/to/workspace"]

  [terminal.filters]
  # Cargo 过滤规则
  "cargo build" = [
      "^\\s*Compiling",
      "^\\s*Building",
      "^\\s*Finished",
      "^\\s*Running",
  ]
  "cargo run" = [
      "^\\s*Compiling",
      "^\\s*Building",
      "^\\s*Finished",
  ]

  # Pip 过滤规则
  "pip install" = [
      "^Collecting",
      "^Downloading",
      "^\\s+\\d+%",
      "^Installing collected packages",
  ]
  ```

- **工具定义**:
  ```rust
  // src/tools/terminal.rs
  pub struct TerminalTool {
      config: TerminalConfig,
  }

  #[derive(Debug, Deserialize, Clone)]
  pub struct TerminalConfig {
      pub max_output_bytes: usize,
      pub timeout_seconds: u64,
      pub allowed_workspaces: Vec<String>,
      pub command_blacklist: Vec<String>,
      pub filters: HashMap<String, Vec<String>>,
  }

  pub struct RunCommandParams {
      pub command: String,
      pub args: Vec<String>,
      pub cwd: Option<String>,
      pub stream_output: Option<bool>,
  }

  pub struct RunCommandResult {
      pub exit_code: i32,
      pub stdout: String,
      pub stderr: String,
      pub truncated: bool,
  }
  ```

### 3. 文件工具集
- **功能描述**: 提供 9 个指定的文件操作工具
- **实现细节**:
  - 所有读取操作有最大文本限制（默认 16KB）
  - 工作目录限制（基于配置）
  - 路径遍历防护（`..` 检查）
  - 基于 agentfs 的 checkpoint 集成

- **工具列表**:

  1. **read_file** - 读取完整文件内容
     ```rust
     pub struct ReadFileParams {
         pub path: String,
     }
     pub struct ReadFileResult {
         pub content: String,
         pub truncated: bool,
         pub total_bytes: usize,
     }
     ```

  2. **write_file** - 创建新文件或完全覆盖现有文件
     ```rust
     pub struct WriteFileParams {
         pub path: String,
         pub content: String,
     }
     pub struct WriteFileResult {
         pub success: bool,
         pub bytes_written: usize,
     }
     ```

  3. **list_directory** - 列出目录内容
     ```rust
     pub struct ListDirectoryParams {
         pub path: String,
         pub recursive: Option<bool>,
     }
     pub struct DirectoryEntry {
         pub name: String,
         pub path: String,
         pub is_dir: bool,
         pub size: Option<u64>,
         pub modified: Option<DateTime<Utc>>,
     }
     pub struct ListDirectoryResult {
         pub entries: Vec<DirectoryEntry>,
     }
     ```

  4. **search_file** - 在文件中搜索特定文本模式
     ```rust
     pub struct SearchFileParams {
         pub path: String,
         pub pattern: String,
         pub case_sensitive: Option<bool>,
     }
     pub struct SearchMatch {
         pub line_number: usize,
         pub line_content: String,
         pub start_column: usize,
         pub end_column: usize,
     }
     pub struct SearchFileResult {
         pub matches: Vec<SearchMatch>,
         pub total_matches: usize,
     }
     ```

  5. **move_file** - 移动或重命名文件
     ```rust
     pub struct MoveFileParams {
         pub source: String,
         pub destination: String,
         pub overwrite: Option<bool>,
     }
     pub struct MoveFileResult {
         pub success: bool,
     }
     ```

  6. **move_directory** - 移动或重命名目录
     ```rust
     pub struct MoveDirectoryParams {
         pub source: String,
         pub destination: String,
         pub overwrite: Option<bool>,
     }
     pub struct MoveDirectoryResult {
         pub success: bool,
     }
     ```

  7. **delete_file** - 删除文件
     ```rust
     pub struct DeleteFileParams {
         pub path: String,
     }
     pub struct DeleteFileResult {
         pub success: bool,
     }
     ```

  8. **delete_directory** - 删除目录
     ```rust
     pub struct DeleteDirectoryParams {
         pub path: String,
         pub recursive: Option<bool>,
     }
     pub struct DeleteDirectoryResult {
         pub success: bool,
     }
     ```

  9. **create_directory** - 创建新目录
     ```rust
     pub struct CreateDirectoryParams {
         pub path: String,
         pub parents: Option<bool>,
     }
     pub struct CreateDirectoryResult {
         pub success: bool,
     }
     ```

  10. **search_and_replace** - 精准替换文件中的特定内容
      ```rust
      pub struct SearchAndReplaceParams {
          pub path: String,
          pub search: String,
          pub replace: String,
          pub case_sensitive: Option<bool>,
      }
      pub struct SearchAndReplaceResult {
          pub success: bool,
          pub replacements_made: usize,
      }
      ```

- **文件工具配置**:
  ```toml
  # config/mineclaw.toml
  [filesystem]
  max_read_bytes = 16384  # 16KB
  allowed_directories = [
      "/path/to/workspace",
      "/another/allowed/path"
  ]
  enable_checkpoint = true
  checkpoint_directory = ".checkpoints"
  ```

### 4. Checkpoint 集成 (agentfs)
- **功能描述**: 使用 agentfs 实现文件操作的 checkpoint 功能，支持回滚
- **实现细节**:
  - 在每个写操作前自动创建 checkpoint
  - 支持按会话管理 checkpoints
  - 提供手动创建和恢复 checkpoint 的工具
  - checkpoint 自动清理策略

- **数据结构**:
  ```rust
  // src/checkpoint/mod.rs
  pub struct CheckpointManager {
      fs: AgentFs,
      config: CheckpointConfig,
  }

  #[derive(Debug, Deserialize, Clone)]
  pub struct CheckpointConfig {
      pub enabled: bool,
      pub checkpoint_dir: String,
      pub max_checkpoints_per_session: usize,
      pub auto_cleanup_days: u64,
  }

  pub struct CheckpointInfo {
      pub id: String,
      pub session_id: String,
      pub created_at: DateTime<Utc>,
      pub description: Option<String>,
  }

  impl CheckpointManager {
      pub async fn create_checkpoint(
          &self,
          session_id: &str,
          description: Option<String>,
      ) -> Result<CheckpointInfo>;

      pub async fn restore_checkpoint(&self, checkpoint_id: &str) -> Result<()>;

      pub async fn list_checkpoints(&self, session_id: &str) -> Result<Vec<CheckpointInfo>>;

      pub async fn delete_checkpoint(&self, checkpoint_id: &str) -> Result<()>;
  }
  ```

- **Checkpoint 工具**:
  1. **create_checkpoint** - 手动创建 checkpoint
     ```rust
     pub struct CreateCheckpointParams {
         pub description: Option<String>,
     }
     pub struct CreateCheckpointResult {
         pub checkpoint_id: String,
         pub created_at: DateTime<Utc>,
     }
     ```

  2. **restore_checkpoint** - 恢复到指定 checkpoint
     ```rust
     pub struct RestoreCheckpointParams {
         pub checkpoint_id: String,
     }
     pub struct RestoreCheckpointResult {
         pub success: bool,
     }
     ```

  3. **list_checkpoints** - 列出当前会话的 checkpoints
     ```rust
     pub struct ListCheckpointsParams {}
     pub struct ListCheckpointsResult {
         pub checkpoints: Vec<CheckpointInfo>,
     }
     ```

### 5. 本地工具集成
- **功能描述**: 本地工具直接作为函数集成到 `ToolExecutor`，避免 MCP 延迟
- **实现细节**:
  - 直接实现 `LocalTool` trait
  - 注册到 `ToolRegistry` 作为本地工具
  - 直接函数调用，无进程间通信延迟
  - 工具权限与配置集成
  - 会话隔离

- **架构设计**:
  ```rust
  // src/tools/mod.rs
  #[async_trait]
  pub trait LocalTool: Send + Sync {
      fn name(&self) -> &str;
      fn description(&self) -> &str;
      fn input_schema(&self) -> serde_json::Value;
      async fn call(&self, arguments: serde_json::Value, context: ToolContext) -> Result<serde_json::Value>;
  }

  pub struct ToolContext {
      pub session_id: String,
      pub config: Arc<Config>,
      pub checkpoint_manager: Option<Arc<CheckpointManager>>,
  }

  pub struct LocalToolRegistry {
      tools: HashMap<String, Arc<dyn LocalTool>>,
  }

  impl LocalToolRegistry {
      pub fn new() -> Self;
      pub fn register(&mut self, tool: Arc<dyn LocalTool>);
      pub fn list_tools(&self) -> Vec<Tool>;
      pub async fn call_tool(
          &self,
          tool_name: &str,
          arguments: serde_json::Value,
          context: ToolContext,
      ) -> Result<ToolResult>;
  }
  ```

- **配置示例**:
  ```toml
  # config/mineclaw.toml
  [local_tools]
  enabled = true

  [local_tools.terminal]
  enabled = true

  [local_tools.filesystem]
  enabled = true

  [local_tools.checkpoint]
  enabled = true
  ```

---

## 项目结构变更

```
mineclaw/
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── config.rs              # 扩展配置
│   ├── error.rs               # 扩展错误类型
│   ├── state.rs
│   ├── encryption.rs          # 新增：加密管理
│   ├── tool_coordinator.rs    # 更新：集成本地工具
│   ├── api/
│   │   ├── handlers.rs
│   │   ├── routes.rs
│   │   └── sse.rs
│   ├── models/
│   │   ├── mod.rs
│   │   ├── message.rs
│   │   ├── session.rs
│   │   └── sse.rs
│   ├── llm/
│   │   ├── mod.rs
│   │   └── client.rs
│   ├── mcp/
│   │   ├── mod.rs
│   │   ├── protocol.rs
│   │   ├── transport.rs
│   │   ├── client.rs
│   │   ├── server.rs
│   │   ├── registry.rs          # 更新：支持本地工具
│   │   └── executor.rs          # 更新：支持本地工具执行
│   ├── tools/                   # 新增：本地工具
│   │   ├── mod.rs
│   │   ├── registry.rs          # 本地工具注册表
│   │   ├── terminal.rs          # 终端工具
│   │   ├── filesystem.rs        # 文件工具
│   │   └── checkpoint.rs        # Checkpoint 工具
│   └── checkpoint/              # 新增：Checkpoint 管理
│       ├── mod.rs
│       └── manager.rs
├── src/bin/
│   └── keygen.rs                # 新增：密钥生成工具
├── tests/
│   ├── mcp_integration.rs
│   ├── encryption_tests.rs      # 新增：加密测试
│   ├── terminal_tests.rs        # 新增：终端工具测试
│   ├── filesystem_tests.rs      # 新增：文件工具测试
│   └── checkpoint_tests.rs      # 新增：Checkpoint 测试
└── config/
    └── mineclaw_template.toml    # 更新配置模板
```

---

## 配置文件扩展

```toml
# config/mineclaw.toml
[server]
host = "127.0.0.1"
port = 18789

[llm]
provider = "openai"
api_key = "encrypted:..."  # 加密后的 API Key
base_url = "https://api.openai.com/v1"
model = "gpt-4o"
max_tokens = 2048
temperature = 0.7

[encryption]
# 加密密钥通过环境变量 MINECLAW_ENCRYPTION_KEY 提供

[local_tools]
enabled = true

[local_tools.terminal]
enabled = true
max_output_bytes = 65536
timeout_seconds = 300
allowed_workspaces = ["/path/to/workspace"]

# 命令黑名单（正则表达式）
command_blacklist = [
    "^rm -rf /",
    "^mkfs",
    "^dd if=",
    "^:\\(\\)\\{ :\\|:& \\};:",
    "^chmod 777",
    "^chown -R",
]

[local_tools.terminal.filters]
# Cargo 过滤
"cargo build" = [
    "^\\s*Compiling",
    "^\\s*Building",
    "^\\s*Finished",
    "^\\s*Running",
]
"cargo run" = [
    "^\\s*Compiling",
    "^\\s*Building",
    "^\\s*Finished",
]
"cargo test" = [
    "^\\s*Compiling",
    "^\\s*Building",
    "^\\s*Finished",
]

# Pip 过滤
"pip install" = [
    "^Collecting",
    "^Downloading",
    "^\\s+\\d+%",
    "^Installing collected packages",
    "^Successfully installed",
]
"pip3 install" = [
    "^Collecting",
    "^Downloading",
    "^\\s+\\d+%",
    "^Installing collected packages",
    "^Successfully installed",
]

# NPM 过滤
"npm install" = [
    "^npm WARN",
    "^npm ERR",
    "^added",
    "^removed",
    "^changed",
]

[local_tools.filesystem]
enabled = true
max_read_bytes = 16384
allowed_directories = [
    "/path/to/workspace",
]

[local_tools.checkpoint]
enabled = true
checkpoint_directory = ".checkpoints"
max_checkpoints_per_session = 50
auto_cleanup_days = 30

[mcp]
enabled = true

[[mcp.servers]]
# 外部 MCP 服务器配置...
```

---

## 实现步骤

### Phase 3.1: API Key 加密
- [ ] 设计加密模块架构
- [ ] 实现 `EncryptionManager`
- [ ] 集成到配置加载流程
- [ ] 创建密钥生成工具 (`src/bin/keygen.rs`)
- [ ] 编写单元测试
- [ ] 更新配置模板

### Phase 3.2: 终端工具
- [ ] 设计终端工具配置结构
- [ ] 实现命令黑名单检查
- [ ] 实现输出过滤系统
- [ ] 实现 `TerminalTool`
- [ ] 集成 SSE 流式输出
- [ ] 编写单元测试

### Phase 3.3: 文件工具集
- [ ] 设计文件工具配置结构
- [ ] 实现路径安全检查
- [ ] 实现 10 个文件工具
  - [ ] `read_file`
  - [ ] `write_file`
  - [ ] `list_directory`
  - [ ] `search_file`
  - [ ] `move_file`
  - [ ] `move_directory`
  - [ ] `delete_file`
  - [ ] `delete_directory`
  - [ ] `create_directory`
  - [ ] `search_and_replace`
- [ ] 集成读取大小限制
- [ ] 编写单元测试

### Phase 3.4: Checkpoint 集成
- [ ] 添加 agentfs 依赖
- [ ] 设计 checkpoint 配置结构
- [ ] 实现 `CheckpointManager`
- [ ] 集成到文件写操作
- [ ] 实现 checkpoint 工具
  - [ ] `create_checkpoint`
  - [ ] `restore_checkpoint`
  - [ ] `list_checkpoints`
- [ ] 实现自动清理策略
- [ ] 编写单元测试

### Phase 3.5: 本地工具集成
- [ ] 实现 `LocalTool` trait
- [ ] 实现 `LocalToolRegistry`
- [ ] 更新 `ToolRegistry` 支持本地工具
- [ ] 更新 `ToolExecutor` 支持本地工具执行
- [ ] 更新 `ToolCoordinator` 集成本地工具
- [ ] 更新 `AppState` 添加本地工具注册表
- [ ] 编写集成测试

### Phase 3.6: 集成与测试
- [ ] 更新配置加载流程
- [ ] 更新 `AppState` 初始化
- [ ] 完整端到端测试
- [ ] 更新文档
- [ ] 安全审计

---

## 依赖更新

需要在 `Cargo.toml` 中添加以下依赖：

```toml
# 加密相关
aes-gcm = "0.10"
rand = "0.8"
base64 = "0.22"

# 文件系统与 checkpoint
agentfs = "0.1"  # 需确认实际版本

# 其他
regex = "1.10"
walkdir = "2.5"
```

---

## 测试检查清单

### Phase 3.1: API Key 加密
- [ ] 密钥生成正常
- [ ] 加密/解密功能正常
- [ ] 配置文件加载时自动解密
- [ ] 错误处理正常（密钥错误等）

### Phase 3.2: 终端工具
- [ ] 命令执行正常
- [ ] 输出限制生效
- [ ] 黑名单命令被阻止
- [ ] 过滤规则生效
- [ ] 工作目录限制生效
- [ ] 超时控制生效
- [ ] SSE 流式输出正常

### Phase 3.3: 文件工具集
- [ ] `read_file` 正常（含截断）
- [ ] `write_file` 正常
- [ ] `list_directory` 正常
- [ ] `search_file` 正常
- [ ] `move_file` 正常
- [ ] `move_directory` 正常
- [ ] `delete_file` 正常
- [ ] `delete_directory` 正常
- [ ] `create_directory` 正常
- [ ] `search_and_replace` 正常
- [ ] 路径遍历防护生效
- [ ] 目录限制生效

### Phase 3.4: Checkpoint 集成
- [ ] Checkpoint 创建正常
- [ ] Checkpoint 恢复正常
- [ ] Checkpoint 列表正常
- [ ] 自动 checkpoint 在写操作前创建
- [ ] 自动清理正常
- [ ] 会话隔离正常

### Phase 3.5: 本地工具集成
- [ ] 本地工具注册正常
- [ ] 工具列表查询正常（包含 MCP 和本地工具）
- [ ] 本地工具调用执行正常
- [ ] 本地工具与 MCP 工具协同工作正常
- [ ] 会话隔离正常

---

## 安全考虑

### 1. 命令黑名单
- 使用正则表达式匹配
- 定期更新内置黑名单
- 支持用户自定义黑名单

### 2. 文件系统隔离
- 严格的路径规范化
- `..` 检测与阻止
- 白名单目录机制
- 符号链接处理

### 3. API Key 安全
- AES-GCM 256 位加密
- 密钥通过环境变量提供
- 不在日志中输出

### 4. 资源限制
- 输出大小限制
- 执行时间限制
- 内存使用监控（可选）

---

## 后续扩展方向

Phase 3 完成后，可以考虑：

1. **更多工具**: Git 工具、Docker 工具、数据库工具等
2. **Web UI**: 提供图形化界面管理会话和工具
3. **多用户**: 支持多用户隔离和权限管理
4. **插件系统**: 支持自定义工具插件
5. **审计日志**: 详细的操作审计和回放
