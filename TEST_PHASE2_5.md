# Phase 2.5 测试指南

## 概述

本文档提供 Phase 2.5（集成工具调用循环）的测试指南。Phase 2.5 完成了完整的工具调用流程集成，使得 MineClaw 能够通过 LLM 调用 MCP 工具并返回最终结果。

## 前置条件

- Rust 1.70+
- Node.js 18+（用于运行测试 MCP 服务器）
- OpenAI API Key（或兼容的 LLM API）

## 测试环境准备

### 1. 配置文件

复制配置模板并填写必要信息：

```bash
cp config/mineclaw_template.toml config/mineclaw.toml
```

编辑 `config/mineclaw.toml`：

```toml
[server]
host = "127.0.0.1"
port = 18789

[llm]
provider = "openai"
api_key = "your-openai-api-key-here"
base_url = "https://api.openai.com/v1"
model = "gpt-4o"  # 推荐使用支持工具调用的模型
max_tokens = 2048
temperature = 0.7

[mcp]
enabled = true

[[mcp.servers]]
name = "test-server"
command = "node"
args = ["./test-mcp-server.js"]
env = {}
```

### 2. 验证测试 MCP 服务器

测试服务器 `test-mcp-server.js` 已包含两个工具：
- `echo` - 回显输入消息
- `add` - 两个数字相加

验证服务器可以正常启动：

```bash
node test-mcp-server.js
```

应该看到输出：
```
[test-server] Starting...
```

按 `Ctrl+C` 停止服务器。

## 单元测试和集成测试

### 运行所有测试

```bash
cargo test
```

预期结果：
- 51 个单元测试全部通过
- 3 个集成测试全部通过

### 运行特定测试

```bash
# 只运行 MCP 相关测试
cargo test mcp::

# 只运行集成测试
cargo test --test mcp_integration
```

## 手动测试

### 1. 启动 MineClaw 服务器

```bash
cargo run
```

预期日志：
```
INFO Configuration loaded successfully
INFO MCP is enabled, starting 1 servers
INFO Starting MCP server server_name=test-server
[test-server] Starting...
INFO Successfully started MCP server: test-server
INFO MineClaw server listening on 127.0.0.1:18789
INFO Health check: http://127.0.0.1:18789/health
```

### 2. 健康检查

在另一个终端窗口：

```bash
curl http://127.0.0.1:18789/health
```

预期响应：
```
OK
```

### 3. 发送测试消息（无工具调用）

测试简单的对话，不涉及工具调用：

```bash
curl -X POST http://127.0.0.1:18789/api/messages \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Hello, who are you?"
  }'
```

预期响应：
```json
{
  "message_id": "uuid-here",
  "session_id": "uuid-here",
  "assistant_response": "I'm MineClaw, an AI assistant..."
}
```

### 4. 测试工具调用 - echo

发送消息让 LLM 使用 echo 工具：

```bash
curl -X POST http://127.0.0.1:18789/api/messages \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Please use the echo tool to echo this message: Hello from test!"
  }'
```

预期服务器日志（应包含）：
```
INFO Running tool coordinator
INFO Starting tool coordinator, message_count=1
DEBUG Available tools: 2
INFO LLM returned 1 tool calls
INFO Executing tool tool_name=echo
DEBUG Found tool server server_name=test-server tool_name=echo
DEBUG Tool execution succeeded tool_name=echo
INFO Tool coordinator finished, intermediate_messages=2
```

预期响应应包含：
```json
{
  "assistant_response": "Echoed: Hello from test!"
}
```

### 5. 测试工具调用 - add

发送消息让 LLM 使用 add 工具：

```bash
curl -X POST http://127.0.0.1:18789/api/messages \
  -H "Content-Type: application/json" \
  -d '{
    "content": "What is 40 + 2? Please use the add tool."
  }'
```

预期服务器日志（应包含）：
```
INFO Executing tool tool_name=add
```

预期响应应包含：
```json
{
  "assistant_response": "40 + 2 = 42"
}
```

### 6. 查看会话消息

使用之前返回的 `session_id` 查看完整的消息历史（包括工具调用和结果）：

```bash
curl http://127.0.0.1:18789/api/sessions/{session-id}/messages
```

预期响应应包含：
- 用户消息
- 工具调用消息（role: "tool_call"）
- 工具结果消息（role: "tool_result"）
- 最终助手回复

示例：
```json
{
  "messages": [
    {
      "id": "...",
      "session_id": "...",
      "role": "user",
      "content": "What is 40 + 2?",
      "timestamp": "...",
      "tool_calls": null,
      "tool_result": null
    },
    {
      "id": "...",
      "session_id": "...",
      "role": "tool_call",
      "content": "",
      "timestamp": "...",
      "tool_calls": [
        {
          "id": "...",
          "name": "add",
          "arguments": {"a": 40, "b": 2}
        }
      ],
      "tool_result": null
    },
    {
      "id": "...",
      "session_id": "...",
      "role": "tool_result",
      "content": "",
      "timestamp": "...",
      "tool_calls": null,
      "tool_result": {
        "tool_call_id": "...",
        "content": "42",
        "is_error": false
      }
    },
    {
      "id": "...",
      "session_id": "...",
      "role": "assistant",
      "content": "40 + 2 equals 42.",
      "timestamp": "...",
      "tool_calls": null,
      "tool_result": null
    }
  ]
}
```

### 7. 多轮对话测试

使用同一个 session_id 进行多轮对话：

```bash
curl -X POST http://127.0.0.1:18789/api/messages \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "previous-session-id",
    "content": "Now add 100 to that result."
  }'
```

验证 LLM 能够记住上下文并继续使用工具。

### 8. 多轮工具调用测试（单次请求内）

**当前实现说明**：在单次 API 请求中，`ToolCoordinator` 会自动进行多轮工具调用，直到 LLM 返回最终文本回复。最大迭代次数默认为 10 次。

**重要更新**：从 2026-03-03 开始，LLM 可以**同时返回文本内容和工具调用**。例如：
- LLM 可以说 "Okay, let me calculate 1+2 first..." 然后调用 add 工具
- 工具结果返回后，LLM 可以说 "The result is 3. Now let me calculate 2+3..." 然后继续调用工具

**测试场景**：让 LLM 依次计算多个加法

```bash
curl -X POST http://127.0.0.1:18789/api/messages \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Please calculate these sums one by one using the add tool: 1+2, 2+3, 3+4. After each calculation, tell me the result, then continue to the next one."
  }'
```

**预期行为**：
1. LLM 可能回复 "Okay, let me start with 1+2..." 同时调用 add 工具
2. 执行 add(1+2)，得到结果 "3"
3. LLM 可能回复 "1+2 = 3. Now let me calculate 2+3..." 同时调用 add 工具
4. 执行 add(2+3)，得到结果 "5"
5. LLM 可能回复 "2+3 = 5. Now let me calculate 3+4..." 同时调用 add 工具
6. 执行 add(3+4)，得到结果 "7"
7. LLM 返回最终文本回复总结所有结果

**查看完整消息历史**：
```bash
curl http://127.0.0.1:18789/api/sessions/{session-id}/messages
```

应该能看到：
- 多个 Assistant 消息（LLM 的文本回复）
- 多个 ToolCall 消息（工具调用）
- 多个 ToolResult 消息（工具结果）

**实现细节**：
- `LlmResponse` 结构体现在可以同时包含 `text` 和 `tool_calls`
- `ToolCoordinator` 会先保存文本消息（如果有），然后执行工具调用
- 这个流程会循环直到 LLM 只返回文本而没有工具调用

## 测试清单

### 基础功能
- [ ] 服务器可以正常启动
- [ ] 健康检查端点正常工作
- [ ] 简单对话（无工具调用）正常工作

### 工具调用
- [ ] LLM 能够识别并调用 echo 工具
- [ ] LLM 能够识别并调用 add 工具
- [ ] 工具执行结果正确返回
- [ ] 工具调用和结果保存到会话历史

### 会话管理
- [ ] 可以查看会话消息列表
- [ ] 消息历史包含所有类型（user, assistant, tool_call, tool_result）
- [ ] 多轮对话正常工作

### 错误处理
- [ ] 工具参数错误时能正确处理
- [ ] 超过最大迭代次数时返回错误
- [ ] MCP 服务器断开时能正确处理

## 故障排除

### 问题：MCP 服务器无法启动

**检查**：
1. Node.js 版本是否 >= 18
2. `test-mcp-server.js` 文件是否存在
3. 端口是否被占用

**解决**：
```bash
node --version
ls -la test-mcp-server.js
```

### 问题：LLM 不调用工具

**检查**：
1. 使用的模型是否支持工具调用（推荐 gpt-4o, gpt-3.5-turbo-1106+）
2. 提示词是否明确要求使用工具
3. 服务器日志中是否显示 "Available tools: 2"

**解决**：
- 尝试更明确的提示词，如 "Please use the echo tool to..."
- 检查 LLM 配置中的 model 字段

### 问题：测试超时

**检查**：
1. LLM API 响应时间
2. MCP 服务器响应时间

**解决**：
- 检查网络连接
- 考虑在测试中增加超时时间

## 性能测试（可选）

### 并发测试

使用多个并发请求测试服务器性能：

```bash
# 使用 ab 工具（Apache Bench）
ab -n 10 -c 2 -p test.json -T application/json \
  http://127.0.0.1:18789/api/messages
```

### 响应时间

测量单次请求的响应时间：

```bash
time curl -X POST http://127.0.0.1:18789/api/messages \
  -H "Content-Type: application/json" \
  -d '{"content": "Hello"}'
```

预期响应时间（包含工具调用）：2-10 秒（取决于 LLM API 响应时间）

## 清理

停止服务器：按 `Ctrl+C`

清理会话（可选）：服务器重启后内存中的会话会自动清除。

## 参考文档

- [phase2_target_temp.md](./phase2_target_temp.md) - Phase 2 完整计划
- [API 端点文档](#api-端点) - 详见下文

## 附录：API 端点参考

### POST /api/messages
发送消息并获取回复

**请求体：**
```json
{
  "session_id": "可选的会话ID（新建会话时省略）",
  "content": "消息内容",
  "metadata": {}
}
```

**响应：**
```json
{
  "message_id": "消息ID",
  "session_id": "会话ID",
  "assistant_response": "助手回复"
}
```

### GET /api/sessions/:id/messages
获取会话的消息列表

**响应：**
```json
{
  "messages": [
    {
      "id": "uuid",
      "session_id": "uuid",
      "role": "user|assistant|tool_call|tool_result",
      "content": "文本内容",
      "timestamp": "ISO8601",
      "tool_calls": [...],
      "tool_result": {...}
    }
  ]
}
```

### GET /health
健康检查

**响应：**
```
OK