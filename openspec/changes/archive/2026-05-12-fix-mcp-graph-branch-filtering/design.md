# Design: 图查询 MCP 工具分支过滤修复

## Context

`edges` 表有 `branch_id` 列，但所有图遍历函数在查询边时都不加 `branch_id` WHERE 条件：

```sql
-- 当前: 读取所有分支的边
SELECT source_id, target_id, edge_type FROM edges WHERE target_id != 0

-- 期望: 只读指定分支的边
SELECT source_id, target_id, edge_type FROM edges WHERE target_id != 0 AND branch_id = ?
```

当仓库有多个分支的索引时，比如 `main1` (branch_id=1) 和 `main` (branch_id=387)，图查询把两个分支的边混在一起返回。

## Goals

- 所有图查询按 `branch_id` 隔离，多分支索引互不污染
- 不需要改 MCP 工具的 inputSchema 或返回格式
- `branch_id` 参数从已存在的 repo+branch 参数解析得来

## Non-Goals

- 不改 `search` 和 `list_symbols` 工具（它们已经正确过滤）
- 不改 DISABLED 工具（status/get_doc/query_excel）
- 不优化性能（建全量邻接表而非 SQL 逐跳查询的瓶颈已存在）

## System Architecture

不改架构，只改数据通路：

```
MCP (handle_tool_call)  →  图遍历函数 (get_edges/build_adj/traverse_calls)
   已有 branch 参数            ↓ 新增 branch_id 参数
                           SQL: ... AND e.branch_id = ?
```

## Decisions

### Decision 1: branch_id 作为函数参数传递，而非在 SQL 中查询

`resolve_branch_id()` 已在 `run_vector_search` 和 `call_graph.rs` 中使用，效果良好。统一用整数 `branch_id` 做 SQL 参数（参数化查询，无注入风险），不用字符串 `branch_name`。

### Decision 2: build_forward_adj / build_reverse_adj 接收 branch_id

这两个函数当前构建全表邻接表（所有边加载到 HashMap）。修复后：
- 加 `branch_id: i64` 参数
- SQL 加 `AND (branch_id = ? OR branch_id = 0)`
- `branch_id = 0` 为兼容未指定分支的旧数据

这样 `bfs_path_search()`、`transitive_closure()`、`neighbor_map()` 自动继承过滤。

### Decision 3: get_edges() 接收 branch_id

被 `neighbor_map()` 和 `inspect_symbol()` 的边查询使用。直接加 SQL WHERE 子句。

### Decision 4: traverse_calls() 接收分支参数

当前已有 `branch: &str` 参数，改为 resolve 后传 `branch_id: i64`，SQL 加 `AND branch_id = ?`。

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|---------|
| `branch_id = 0` 回退不够 | 所有 `resolve_branch_id` 会 CREATE 记录，不会返回 0（除非 fail） |
| 全表邻接表 `build_forward_adj` 不过滤分支导致内存浪费 | 但修复后它不过滤，是在 build 时加 WHERE—所有边全量加载的问题已存在，本次不治 |
| 测试覆盖不足 | 已有 leveldb 索引 `main1`，修改后手动验证 `inspect Compaction` 返回正确边 |

## Project Directory Structure

无新增文件。纯修改 `src/query/graph.rs`（~400行）和 `src/query/call_graph.rs`（~137行）和 `src/mcp/mod.rs`（~1054行）。
