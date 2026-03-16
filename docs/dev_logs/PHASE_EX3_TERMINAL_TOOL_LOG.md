```# Phase EX3: 任务后台化与状态监控 (Background Tasks) 开发日志

## 1. 任务背景
在 Phase EX1 和 EX2 解决了长时任务的挂起续航以及交互式分页器防护后，Terminal Tool 已经具备了处理复杂任务的能力。然而，目前的 `call` 逻辑仍然是同步阻塞的（即 LLM 必须等待 `timeout` 或任务结束）。对于像“启动一个编译服务器”或“运行长时间测试套件”这类任务，LLM 不应被阻塞。Phase EX3 的目标是实现“任务分离 (Detach)”，允许 Agent 启动任务后立即获得 ID 并继续后续对话，同时支持随时查询后台任务的状态。

## 2. 开发进度记录

### Step 1: 任务分离 (Detach) 机制实现 (2026-03-16)
- **目标**：在 `RunCommandParams` 中增加 `detach` 动作，支持任务启动后立即返回 `task_id` 而不等待执行结果。
- **涉及文件**：
    - `src/tools/terminal.rs`
- **修改内容**：
    - **参数扩展**：在 `RunCommandParams` 中增加 `detach: bool` 字段。
    - **逻辑分支**：在 `call` 方法中判断，若 `detach` 为 `true`，则在 `spawn` 进程并建立 `ActiveProcess` 后，不进入 `run_and_handle_output` 的等待循环，而是直接将进程存入注册表并返回 `task_id`。
    - **状态标记**：在返回结果中明确标识任务已转入后台。
- **验证状态**：
    - [x] **功能验证**：新增单元测试 `test_run_command_detach` 验证通过，任务可立即返回 ID 且句柄正确保留在注册表中。
    - [x] **静态检查**：`cargo clippy` 检查通过，已适配全量现有测试用例。

### Step 2: 任务管理器工具 (list_background_tasks) (2026-03-16)
- **目标**：新增一个配套工具，允许 Agent 查询当前所有在后台运行的任务列表、PID、存活时长及最新的输出摘要。
- **涉及文件**：
    - `src/tools/terminal.rs`
- **修改内容**：
    - **新增工具定义**：实现 `ListBackgroundTasksTool` 结构体并注册到 `TerminalTool` 管理器中。
    - **信息收集**：遍历 `processes` 注册表，利用 `child.id()` 获取 PID，并计算自任务启动以来的存活时长。
    - **摘要返回**：提供 `stdout`/`stderr` 的最后若干字节作为快照摘要，帮助 Agent 判断任务进度。
- **验证状态**：
    - [x] **功能验证**：新增 `ListBackgroundTasksTool` 并成功通过 `test_list_background_tasks` 单元测试，可实时监控后台任务。
    - [x] **静态检查**：`cargo clippy` 检查通过，解决了状态共享导致的生命周期与所有权问题。

### Step 3: 结果检索与回收 (get_task_result) (2026-03-16)
- **目标**：实现 `get_task_result` 工具，让 Agent 能通过 `task_id` 随时拉取后台任务的完整输出，并手动终止 (kill) 已不再需要的任务。
- **涉及文件**：
    - `src/tools/terminal.rs`
- **修改内容**：
    - **新增工具定义**：实现 `GetTaskResultTool` 结构体，支持 `wait` (阻塞等待直到结束) 和 `kill` (强制终止) 动作。
    - **结果提取**：从 `processes` 注册表中根据 `task_id` 找回任务，并根据参数决定是继续监听 (`run_and_handle_output`) 还是直接强杀并清理注册表。
- **验证状态**：
    - [x] **结果提取**：实现了 `GetTaskResultTool`，支持通过 `task_id` 获取后台任务的当前输出或最终结果。
    - [x] **终止验证**：成功通过 `kill: true` 动作调用 `child.kill()` 并清理注册表。
    - [x] **后台 IO 重构**：为了支持 `detach` 后的持续输出捕获，重构了缓冲区为 `Arc<Mutex<Vec<u8>>>`，并引入 `tokio::spawn` 在后台持续读取输出流，解决了 `detach` 任务输出丢失的问题。
    - [x] **静态检查**：修复了 `MutexGuard` 跨 `await` 以及 `TerminalTool` 重定义等编译错误，`cargo clippy` 检查通过。

### Step 4: 自动清理逻辑 (TTL/GC) (2026-03-16)
- **目标**：为后台任务设置最大存活时间（TTL），防止进程无限堆积导致宿主机资源耗尽。
- **涉及文件**：
    - `src/config.rs`
    - `src/tools/terminal.rs`
- **修改内容**：
    - **配置扩展**：在 `TerminalConfig` 中增加 `background_task_ttl_minutes`（默认 30 分钟）。
    - **GC 扫描器**：在 `TerminalTool` 初始化时启动一个后台监控协程。
    - **清理策略**：定时（如每 5 分钟）扫描 `processes` 注册表，识别并强杀超过 TTL 的进程，释放句柄和缓冲区资源。
- **验证状态**：
    - [x] **清理逻辑实现**：在 `TerminalTool::register_all` 中成功启动后台 GC 协程，定期扫描并清理不活跃任务。
    - [x] **配置集成**：`TerminalConfig` 已支持 `background_task_ttl_minutes` 动态配置。
    - [x] **状态追踪**：`ActiveProcess` 增加了 `last_activity` 字段，每次 `get_task_result` 交互都会刷新活跃时间。
    - [x] **静态检查**：修复了 `HashMap` 类型推导和 `MutexGuard` 跨 `await` 的并发安全问题，`cargo clippy` 检查通过。

## 3. 下一步原子计划
- **Phase EX3 总结与文档更新**：
    - 完善长时任务管理的对外接口说明。
