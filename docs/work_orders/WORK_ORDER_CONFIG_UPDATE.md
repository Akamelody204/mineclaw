# MineClaw 配置更新工单 (Work Order: Config Update)

## 1. 任务背景
在 Phase 2.1 中，我们扩展了数据模型并定义了基础配置结构。为了支持更灵活的运行时行为，需要实现动态配置 API，允许系统在不重启的情况下热加载部分配置，并提供安全的配置读写接口。

## 2. 需求描述
*   **Config Struct**: 完善 `Config` 结构体，支持 serde 序列化/反序列化。
*   **API Endpoints**:
    *   `GET /api/config`: 获取当前配置（需脱敏敏感信息）。
    *   `POST /api/config`: 更新配置（需验证权限与合法性）。
*   **热加载 (Hot Reload)**: 配置变更后，系统应能自动感知并应用新值（如 LLM 参数变更）。
*   **持久化**: 更新后的配置应回写到 `config/mineclaw.toml`。

## 3. 实现细节
*   使用 `tokio::sync::RwLock` 保护全局配置状态。
*   实现 `Config::load()` 和 `Config::save()` 方法。
*   API 层增加 `auth_middleware` 确保只有授权用户可修改配置。
*   对于敏感字段（如 API Key），在返回给前端时显示为 `******`。

## 4. 验证计划
*   **测试 1**: 启动服务，调用 `GET /api/config`，验证默认值。
*   **测试 2**: 调用 `POST /api/config` 修改 `llm.temperature`。
*   **测试 3**: 再次调用 `GET` 确认值已更新。
*   **测试 4**: 检查磁盘上的 `mineclaw.toml` 是否已同步修改。
