# Delta for mcp-server

## MODIFIED Requirements

### Requirement: 提供核心查询工具
系统 SHALL 通过 MCP 暴露 8 个查询工具（不变）。`codeloom_status` 和 `codeloom_overview` 的统计输出 SHALL 不再包含 Doc-Code links 行。

#### Scenario: status 输出不含 links
- GIVEN 项目已索引
- WHEN LLM 调用 codeloom_status(branch="main")
- THEN 输出包含 Symbols、Edges、Docs、DB size
- AND 不包含 "Doc-Code links" 行
