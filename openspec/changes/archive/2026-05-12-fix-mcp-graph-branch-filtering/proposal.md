## Why

所有图分析类 MCP 工具（call_graph、path_analysis、impact_analysis、neighbor_graph、inheritance_tree、inspect_symbol）在查询 `edges` 表时不加 `branch_id` 过滤。当仓库有多个分支的索引时，边数据混在一起返回，导致结果不准确。

## What Changes

- 给 `edges` 表的所有查询统一加 `branch_id` 过滤
- 影响 6 个 MCP 工具 + 3 个底层图遍历函数
- 不涉及 API 接口变化，不改返回格式

## Capabilities

### New Capabilities
（无新能力，纯修复）

### Modified Capabilities
- `mcp-tools`: 所有图分析工具在边查询中 SHALL 按 branch_id 过滤

## Impact

受影响的源码位置：

| 文件 | 函数/位置 | 影响 |
|------|----------|------|
| `src/query/graph.rs` | `get_edges()` | 加 branch_id 参数 |
| `src/query/graph.rs` | `build_forward_adj()` / `build_reverse_adj()` | 加 branch_id 参数 |
| `src/query/graph.rs` | `get_terminal_deps()` | 加 branch_id 过滤 |
| `src/query/call_graph.rs` | `traverse_calls()` | 加 branch_id 过滤 |
| `src/mcp/mod.rs` | `inspect_symbol()` 的边查询 | 加 branch_id 过滤 |
| `src/mcp/mod.rs` | `inheritance_tree()` 的 build_tree/get_overrides | 加 branch_id 过滤 |
| `src/mcp/mod.rs` | `path_analysis()`/`impact_analysis()`/`neighbor_graph()` | 无改动（参数一路透传） |

`resolve_symbol_id()` 已正确使用 branch_id，不需要改。
