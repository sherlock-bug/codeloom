# Delta for CLI Mode

## ADDED Requirements

### Requirement: 单二进制同时支持 CLI 和 MCP
系统 SHALL 以单个二进制分发，同时支持 CLI 模式（通过子命令）和 MCP Server 模式（通过 mcp 子命令或自动检测）。

#### Scenario: CLI 方式执行索引
- GIVEN rag 二进制已安装
- WHEN 执行 `rag index /path/to/project --branch main`
- THEN 开始索引该项目的代码符号
- AND 输出进度和索引统计

#### Scenario: MCP 方式启动服务
- GIVEN rag 二进制已安装
- WHEN 执行 `rag mcp`
- THEN 以 stdin/stdout 方式启动 MCP 服务端
- AND 等待并响应 MCP 协议消息

### Requirement: 一键安装脚本
系统 SHALL 提供一键安装脚本，自动下载对应平台的二进制并完成配置。

#### Scenario: Linux x86_64 一键安装
- GIVEN curl 可用
- WHEN 执行 `curl -sSL https://example.com/install-rag.sh | bash`
- THEN rag 二进制下载到 /usr/local/bin/rag
- AND OpenCode 自动注册 `opencode mcp add rag`
- AND 显示安装完成信息和快速上手命令

### Requirement: 查看索引状态
系统 SHALL 提供 CLI 命令查看当前项目的索引状态和统计信息。

#### Scenario: 查看索引状态
- GIVEN 项目已索引 main 和 feature 两个分支
- WHEN 执行 `rag status`
- THEN 输出类似：
  - 分支 main: 100,000 符号, 50,000 边, 索引时间: 2026-05-01
  - 分支 feature: +5,000 符号, 3,000 去重, 索引时间: 2026-05-02
  - 存储: 280MB (节省 53%)

### Requirement: CLI 错误处理
系统 SHALL 对 CLI 命令的错误情况提供明确的错误信息，包含错误原因和建议操作。

#### Scenario: 在非项目目录执行索引
- GIVEN 当前目录没有任何源代码文件
- WHEN 执行 `rag index .`
- THEN 输出错误 "当前目录未检测到支持的源代码文件"
- AND 建议 "请切换到项目根目录后重试"
