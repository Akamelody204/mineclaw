```# MineClaw Terminal Tool 完整开发工单与日志 (Master Log)

本文件汇编了从 Phase 1 到 Phase EX3 的所有开发细节、设计决策、事故报告及经验总结。记录了 MineClaw 终端工具从基础执行引擎进化为生产级多任务管理系统的全过程。

---

## 目录
1. [Phase 1: 基础执行与核心架构](#phase-1-基础执行与核心架构)
2. [Phase 1.5: 漏洞整改与安全加固 (Hotfix)](#phase-15-漏洞整改与安全加固)
3. [Phase 2: 安全性与权限管理](#phase-2-安全性与权限管理)
4. [Phase 2.5: 安全深度加固与性能调优](#phase-25-安全深度加固与性能调优)
5. [Phase 3: 输出管理与容错 (Robustness)](#phase-3-输出管理与容错)
6. [Phase 3.5: 并发与截断策略优化](#phase-35-并发与截断策略优化)
7. [Phase 4: 高级集成与环境适配](#phase-4-高级集成与环境适配)
8. [Phase EX1: 交互式超时与长时任务管理](#phase-ex1-交互式超时与长时任务管理)
9. [Phase EX2: 交互式分页器防护 (Pager Protection)](#phase-ex2-交互式分页器防护)
10. [Phase EX3: 任务后台化与状态监控 (Background Tasks)](#phase-ex3-任务后台化与状态监控)

---

## Phase 1: 基础执行与核心架构

### 核心实现
- **工具接口定义**：定义了 `RunCommandTool` 结构体并实现 `LocalTool` trait，确立了终端工具在 `LocalToolRegistry` 中的注册标准。
- **基础执行逻辑**：针对 Windows 环境引入 `cmd /c` 包装，解决了 Shell 内置命令执行问题。
- **参数自动处理**：实现了带空格参数的自动引号封装，提升了命令解析兼容性。
- **依赖清理**：移除了过时的 `tokio-process` (v0.2.5)，彻底解决了与 Tokio 1.0+ 的冲突及 `net2` 带来的编译警告。

### 关键改进
- **路径规范化**：改用 `fs::canonicalize` 处理工作目录，解决了符号链接和 `..` 路径遍历风险。
- **异步超时控制**：结合 `tokio::time::timeout` 与 `child.wait_with_output()`，实现非阻塞执行时长限制。
- **进程树清理**：在超时发生时，针对 Windows 调用 `taskkill /F /T` 确保杀掉包括子 Shell 在内的整个进程树。

---

## Phase 1.5: 漏洞整改与安全加固 (Hotfix)

### 针对性修复
- **Windows UNC 路径修复**：解决了 `canonicalize()` 产生的 `\\?\` 前缀导致 `starts_with` 校验失效的问题。实现了 `normalize_unc` 函数统一路径格式。
- **参数注入根除**：彻底弃用手动拼接命令字符串。改用 `Command::arg()` 和 `args()` 接口，利用 Rust 标准库内置的转义机制杜绝注入风险。
- **正则性能优化**：在 `Config` 加载阶段引入正则表达式预编译逻辑，将匹配开销降至最低。

---

## Phase 2: 安全性与权限管理

### 权限体系构建
- **三级权限过滤链**：确立了 `Always Allow` -> `Blacklist` -> `Always Confirm` 的分层校验流程。
- **Zed 风格硬编码增强**：在拦截 `rm` 变体的基础上，新增了对磁盘分区 (`fdisk`)、系统停机 (`shutdown`) 及 Fork 炸弹的拦截。
- **路径规范化防护**：实现 `normalize_path_safe` 算法，在逻辑层面拦截任何尝试越过逻辑根目录的 `..` 遍历行为。

---

## Phase 2.5: 安全深度加固与性能调优

### 核心功能
- **交互式确认机制**：引入 `ConfirmationRequired` 错误变体。当命令触发 `always_confirm` 规则时，工具挂起执行并向调用方请求二次确认。
- **精细化 Tokenize 校验**：引入 Shell 语法感知算法，能够智能拆解通过 `;`, `|`, `&&`, `||` 连接的复合指令，确保每一个子指令都经过独立审查。
- **消除现场编译回退**：重构校验逻辑，强制要求所有规则必须预编译，消除了测试用的现场编译逻辑，保障高频调用延迟。

---

## Phase 3: 输出管理与容错 (Robustness)

### 稳定性增强
- **输出大小限制**：将默认输出限制提升至 1MB，防止 LLM 上下文溢出，并支持通过配置动态调整。
- **原子并发控制**：在 `RunCommandTool` 中引入 `AtomicUsize` 和 CAS 校验逻辑，实现 `max_concurrent_processes`（默认 4）限制。
- **RAII 资源管理**：使用 `ProcessGuard` 确保无论执行成功、失败或 Panic，进程计数均能正确释放。

---

## Phase 3.5: 并发与截断策略优化

### 细节微调
- **并发忙等优化**：在 CAS 循环中引入 `std::hint::spin_loop()`，有效降低了高竞争场景下的 CPU 损耗。
- **双向截断策略 (Head/Tail)**：重构 `truncate_output` 算法。在命令执行成功时保留头部，执行失败 (Exit Code != 0) 时自动保留尾部，确保 LLM 优先看到错误堆栈末尾。
- **行对齐保障**：截断算法优先在换行符位置切割，确保输出结果对 LLM 而言具有语义完整性。

---

## Phase 4: 高级集成与环境适配

### 协作系统对接
- **状态记录 (State Tracking)**：新增 `CommandHistoryEntry` 并实现最近 100 条记录的滚动存储，为上下文管理 Agent (Context Manager) 提供数据支撑。
- **跨平台 Shell 探测**：集成 `ShellDetector`，实现运行时的 OS 与 Shell 类型自动识别（PowerShell/Bash/CMD）。
- **PowerShell 稳健执行**：针对 Windows PowerShell 增加了 `-NonInteractive` 和 `-NoProfile` 标志，解决了交互式输入导致的测试卡死风险。

---

## Phase EX1: 交互式超时与长时任务管理

### 任务续航能力
- **异步双流监听**：重构执行逻辑，从 `wait_with_output` 转向基于 `tokio::select!` 的非阻塞 stdout/stderr 实时抓取。
- **实时画面快照 (Snapshot)**：实现超时后的“部分输出返回”功能。即使任务未结束，Agent 也能看到当前已抓取的内容。
- **活跃进程注册表**：引入 `ActiveProcessRegistry`。超时任务不再直接强杀，而是保留句柄并分配 `task_id`，支持 Agent 发送 `continue` 指令进行断点续航。

---

## Phase EX2: 交互式分页器防护 (Pager Protection)

### 自动化鲁棒性
- **环境抑制注入**：在进程启动前强制注入 `PAGER=cat`, `MANPAGER=cat`, `GIT_PAGER=cat` 环境变量，诱导工具放弃交互模式。
- **分页器指令拦截**：精准拦截 `less`, `more`, `vi`, `man` 等交互式工具。
- **MineClaw 智能引导**：当拦截发生时，返回包含 "MineClaw Hint" 的替代建议（如建议使用 `cat`, `grep`），帮助 Agent 修正操作逻辑。

---

## Phase EX3: 任务后台化与状态监控 (Background Tasks)

### 异步任务调度
- **任务分离 (Detach)**：支持 Agent 启动任务后立即返回 `task_id`，不再阻塞对话过程。
- **后台 IO 重构**：缓冲区升级为 `Arc<Mutex<Vec<u8>>>`，通过 `tokio::spawn` 的后台协程持续捕获分离任务的输出，解决了 `detach` 导致输出丢失的问题。
- **后台管理器 (List & Get)**：
    - 新增 `ListBackgroundTasksTool`：查询所有存活任务、存活时长及最新输出摘要。
    - 新增 `GetTaskResultTool`：支持通过 `task_id` 找回任务、获取结果或强制终止。
- **自动清理机制 (TTL/GC)**：在 `TerminalTool` 初始化时启动后台扫描协程，自动清理超过 30 分钟（默认）未交互的不活跃任务，防止资源泄露。

---

## 事故报告与解决方案总结

| 事故现象 | 根本原因 | 解决方案 |
| :--- | :--- | :--- |
| **Windows 路径校验失败** | `canonicalize()` 产生的 UNC 前缀 (`\\?\`) 与配置字符串不匹配 | 引入 `normalize_unc` 剥离前缀，并统一大小写匹配 |
| **测试套件卡死 (Hang)** | PowerShell 某些命令在非交互模式下尝试等待输入 | 强制添加 `-NonInteractive` 标志，改用 `-Command` 包装 |
| **内存/句柄增长风险** | Phase EX1 引入的注册表在任务完成后未及时移除 | 在 EX3 引入后台 GC 协程，结合 `last_activity` 实现 TTL 清理 |
| **MutexGuard 跨 await 报错** | 在 `async` 函数中使用普通 `std::sync::Mutex` | 将长生命周期的锁逻辑重构，或缩小作用域，确保锁不跨越 `.await` 点 |
| **输出丢失 (Detach)** | 进程分离后，主协程退出导致 Pipe 被关闭 | 使用 `tokio::spawn` 启动独立 IO 监听协程维持输出捕获 |

---

## 经验总结
1. **安全重于功能**：终端工具是 Agent 的最高权限入口，必须在每一层（环境、指令、参数、路径）都实施冗余的纵深防御。
2. **异步非阻塞是核心**：利用 `tokio::select!` 和后台协程处理 IO，是实现“长时任务管理”和“后台任务分离”的技术前提。
3. **平台差异必须重视**：Windows 和 Unix 的进程管理、路径规则、Shell 行为差异极大，必须通过专门的探测器实现动态适配。
4. **自动化需要引导**：拦截危险命令后，返回“为什么拦截”以及“替代建议”比直接报错更能有效维持 Agent 的工作流。

---
**Made with ❤️ by the MineClaw Team**```
