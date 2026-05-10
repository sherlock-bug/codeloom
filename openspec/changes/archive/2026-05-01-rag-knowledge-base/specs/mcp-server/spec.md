# Delta for MCP Server

## ADDED Requirements

### Requirement: MCP 协议兼容
系统 SHALL 实现标准的 MCP (Model Context Protocol) 服务端，支持 stdio 传输方式。

#### Scenario: OpenCode 注册 MCP 服务
- GIVEN 用户执行 `opencode mcp add rag`
- WHEN OpenCode 启动并连接 rag MCP 服务
- THEN rag 以 stdio 模式启动并响应 MCP 初始化请求
- AND 返回所有可用工具列表

### Requirement: 提供 10 个核心查询工具
系统 SHALL 通过 MCP 暴露至少以下工具：

| 工具名 | 功能 |
|--------|------|
| rag_list_symbols | 模糊搜索符号 |
| rag_get_definition | 获取符号完整定义 |
| rag_get_call_graph | 获取调用图 |
| rag_get_inheritance | 获取继承链 |
| rag_find_references | 查找所有引用 |
| rag_get_file_symbols | 文件内全部符号 |
| rag_get_dependency_graph | 模块级依赖 |
| rag_impact_analysis | 修改影响分析 |
| rag_search | 全文搜索 |
| rag_overview | 架构全貌 |

#### Scenario: LLM 查询调用链
- GIVEN OpenCode 已连接 rag MCP 服务
- WHEN LLM 调用 rag_get_call_graph tool (参数: name="MyClass::render", branch="main")
- THEN 返回 JSON 格式的调用图（callers + callees）
- AND 响应时间 < 500ms

### Requirement: 提供索引管理工具
系统 SHALL 通过 MCP 暴露索引管理工具，允许 LLM 触发索引操作。

#### Scenario: LLM 触发增量索引
- GIVEN 用户在 OpenCode 中说 "索引一下当前项目"
- WHEN LLM 调用 rag_index tool (参数: path=".", branch="feature-x")
- THEN 系统执行增量索引
- AND 返回索引结果（新增符号数、修改符号数、耗时）

### Requirement: MCP 工具参数校验
系统 SHALL 校验所有 MCP 工具调用参数，无效参数返回明确错误信息。

#### Scenario: 传入不存在的分支名
- GIVEN 项目只索引了 main 分支
- WHEN LLM 调用 rag_get_definition (参数: name="foo", branch="nonexistent")
- THEN 返回错误 "分支 nonexistent 未索引，可用分支: [main]"
