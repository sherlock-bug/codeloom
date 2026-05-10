# Delta for mcp-server

## ADDED Requirements

### Requirement: MCP 工具列表自动化验证
系统 SHALL 提供自动化测试验证 MCP tools/list 返回 8 个工具且每个工具的 branch 参数为必传。

#### Scenario: tools/list 返回 8 个工具
- GIVEN codeloom mcp 进程运行
- WHEN 发送 tools/list JSON-RPC 请求
- THEN 返回 8 个工具
- AND 每个工具的 inputSchema.required 包含 "branch"

#### Scenario: 缺 branch 参数返回错误
- GIVEN codeloom mcp 进程运行
- WHEN 调用 codeloom_status 不传 branch 参数
- THEN 返回 JSON-RPC error code=-32602 message="branch is required"
