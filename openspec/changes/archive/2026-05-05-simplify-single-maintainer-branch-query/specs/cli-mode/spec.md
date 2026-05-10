# Delta for cli-mode

## MODIFIED Requirements

### Requirement: 单二进制同时支持 CLI 和 MCP
系统 SHALL 以单个二进制分发，同时支持 CLI 模式（通过子命令）和 MCP Server 模式（通过 mcp 子命令或自动检测）。
CLI SHALL 提供以下子命令：`index`、`status`、`branch`（set-alias / list-aliases）、`mcp`、`check`、`update`、`clean`、`completion`。

#### Scenario: CLI 方式执行索引
- GIVEN codeloom 二进制已安装
- WHEN 执行 `codeloom index /path/to/project --branch main`
- THEN 开始索引该项目的代码符号
- AND 输出进度和索引统计

#### Scenario: MCP 方式启动服务
- GIVEN codeloom 二进制已安装
- WHEN 执行 `codeloom mcp`
- THEN 以 stdin/stdout 方式启动 MCP 服务端
- AND 等待并响应 MCP 协议消息

## REMOVED Requirements

### Requirement: Pull/Push/SwitchBranch CLI 命令
**Reason**: 项目改为单人维护模式，Pull/Push/SwitchBranch 功能不再需要。
**Migration**: 直接删除对应 CLI 子命令定义和实现代码。
