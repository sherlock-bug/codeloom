# mcp-server

## Purpose
CodeLoom mcp-server 功能域。本规范描述此功能的需求和行为。

## Purpose
实现 MCP 协议服务端，通过 stdio/HTTP 为 OpenCode 提供 9 个查询工具，支持分支过滤。单人远程部署，LLM 每次调用必须携带当前分支名。codeloom_index 工具不执行实际索引，引导用户使用 CLI。

## Requirements

### Requirement: MCP 协议兼容
系统 SHALL 实现标准的 MCP (Model Context Protocol) 服务端，支持 stdio 传输方式和 HTTP 远程模式。

#### Scenario: OpenCode 注册 MCP 服务
- GIVEN 用户执行 `opencode mcp add codeloom`
- WHEN OpenCode 启动并连接 codeloom MCP 服务
- THEN codeloom 以 stdio 模式启动并响应 MCP 初始化请求
- AND 返回所有可用工具列表

### Requirement: 提供核心查询工具
系统 SHALL 通过 MCP 暴露以下查询工具，每个工具的 `branch` 参数为必传：

| 工具名 | 功能 | branch 行为 |
|--------|------|------------|
| codeloom_list_symbols | 模糊搜索符号 | 必传，过滤到指定分支 |
| codeloom_get_definition | 获取符号完整定义 | 必传，过滤到指定分支 |
| codeloom_get_call_graph | 获取调用图 | 必传，过滤到指定分支 |
| codeloom_search | 混合搜索（BM25+向量） | 必传，过滤到指定分支 |
| codeloom_overview | 架构全貌 | 必传，过滤到指定分支 |
| codeloom_status | 索引状态 | 必传，过滤到指定分支 |
| codeloom_index | 索引引导 | 必传，返回CLI命令+状态 |
| codeloom_list_repos | 列出仓库 | 无额外参数 |
| codeloom_list_branches | 列出分支 | repo 参数必填 |

所有查询工具 SHALL 应用统一的分支过滤规则：`branch_name IS NULL` 的数据对所有分支可见；`branch_name` 有值的数据仅对匹配的分支可见。

#### Scenario: LLM 查询调用链（传入当前分支）
- GIVEN OpenCode 已连接 codeloom MCP 服务
- WHEN LLM 调用 codeloom_get_call_graph (参数: name="MyClass::render", branch="main")
- THEN 返回 main 分支上的调用图
- AND 同时包含 branch_name IS NULL 的符号

#### Scenario: 缺少 branch 参数返回错误
- GIVEN LLM 调用 codeloom_status 但未传 branch 参数
- WHEN MCP 服务处理该请求
- THEN 返回 JSON-RPC error (code: -32602, message: "branch is required")

### Requirement: codeloom_index 工具不执行实际索引
`codeloom_index` MCP 工具 SHALL 不执行实际索引操作，而是返回 CLI 命令字符串和状态指示。

#### Scenario: 调用 codeloom_index
- WHEN LLM 调用 `codeloom_index(path="/project", repo="codeloom", branch="main")`
- THEN 返回 "索引请用 CLI: codeloom index /project --repo codeloom --branch main。如果仓库已索引，请用 codeloom_list_repos 查看。"

#### Scenario: description 明确标注不执行索引
- WHEN LLM 阅读 `codeloom_index` 的 description
- THEN 能看到 "此工具不执行索引（索引需用 CLI 命令）" 的标注

### Requirement: 新增 list-repos 和 list-branches MCP 工具
MCP 工具列表 SHALL 增加 `codeloom_list_repos` 和 `codeloom_list_branches` 两个工具。

#### Scenario: tools/list 返回新增工具
- WHEN MCP 客户端执行 `tools/list`
- THEN 返回的 tools 数组包含 `codeloom_list_repos` 和 `codeloom_list_branches`，共 9 个工具

### Requirement: 统一搜索工具 codeloom_search
MCP 工具列表 SHALL 包含 `codeloom_search` 作为唯一代码搜索入口，整合 BM25 关键词搜索和 vec0 向量语义搜索。

#### Scenario: tools/list 只返回一个搜索工具
- WHEN MCP 客户端执行 `tools/list`
- THEN 返回的 tools 数组包含一个 `codeloom_search` 工具
- AND 不包含旧的分离式搜索工具
