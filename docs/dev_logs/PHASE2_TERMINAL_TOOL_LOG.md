```# MineClaw Terminal Tool 开发日志 - Phase 2

## 2025-05-24: 安全性与权限管理 (Phase 2)

### 开发进度总结
- [x] **Step 1: 增强型硬编码规则系统** (Refining Zed-style Rules)
- [x] **Step 2: 完整正则匹配体系** (Always Allow / Always Confirm)
- [x] **Step 3: 路径规范化防护系统** (Path Normalization Defense)
- [x] **Step 4: 配置系统深度集成与验证** (Configuration Integration)
- [x] **Step 5: 交互式确认机制** (Interactive Confirmation)
- [x] **Step 6: 消除正则现场编译回退** (Regex Pre-compilation Enforcement)
- [x] **Step 7: 精细化 Tokenize 校验** (Shell-Aware Command Tokenization)

---

### Phase 2 前置状态说明 (Pre-status)
在 Phase 1 的整改过程中，为了修复安全测试报错并提升性能，我们已经提前完成了以下工作：
- **初步硬编码拦截**：实现了对 `rm` 变体（`.`, `..`, `~`, `/`）的智能解析。
- **正则黑名单基础**：实现了基于 `compiled_blacklist` 的高性能过滤。
- **路径规范化**：通过 `canonicalize` 和 UNC 路径剥离彻底解决了 Windows 下的路径遍历与兼容性问题。

在 Phase 2 中，我们将进一步深化这些机制，特别是引入“始终允许”和“交互确认”的高级权限控制。

---

### Step 1: 增强型硬编码规则系统 (Completed)
**功能实现**：
- **扩大黑名单覆盖面**：参考 Zed 安全实践，在 `hardcoded_blacklist` 中新增了多项高危指令，包括磁盘分区 (`fdisk`, `parted`)、系统强制指令 (`shutdown`, `reboot`, `halt`, `poweroff`)、破坏性任务删除 (`crontab -r`) 以及经典的 Fork 炸弹。
- **多维度拦截**：结合已有的 `rm` 变体智能解析逻辑，形成了“高危词汇静态拦截 + 危险参数动态解析”的双重硬编码防护网。
- **质量保证**：通过 `cargo clippy` 验证及全量 `terminal_security_tests` 验证，确保防护逻辑生效且无性能衰减。

**涉及文件**：
- `mineclaw\src\tools\terminal.rs`: 扩展 `is_command_blacklisted` 中的硬编码列表。

---

### Step 2: 完整正则匹配体系 (Completed)
**功能实现**：
- **多层权限过滤链**：在 `RunCommandTool` 中确立了 `Always Allow` -> `Blacklist` -> `Always Confirm` 的分层校验流程。
- **配置规则逻辑化**：
    - **Always Allow (白名单)**：若命中此规则，直接跳过后续所有安全检查执行命令。
    - **Blacklist (黑名单)**：扩充了对配置中 `command_blacklist_regex` 的应用。
    - **Always Confirm (预警提示)**：识别需要用户介入的命令，目前作为安全拦截并提示 UI 尚未就绪（为 Step 3 铺路）。
- **性能一致性**：同样采用了“预编译优先 + 实时编译回退”的双轨匹配逻辑，确保即使在复杂正则规则下也能保持高性能响应。

**涉及文件**：
- `mineclaw\src\tools\terminal.rs`: 实现 `is_command_always_allowed` 和 `is_command_confirmation_required` 及其在 `call` 方法中的应用。

---

### Step 3: 路径规范化防护系统 (Completed)
**功能实现**：
- **手动路径解析算法**：实现了 `normalize_path_safe` 函数，通过手动解析 `Path` 组件（Components），严格拦截任何尝试在逻辑根目录（RootDir/Prefix）之上使用 `..` 的行为。
- **前置安全过滤**：在 `is_working_dir_allowed` 的第一阶段引入该算法。不同于 `canonicalize`（依赖磁盘存在性），该算法纯基于逻辑解析，能更早、更彻底地拦截带有恶意遍历模式的路径。
- **跨平台一致性**：算法充分考虑了 Windows 盘符（Prefix）和 Unix 根目录的区别，确保在不同操作系统下均能正确识别“逻辑边界”。

**涉及文件**：
- `mineclaw\src\tools\terminal.rs`: 实现 `normalize_path_safe` 并在 `is_working_dir_allowed` 中应用。

---

### Step 4: 配置系统深度集成与验证 (Completed)
**功能实现**：
- **Schema 扩展**：在 `TerminalConfig` 中正式集成了 `command_blacklist_regex`、`always_allow_regex` 和 `always_confirm_regex` 字段，支持通过 TOML 配置文件或环境变量动态定义权限规则。
- **自动生命周期管理**：在 `Config::load()` 阶段自动触发 `compile_terminal_regexes()`，实现了权限规则的“加载即编译”，从架构层面确保了执行阶段的极致性能。
- **配置健壮性**：为新增配置项提供了默认值（空列表），并完善了配置加载逻辑，确保了系统在不同部署环境下的兼容性。

**涉及文件**：
- `mineclaw\src\config.rs`: 扩展配置结构体并实现正则预编译逻辑。

---

### Step 5: 交互式确认机制 (Completed)
**功能实现**：
- **引入确认错误变体**：在 `src/error.rs` 中新增了 `ConfirmationRequired` 错误枚举，用于向调用方（API/UI）发出明确的挂起信号。
- **参数闭环控制**：在 `RunCommandParams` 中增加了 `confirmed` 隐藏参数。工具在检测到敏感操作时，若该参数为 `false` 则中断执行并抛出确认请求。
- **权限闭环逻辑**：在 `RunCommandTool::call` 中实现了对 `always_confirm` 正则规则的实时拦截，确保危险操作（如生产环境部署、敏感文件修改）必须经过人工二次确认。
- **API 响应适配**：在 `IntoResponse` 实现中为 `ConfirmationRequired` 分配了专用状态码及详细信息，便于前端/CLI 捕获并触发交互式确认流程。

**涉及文件**：
- `mineclaw\src\error.rs`: 增加 `ConfirmationRequired` 错误类型及响应映射。
- `mineclaw\src\tools\terminal.rs`: 实现基于 `confirmed` 标记的拦截逻辑与参数扩展。

---

### Step 6: 消除正则现场编译回退 (Completed)
**功能实现**：
- **强制预编译**：重构了 `RunCommandTool` 的校验逻辑，移除了原本用于兜底的现场正则编译分支。现在系统强制要求所有规则必须在配置加载阶段完成编译，确保了执行路径的极致性能。
- **测试环境优化**：同步更新了 `terminal_security_tests.rs`，在手动构造配置后显式调用 `compile_terminal_regexes()`。这既保证了测试的严谨性，也验证了预编译机制在极端构造场景下的可靠性。
- **架构纯净化**：移除了工具层对 `regex::Regex` 的直接依赖（导入），进一步解耦了业务执行逻辑与底层的正则解析引擎。

**涉及文件**：
- `mineclaw\src\tools\terminal.rs`: 移除现场编译逻辑。
- `mineclaw\tests\terminal_security_tests.rs`: 适配强制预编译流程。

---

### Step 7: 精细化 Tokenize 校验 (Completed)
**功能实现**：
- **Shell 语法感知**：引入了 `tokenize_commands` 算法，能够智能识别并拆解通过分号 (`;`)、管道 (`|`) 以及逻辑运算符 (`&&`, `||`) 连接的复合指令。
- **深度内容审查**：权限校验逻辑从原本的“整串扫描”进化为“原子指令级审查”。复合命令中的每一个子指令现在都必须独立通过 Always Allow、Blacklist 和 Always Confirm 的全链条校验，彻底杜绝了利用 Shell 连词绕过安全规则的可能性。
- **引号语义保护**：Tokenizer 能够正确处理单双引号，防止将路径或参数中的特殊字符误判为指令分隔符，提升了复杂参数场景下的校验准确性。

**涉及文件**：
- `mineclaw\src\tools\terminal.rs`: 实现指令拆解逻辑并重构校验循环。
