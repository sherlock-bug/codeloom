## Why

当前 CodeLoom 的索引流程走的是"写入旧表 → migrate_to_nodes 同步到新表"的双写路径，向量表存的是旧符号的 rowid，搜索链路需要 `symbols` → `nodes` 两次 JOIN 才能拿到完整数据。这不仅增加了代码复杂度和维护负担，还可能导致索引不一致（如前面发现的 `symbol_id` vs `node_id` 列名不匹配 bug）。迁移代码 (`migrate_to_nodes`) 和遗留旧表 (`symbols`, `doc_nodes`, `files`, `fts5_sym`, `fts5_doc`, `fts5_files`) 占据了大量代码量，却不再需要。

本次变更的目标是：**彻底移除所有旧表，索引器直接写入 `nodes` 表，向量表使用 `nodes.id`，搜索链路直接走 `nodes` + `fts5_all` + vec0，消除一切中转和双写。**

## What Changes

- **BREAKING**: 移除旧表 `symbols`, `doc_nodes`, `files` 及其对应的 FTS5 表 (`fts5_sym`, `fts5_doc`, `fts5_files`)
- **Indexer 直写 nodes**：Clang 解析器、tree-sitter 解析器、文档索引器直接 INSERT 到 `nodes` 表，不再经过 `symbols`/`doc_nodes`/`files`
- **边表直写 nodes.id**：`edges.source_id` / `target_id` 改为引用 `nodes.id`，resolve_target 所有查询改为查 `nodes` 表
- **向量索引直读 nodes**：`index_vectors()` 从 `nodes` 读取符号，以 `nodes.id` 作为 vec0 rowid
- **向量搜索直走 nodes**：`vector_semantic_search()` KNN 结果直接 JOIN `nodes` 表，去掉 `symbols` 中转
- **图查询直走 nodes**：`resolve_symbol_id()` / `symbol_name_by_id()` 改为查 `nodes` 表
- **Status 命令改查新表**：`codeloom status` 查 `nodes` + `fts5_all` 而非旧表
- **移除 migrate_to_nodes**：`schema.rs` 简化，去掉 v9 迁移逻辑和 cleanup
- **branches 表保持 node_id**：已修复的 `symbol_id → node_id` 列名变更保留，`INSERT INTO branches` 直接用 `nodes.id`

## Capabilities

### New Capabilities
- `direct-write-indexer`: 索引器直写 `nodes` 表，不再经旧表中转
- `direct-vector-search`: 向量搜索直连 `nodes` 表，使用 `nodes.id` 作为 vec0 rowid
- `direct-graph-query`: 图查询（调用图/继承树/邻接图）直查 `nodes` + `edges`

### Modified Capabilities（无 — 功能行为不变，只是表结构重构）

## Impact

- `src/storage/symbols.rs` — Symbol insert 改为写入 `nodes` 表；移除 `migrate_to_nodes` 相关代码
- `src/storage/fts.rs` — 移除 `migrate_to_nodes()` 各迁移函数；`fill_all_fts()` 不变
- `src/storage/schema.rs` — 简化，移除 v9 迁移（`migrate_v9`, `cleanup_v9`），不创建旧表
- `src/embedding/mod.rs` — `index_vectors()` 改为从 `nodes` 读取，使用 `nodes.id`
- `src/query/search.rs` — `vector_semantic_search()` 改为直查 `nodes`，不通过 `symbols`
- `src/query/graph.rs` — `resolve_symbol_id()` / `symbol_name_by_id()` 改为查 `nodes`
- `src/indexer/smart.rs` — `resolve_target()` 改为查 `nodes` 表
- `src/indexer/clang/mod.rs` — 边 INSERT 使用 `nodes.id`
- `src/cli/mod.rs` — status 命令改为查 `nodes` + `fts5_all`
- **不需要改动**: MCP 工具层（已通过 `query/` 模块查询）、FTS5 搜索（已直接 JOIN `nodes`）
