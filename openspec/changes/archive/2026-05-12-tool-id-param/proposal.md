# Proposal: tool-id-param

## Intent
当前所有工具通过字符串 `name` 定位符号，但实际数据中存在大量重名（`TEST_F` 17 个、`main` 7 个），LLM 无法精确指定。需要让搜索工具返回 `id`，非搜索工具接受 `id`+`name` 至少传一个。

## Scope
In scope:
- search / semantic_search / list_symbols：返回结果中附带符号 `id`
- inspect / call_graph / neighbor_graph / impact_analysis / path_analysis / inheritance_tree / get_doc：新增 `id` 参数，`id` 和 `name` 至少传一个
- `id` 传值时直接查 nodes.id，精度最高；`name` 传值时保持当前模糊匹配逻辑
- 更新所有受影响的 tool description

Out of scope:
- 不改 list_repos / list_branches / schema（它们不查符号）
- 不改搜索工具的 inputSchema（结果里加 id 不影响入参）

## Approach
纯 MCP 层改动，不动索引层。每个工具的 handle 函数入口检查：if id.is_some() → 按 id 查；else → 按 name 查（当前逻辑）。`required` 里不放 id 也不放 name，desc 写"id和name至少传一个"。
