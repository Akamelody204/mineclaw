# MineClaw Config API 测试报告 (Test Report: Config API)

## 1. 测试概览
*   **测试对象**: Config API (`/api/config`)
*   **测试目标**: 验证动态配置读取、更新、权限控制及热加载机制。
*   **测试环境**: 本地开发环境 (Windows 11)
*   **测试工具**: curl, Postman

## 2. 测试用例执行情况

| 用例 ID | 测试描述 | 预期结果 | 实际结果 | 状态 |
| :--- | :--- | :--- | :--- | :--- |
| **TC-001** | 未授权访问 GET /api/config | 返回 401 Unauthorized | 401 Unauthorized | ✅ Pass |
| **TC-002** | 授权访问 GET /api/config | 返回 200 OK 及配置 JSON，敏感字段脱敏 | 200 OK, api_key: "******" | ✅ Pass |
| **TC-003** | 更新非敏感字段 (temperature) | 返回 200 OK，GET 验证值已更新 | 200 OK, value updated | ✅ Pass |
| **TC-004** | 更新非法值 (temperature > 2.0) | 返回 400 Bad Request | 400 Bad Request | ✅ Pass |
| **TC-005** | 更新敏感字段 (API Key) | 返回 200 OK，系统能使用新 Key 调用 LLM | 200 OK, LLM call success | ✅ Pass |
| **TC-006** | 配置文件持久化 | 重启服务后，配置变更依然存在 | 变更已保存至 toml | ✅ Pass |

## 3. 详细测试记录

### 3.1 敏感信息脱敏测试
请求：
```http
GET /api/config HTTP/1.1
Authorization: Bearer <valid_token>
```
响应：
```json
{
  "server": { "host": "127.0.0.1", "port": 8080 },
  "llm": {
    "provider": "openai",
    "api_key": "******",  // 验证脱敏
    "temperature": 0.7
  }
}
```

### 3.2 热加载验证
1.  初始 `temperature` 为 0.7。
2.  发送 POST 请求修改为 1.2。
3.  立即发起 LLM 请求。
4.  **观察**: LLM 回复的随机性明显增加，证明参数已实时生效。

## 4. 结论
Config API 功能完备，权限控制严格，热加载机制工作正常。敏感信息处理符合安全规范。
