# CodeLoom 全面规格

> 生成时间：2026-05-12
> 目的：系统性记录 CodeLoom 的所有功能规格，方便快速查阅，区分 feature vs bug

## 目录

| 文档 | 内容 | 条目数 |
|------|------|--------|
| [01-mcp-tools.md](./01-mcp-tools.md) | MCP 工具规格 | 12 active |
| [02-cli-commands.md](./02-cli-commands.md) | CLI 命令规格 | 16 条命令 + 2 子命令 |
| [03-query-internals.md](./03-query-internals.md) | 内部查询能力 | 14 函数/结构体 |
| [04-indexing-internals.md](./04-indexing-internals.md) | 索引能力 | 13 模块 |
| [05-design-vs-impl.md](./05-design-vs-impl.md) | 设计与实现偏差 | 已清零 |
| [06-known-issues.md](./06-known-issues.md) | 已知问题（Bug） | 7

## 快速索引

### MCP 工具（12 active）
`list_symbols` · `get_call_graph` · `search` · `semantic_search` · `inspect` · `list_repos` · `list_branches` · `schema` · `path_analysis` · `impact_analysis` · `neighbor_graph` · `inheritance_tree`

### CLI 命令（16 active）
`index` · `status` · `mcp` · `check` · `branch (set-alias/list-aliases)` · `completion` · `update` · `clean` · `list-repos` · `list-branches` · `search` · `semantic` · `overview` · `list-symbols` · `inspect` · `call-graph`

### 测试用例
CLI 端到端测试 → `tests/e2e-cli-tests.md`（30 用例）
MCP 端到端测试 → `tests/e2e-mcp-tests.md`（28 用例）
