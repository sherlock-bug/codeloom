# Design: remove-legacy-tables

## Context

当前索引流程：

```
indexer → symbols/doc_nodes/files (old tables)
    ↓ migrate_to_nodes()
nodes + branches (new tables)
    ↓ fill_all_fts()
fts5_all
    ↓ index_vectors() [reads from symbols, stores symbols.rowid in vec0]
symbol_name_vec_{repo}
```

搜索流程（向量）：

```
vec0 KNN → symbols.rowid
    ↓ JOIN symbols WHERE rowid IN (...)
symbols details
    ↓ JOIN nodes ON name+file_path+line_start (复合键，脆弱)
final result
```

目标流程：

```
indexer → nodes (direct write)
    ↓ fill_all_fts()
fts5_all
    ↓ index_vectors() [reads from nodes, stores nodes.id in vec0]
symbol_name_vec_{repo}
```

搜索流程（向量，新）：

```
vec0 KNN → nodes.id
    ↓ JOIN nodes ON nodes.id = rowid
final result (straight, no indirection)
```

## Goals / Non-Goals

- Goal: 移除 symbols/doc_nodes/files 旧表，索引器直写 nodes
- Goal: 向量表使用 nodes.id，搜索链路直连 nodes
- Goal: 图查询直查 nodes + edges
- Non-goal: 不改 edges 表结构（source_id/target_id 改为 references nodes.id，但类型不变）
- Non-goal: 不改 MCP 工具层接口（保持对外兼容）
- Non-goal: 不改 FTS5 搜索（已正确使用 nodes）

## Database Schema Changes

### Removed Tables
- `symbols` — 所有数据 → `nodes` (node_type='sym')
- `doc_nodes` — 所有数据 → `nodes` (node_type='doc')
- `files` — 所有数据 → `nodes` (node_type='file')
- `fts5_sym`, `fts5_doc`, `fts5_files` — 已由 fts5_all 统一覆盖

### Kept Tables (unchanged)
- `nodes` — 主表
- `edges` — 边表。source_id/target_id 语义从 "symbols.id" 变为 "nodes.id"
- `branches` — node_id 列已存在
- `fts5_all` + fts5_all_content + 影子表 — 不变
- `git_index_state` — 不变
- `branch_glossary` — 不变
- `doc_images` — 图片数据，保持引用完整性
- vec0 表 (`symbol_name_vec_{repo}`, `file_vec_{repo}`) — rowid 从 symbols.id 变为 nodes.id

### Schema Initialization Change
`src/storage/schema.rs` 中的 `run()` 函数：
- 移除 `symbols` 表的 CREATE TABLE
- 移除 `fts5_sym`, `fts5_doc` 的 CREATE 和迁移
- 移除 `migrate_v5`, `migrate_v6`, `migrate_v7`, `migrate_v8`, `migrate_v9`, `cleanup_v9`
- `branches` 表直接使用 `node_id` 列（不再经过 symbol_id → node_id 迁移）
- `edges` 表不变（但 source_id/target_id 语义变更为 nodes.id）

## Decisions

### Decision: Symbol.insert() → nodes 表
选择将 `Symbol` struct 的 `insert()` 方法改写为写入 `nodes` 表而非 `symbols` 表。
- `node_type` = 'sym'
- `content` = `kind + ' ' + doc_comment`（保持与现有 FTS5 兼容）
- `attrs` = JSON 对象包含 signature, namespace, access, is_virtual, is_definition, is_external, template_args, language, parent_class, line_end
- 返回值（新行 ID）保持不变类型 `i64`

### Decision: resolve_target 查 nodes 而非 symbols
`smart.rs` 中 `resolve_target()` 的 5 个 DB 查询全部改为 `SELECT id FROM nodes WHERE ... AND node_type='sym'`。id_map 的语义也从 symbols.id 变为 nodes.id。

### Decision: edges.source_id/target_id 引用 nodes.id
边插入时的 source_id 和 target_id 现在来自 `nodes.id`（原来自 `symbols.id`）。这不需要改表结构，只需要确保写入时传入的是 nodes.id。

### Decision: 向量索引从 nodes 读取
`index_vectors()` 原来读 `symbols` 表，改为读 `nodes` 表（`node_type='sym'` 且排除外部符号和特定 kind）。vec0 rowid 直接使用 `nodes.id`。

### Decision: 向量搜索直连 nodes
`vector_semantic_search()` 的 KNN 结果（原返回 symbols.rowid）现在直接 `JOIN nodes ON nodes.id = rowid`，不再经过 `symbols` 表。

### Decision: 简化 schema.rs
移除所有迁移函数。新的 `run()` 只创建需要的表：
- `nodes`（主表）
- `edges`
- `branches`（直接 node_id）
- `git_index_state`
- `branch_glossary`
- `doc_images`
- `fts5_all`

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|---------|
| 已有旧数据的用户无法升级（破坏性变更） | 这是 clean-slate 变更，已有 DB 需 `clean --all` 重建 |
| 边 resolve_target 查 nodes 可能变慢（表更大） | nodes 表有 idx_nodes_name 索引，且 node_type='sym' 过滤 |
| doc_images 的 doc_node_id 外键指向旧 doc_nodes.id | doc_images 使用独立 ID，图片通过查询关联到 nodes |
| edges 的 source_id/target_id 不再指向旧表 | 语义变更，但值类型不变（i64），代码兼容 |

## Migration Plan

无迁移。本次变更是破坏性的，用户需 `clean --all` 后重新索引。
