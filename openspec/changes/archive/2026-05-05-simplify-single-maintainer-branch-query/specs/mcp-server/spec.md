# Delta for mcp-server

## MODIFIED Requirements

### Requirement: 提供核心查询工具
系统 SHALL 通过 MCP 暴露以下查询工具，每个工具的 `branch` 参数为必传：

| 工具名 | 功能 | branch 行为 |
|--------|------|------------|
| codeloom_list_symbols | 模糊搜索符号 | 必传，过滤到指定分支 |
| codeloom_get_definition | 获取符号完整定义 | 必传，过滤到指定分支 |
| codeloom_get_call_graph | 获取调用图 | 必传，过滤到指定分支 |
| codeloom_search | 全文搜索 | 必传，过滤到指定分支 |
| codeloom_semantic_search | 语义搜索 | 必传，过滤到指定分支 |
| codeloom_overview | 架构全貌 | 必传，过滤到指定分支 |
| codeloom_status | 索引状态 | 必传，过滤到指定分支 |
| codeloom_index | 增量索引 | 必传，指定目标分支 |

所有查询工具 SHALL 应用统一的分支过滤规则：`branch_name IS NULL` 的数据（索引时未指定分支）对所有分支可见；`branch_name` 有值的数据仅对匹配的分支可见。

#### Scenario: LLM 查询调用链（传入当前分支）
- GIVEN OpenCode 已连接 codeloom MCP 服务
- WHEN LLM 调用 codeloom_get_call_graph (参数: name="MyClass::render", branch="main")
- THEN 返回 main 分支上的调用图
- AND 同时包含 branch_name IS NULL 的符号（如果有）
- AND 响应时间 < 500ms

#### Scenario: 文档在所有分支可见
- GIVEN 某文档在索引时 branch_name 为 NULL
- WHEN LLM 在任意分支（如 "feature-x"）执行 codeloom_semantic_search
- THEN 该文档出现在搜索结果中

## REMOVED Requirements

### Requirement: 提供索引管理工具（pull/push/switch-branch）
**Reason**: 项目改为单人维护模式，不再需要 Pull/Push/SwitchBranch 功能。
**Migration**: 无。这些 MCP 工具从未被实际使用（仅占位实现）。

### Requirement: MCP 工具参数校验（分支不存在错误）
**Reason**: 分支校验逻辑简化。如果查询传入未索引的分支，返回空结果而非错误。
**Migration**: 无。原有分支错误提示改为返回空结果。
