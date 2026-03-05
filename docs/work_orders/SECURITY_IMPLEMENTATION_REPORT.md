# MineClaw 安全性实现报告 (Security Implementation Report)

## 1. 概述
*   **报告日期**: 2026-03-05
*   **报告类型**: 安全审计与实现验证
*   **核心关注点**: 敏感数据存储、API Key 管理、运行时内存安全

## 2. 安全架构

### 2.1 核心机制：AES-256-GCM 透明加密
*   **Master Key**: 
    *   在首次启动时自动生成 32 字节随机密钥。
    *   存储于 `config/master.key` (二进制文件)。
    *   **要求**: 该文件必须严格限制文件系统权限，且禁止提交到版本控制系统。
*   **数据加密**:
    *   使用 `AES-256-GCM` 算法。
    *   每次加密生成唯一的 12 字节 Nonce。
    *   密文格式: `Nonce (12B) + Ciphertext + Tag`，并进行 Base64 编码。

### 2.2 配置生命周期
1.  **加载 (Load)**: 
    *   读取 `mineclaw.toml`。
    *   检测到 `llm.api_key` 字段。
    *   如果以 `enc:` 开头，尝试使用 Master Key 解密。
    *   解密失败则 Panic，阻止不安全启动。
2.  **运行时 (Runtime)**:
    *   内存中持有**明文** API Key (仅存在于堆内存中)。
    *   Drop 时自动清理 (Rust 所有权机制保证)。
3.  **保存 (Save)**:
    *   当通过 API 更新配置时，自动对敏感字段进行加密。
    *   写入磁盘的永远是 `enc:...` 格式的密文。

## 3. 实现验证

### 3.1 首次启动测试
*   **前置条件**: 无 `config/master.key`，无配置文件。
*   **行为**: 
    *   生成新 Master Key。
    *   生成默认配置模板。
*   **结果**: Pass

### 3.2 配置文件加密测试
*   **操作**: 手动在 `mineclaw.toml` 中填入明文 Key: `sk-test-123456`。
*   **行为**: 
    *   系统启动时识别明文。
    *   **自动重写**配置文件，将 Key 替换为 `enc:YWJjZ...`。
*   **结果**: Pass (Verified by file inspection)

### 3.3 密钥丢失模拟
*   **操作**: 删除 `config/master.key` 但保留加密后的 `mineclaw.toml`。
*   **行为**: 系统启动失败，报错 `Decryption failed`。
*   **结果**: Pass (符合预期，防止密钥泄露导致数据被解密)

## 4. 安全建议
1.  **Master Key 轮换**: 当前未实现自动轮换，建议后续增加 CLI 命令支持。
2.  **内存保护**: 虽然 Rust 内存安全，但建议对极高敏感度数据使用 `mlock` 防止交换到磁盘 (Phase 3 考虑)。
3.  **权限最小化**: 生产环境应确保 `mineclaw` 进程以非 root 用户运行，且仅对 config 目录有读写权限。
