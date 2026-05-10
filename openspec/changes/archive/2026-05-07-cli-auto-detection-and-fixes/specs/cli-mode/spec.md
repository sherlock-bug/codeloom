# Delta for cli-mode

## MODIFIED Requirements

### Requirement: 无子命令默认行为
系统 SHALL 在 `codeloom` 不带任何子命令时输出帮助信息，而非直接启动 MCP 服务。MCP 服务 SHALL 仅通过显式 `codeloom mcp` 子命令启动。

#### Scenario: 不带参数运行
- GIVEN codeloom 二进制已安装
- WHEN 执行 `codeloom`（无任何子命令）
- THEN 系统 SHALL 输出 `--help` 内容并退出，不启动 MCP

#### Scenario: 显式启动 MCP
- GIVEN codeloom 二进制已安装
- WHEN 执行 `codeloom mcp`
- THEN 系统 SHALL 启动 MCP JSON-RPC 服务
