# MineClaw Phase 3.1 人工验证计划 (Manual Verification Plan)

虽然自动化测试已经覆盖了核心逻辑和安全边界，但为了确保**端到端用户体验**和**真实环境下的可用性**，我们需要进行以下人工验证步骤。

## 验证环境
- OS: Windows (当前环境)
- Shell: PowerShell
- 工具: `keygen.exe` (已编译), `mineclaw.exe` (主程序)

## 验证步骤

### 1. 密钥生成与管理体验
- [ ] **步骤**: 运行 `cargo run --bin keygen`。
- [ ] **预期**:
  - 输出一个新的 Base64 密钥。
  - 提示用户设置环境变量 `MINECLAW_ENCRYPTION_KEY`。
- [ ] **验证点**: 复制生成的密钥，是否方便？提示是否清晰？

### 2. 密文生成体验
- [ ] **步骤**: 使用上一步生成的密钥，运行 `cargo run --bin keygen -- --encrypt <KEY> "sk-real-secret-key-123"`。
- [ ] **预期**:
  - 输出 `encrypted:xxxx...` 格式的字符串。
- [ ] **验证点**: 密文长度是否合理？格式是否符合配置文件要求？

### 3. 配置文件集成 (模拟真实用户流程)
- [ ] **步骤**:
  1. 创建一个临时的 `config/test_config.toml`。
  2. 在其中填入 `[llm] api_key = "encrypted:..."` (使用第2步生成的密文)。
  3. 设置环境变量 `$env:MINECLAW_ENCRYPTION_KEY = "..."` (使用第1步生成的密钥)。
  4. 运行主程序 (或编写一个小脚本调用 `Config::load`) 读取该配置。
  *注：由于主程序启动流程较复杂，我们可以通过修改 `src/main.rs` 打印 Config 或使用 `cargo run` 的测试模式来验证。鉴于我们已有集成测试，这里主要验证“人”的操作流程是否顺畅。*

### 4. 错误处理体验 (模拟用户配置错误)
- [ ] **场景 A**: 忘记设置环境变量。
  - **步骤**: 清除环境变量，再次运行加载配置。
  - **预期**: 程序启动失败，控制台输出清晰的错误信息 "MINECLAW_ENCRYPTION_KEY is missing"。
- [ ] **场景 B**: 密钥错误。
  - **步骤**: 修改环境变量为错误的密钥，运行加载配置。
  - **预期**: 程序启动失败，报错 "Decryption failed"。

### 5. 内存零化验证 (高级)
- [ ] **步骤**: 使用调试器 (如 GDB/LLDB，但在 Windows 上可能受限) 或插入临时打印代码，在 `EncryptionManager` drop 后尝试读取密钥内存。
  *注：此步骤在当前环境中难以通过简单的 CLI 完成，依赖代码审查和 `zeroize` crate 的信誉。*

## 执行记录

我们将使用 PowerShell 脚本来模拟上述步骤 1, 2, 4。

```powershell
# 1. Generate Key
$key = cargo run -q --bin keygen | Select-String "Generate" -Context 0,2 | ForEach-Object { $_.Context.PostContext[0].Trim() }
Write-Host "Generated Key: $key"

# 2. Encrypt Data
$plaintext = "sk-secret-123"
$encrypted = cargo run -q --bin keygen -- --encrypt $key $plaintext
Write-Host "Encrypted: $encrypted"

# 3. (Optional) Verify via Test
# 我们已经有 automated test 覆盖了加载流程，这里主要确认 keygen 工具生成的 output 能被系统识别。
```
