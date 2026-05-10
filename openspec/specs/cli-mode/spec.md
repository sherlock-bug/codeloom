# cli-mode

## Purpose
CodeLoom cli-mode 功能域。本规范描述此功能的需求和行为。

## Purpose
提供命令行接口，支持单二进制同时运行 CLI 子命令和 MCP Server 模式，包含自动仓库/分支检测、一键安装脚本。纯 CLI 模式下，不带子命令时输出帮助信息，MCP 服务需通过显式子命令启动。

## Requirements

### Requirement: 单二进制同时支持 CLI 和 MCP
系统 SHALL 以单个二进制分发，同时支持 CLI 模式（通过子命令）和 MCP Server 模式（通过 mcp 子命令或自动检测）。
CLI SHALL 提供以下子命令：`index`、`status`、`branch`（set-alias / list-aliases）、`mcp`、`check`、`update`、`clean`、`completion`、`search`、`overview`、`list-symbols`、`get-definition`、`call-graph`、`list-repos`、`list-branches`。

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

### Requirement: 一键安装脚本
系统 SHALL 提供一键安装脚本，自动下载对应平台的二进制并完成配置。

#### Scenario: Linux x86_64 一键安装
- GIVEN curl 可用
- WHEN 执行安装脚本
- THEN codeloom 二进制下载到指定目录
- AND OpenCode 可注册 `opencode mcp add codeloom`
- AND 显示安装完成信息和快速上手命令

### Requirement: 查看索引状态
系统 SHALL 提供 CLI 命令查看当前项目的索引状态和统计信息。

#### Scenario: 查看索引状态
- GIVEN 项目已索引 main 和 feature 两个分支
- WHEN 执行 `codeloom status`
- THEN 输出符号数、边数、文档数、向量数、FTS5 索引状态和数据库大小

### Requirement: CLI 错误处理
系统 SHALL 对 CLI 命令的错误情况提供明确的错误信息，包含错误原因和建议操作。

#### Scenario: 在非项目目录执行索引
- GIVEN 当前目录没有任何源代码文件
- WHEN 执行 `codeloom index .`
- THEN 输出提示表明未检测到支持的源代码
- AND 建议切换目录后重试
