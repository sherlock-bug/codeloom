# Design: update-known-limitations

## Context

known-limitations spec 记录了 4 项 v0.9 遗留问题：

1. **向量索引用旧 symbols.rowid** → 实际代码中 `embedding/mod.rs` 已使用 `SELECT id FROM nodes`，vec0 行 ID 对应 `nodes.id`
2. **MCP 工具引用 doc_nodes 表** → `doc_nodes` 表已删除，MCP get_doc/query_excel 已 DISABLED
3. **Indexer 暂未直写 nodes 表** → `migrate_to_nodes` 已删除，所有写入直达 `nodes`
4. **边的 UNIQUE 索引缺分支维度** → `idx_ed_unique ON edges(source_id, edge_type, branch_id)` 已包含 `branch_id`

## Goals / Non-Goals

Goals:
- 4 项 known-limitations 标记 ✅ 已修复
- 保留场景描述但标注为历史记录

Non-Goals:
- 不删场景（保留审计轨迹）
- 不改代码

## Decisions

### Decision: 添加 ✅ 标记 + 修复说明 + 删除线旧偏差
参考已修复条目的样式（如模板实例去重、LLM 工具增强），在条目标题加 ✅，旧偏差描述加 `~~` 删除线，新增修复说明行。
