# Tasks

## 1. call_graph LIKE fallback 修复
- [x] 1.1 在 `src/query/call_graph.rs` LIKE 查询中加 `AND s.kind NOT IN ('string_literal', 'enum_value')`
- [x] 1.2 确认 exact match 路径（line 16-26）不受影响

## 2. 断边过滤（target_id=0）
- [x] 2.1 在 `src/query/graph.rs` `get_edges()` 的 forward SQL 加 `AND e.target_id != 0`
- [x] 2.2 在 reverse SQL 加 `AND e.source_id != 0`（对称保护）
- [x] 2.3 在 `build_forward_adj` 的 SQL 加 `AND target_id != 0`
- [x] 2.4 在 `build_reverse_adj` 的 SQL 加 `AND source_id != 0`

## 3. inheritance_tree kind 校验
- [x] 3.1 在 `src/mcp/mod.rs` 的 upward inherits SQL（line 521）加 `AND s.kind IN ('class', 'struct')`
- [x] 3.2 在 downward inherits SQL（line 539）加 `AND s.kind IN ('class', 'struct')`

## 4. 验证
- [x] 4.1 cargo test（77 passed: 65 unit + 2 ignored + 10 integration）
- [x] 4.2 build release
- [x] 4.3 leveldb 实测：call_graph LIKE ✅ / neighbor_graph 0空串 ✅ / inheritance_tree 0假阳性 ✅
- [x] 4.4 README 无需更新（修复内部逻辑，接口不变）
