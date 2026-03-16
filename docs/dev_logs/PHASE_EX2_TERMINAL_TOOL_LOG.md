```# Phase EX2: 交互式分页器防护 (Pager Protection) 开发日志

## 1. 任务背景
在长时任务管理（Phase EX1）解决掉任务耗时过长的阻塞问题后，Phase EX2 旨在彻底消除终端工具中常见的“交互式挂起”风险。诸如 `less`、`more`、`git log` 或 `man` 等工具在默认情况下会启动交互式分页器（Pager），这会导致 Agent 的标准输出流被挂起，等待用户按下 `q` 或空格键。在无头（Headless）运行的 Agent 环境中，这将导致进程永久卡死。本阶段将通过环境注入、指令拦截和智能引导，确保终端执行环境始终是非交互式的。

## 2. 开发进度记录

### Step 1: 环境抑制注入 (2026-03-16)
- **目标**：在所有终端进程启动前，强制注入环境变量，诱导主流工具自动降级为非交互式的 `cat` 模式。
- **涉及文件**：
    - `src/tools/terminal.rs`
- **修改内容**：
    - **环境变量预设**：在 `RunCommandTool::call` 的 `Command` 构造阶段，添加 `env("PAGER", "cat")`、`env("MANPAGER", "cat")`、`env("GIT_PAGER", "cat")` 等。
    - **覆盖逻辑**：确保即使 LLM 尝试手动设置这些环境变量，系统级的安全预设也能起到兜底作用（或者至少提供稳健的默认值）。
- **验证状态**：
    - [x] **功能验证**：通过注入测试验证了环境变量 `PAGER` 已正确强制设为 `cat`。
    - [x] **静态检查**：`cargo clippy` 检查通过，新增单元测试 `test_run_command_pager_protection` 验证通过。

### Step 2: 指令级拦截逻辑 (2026-03-16)
- **目标**：利用 Tokenizer 在预检阶段识别并直接拦截 `less` 等纯交互式指令，防止 Agent 尝试启动交互式会话。
- **涉及文件**：
   - `src/tools/terminal.rs`
- **修改内容**：
   - **硬编码禁止列表扩展**：在 `is_command_blacklisted` 中增加 `less`, `more`, `vi`, `vim`, `nano`, `man` 等交互式编辑/查看器的拦截。
   - **Tokenizer 匹配优化**：确保即使在管道流中（如 `cat file | less`）也能精准识别并拦截危险环节。
- **验证状态**：
   - [x] **拦截验证**：确保执行 `less test.txt` 时触发 Security Policy 错误，新增单元测试 `test_run_command_interactive_blocked` 验证通过。
   - [x] **静态检查**：`cargo clippy` 检查通过。

### Step 2.1: 回归修复与鲁棒性增强 (2026-03-16)
- **目标**：修复由于 `RunCommandParams` 结构变更导致的 `test_functional_empty_command` 测试失败，并增强参数校验。
- **涉及文件**：
   - `src/tools/terminal.rs`
- **修改内容**：
   - **参数结构调整**：移除 `command` 字段上的 `#[serde(default)]`。在非续航场景下，必须显式提供命令。
   - **显式空值校验**：在 `call` 方法入口增加校验。若既无 `task_id` 也无有效 `command`（含空格字符串），则直接拒绝执行。
- **验证状态**：
   - [x] **回归验证**：`terminal_functional_tests::test_functional_empty_command` 重新通过。
   - [x] **全量验证**：110+ 项全量测试全部通过。
 
   ### Step 3: 智能引导机制 (2026-03-16)
   - **目标**：优化交互式命令拦截的反馈。当拦截发生时，不直接抛出系统级 Security Policy 错误，而是返回定制化的“MineClaw 助手提示”，告知 Agent 终端环境的非交互特性，并建议使用 `cat`、`head`、`tail` 或 `grep` 等替代方案。
   - **涉及文件**：
      - `src/tools/terminal.rs`
   - **修改内容**：
      - **错误信息定制**：重构 `is_command_blacklisted` 的拦截返回，识别分页器和编辑器并构造特定的建议文本。
      - **Agent 交互优化**：确保 LLM 收到的是包含指令替代建议的“执行结果”或“有意义的错误信息”，而非冷冰冰的访问拒绝。
   - **验证状态**：
      - [x] **提示验证**：重构了 `is_command_blacklisted` 返回 `Option<String>`，成功捕获 `less` 等命令并返回包含 "MineClaw Hint" 和替代建议的定制化错误信息。
      - [x] **静态检查**：`cargo clippy` 检查通过，单元测试 `test_run_command_interactive_blocked` 验证通过。

### Step 4: 管道流容错与拦截增强 (2026-03-16)
- **目标**：专门处理 `| less` 场景，确保在复合命令中也能稳定识别并拦截交互式分页器，提供针对性的管道优化建议。
- **涉及文件**：
   - `src/tools/terminal.rs`
- **修改内容**：
   - **拦截逻辑深化**：在 `is_command_blacklisted` 中不仅检查命令首项，还需扫描整个子指令字符串，防止利用管道绕过拦截。
   - **定制引导消息**：针对管道场景（如检测到 `|` 且包含 `less`），返回特定的“管道优化建议”，告知 Agent 无需分页，直接让管道流向 stdout。
- **验证状态**：
   - [x] **拦截验证**：通过扫描子指令实现了管道内的分页器识别，单元测试 `test_run_command_pipeline_pager_blocked` 验证通过，成功返回针对性的管道优化建议。
   - [x] **静态检查**：`cargo clippy` 检查通过。

### Step 4.1: 回归修复 - 配置黑名单生效 (2026-03-16)
- **目标**：修复由于重构 `is_command_blacklisted` 导致配置中的 `command_blacklist`（简单字符串匹配）失效的问题。
- **涉及文件**：
   - `src/tools/terminal.rs`
- **修改内容**：
   - **逻辑补全**：在 `is_command_blacklisted` 中重新加入了对 `context.config.local_tools.terminal.command_blacklist` 的遍历检查，确保通过配置文件定义的黑名单依然有效。
- **验证状态**：
   - [x] **回归验证**：集成测试 `terminal_integration::test_terminal_tool_blacklist` 重新通过。
   - [x] **全量验证**：全量 110+ 项测试全部通过。
