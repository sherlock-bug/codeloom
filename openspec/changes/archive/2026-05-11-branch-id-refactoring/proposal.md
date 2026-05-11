# Proposal: branch-id-refactoring

## Why

当前分支存储三个地方各自为政：
- `nodes.branch_name`：1,935 个 sym 节点为 `''`，394 个 doc 节点为 `'main'`
- `edges.branch_name`：全部为 NULL
- `branches.branch_name`：全部为 `'master'`

三者互不一致。同时 `edges` 的 UNIQUE 索引 `(source_id, edge_type, source_repo)` 不含分支维度，多分支索引时不同分支的同源边会相互丢弃。

## What Changes

引入 `branch_meta` 表作为分支 ID 主表，所有分支引用改为 INTEGER ID：

- **新增** `branch_meta(id, repo, branch_name)` 表
- `nodes.branch_name TEXT` → `nodes.branch_id INTEGER NOT NULL DEFAULT 0`
- `edges.branch_name TEXT` → `edges.branch_id INTEGER NOT NULL DEFAULT 0`
- `branches.branch_name TEXT` → `branches.branch_id INTEGER`
- `edges` UNIQUE 索引改为 `(source_id, edge_type, branch_id)`
- `nodes` UNIQUE 约束改为 `(content_hash, file_path, name, branch_id, repo)`
- 所有 INSERT 节点/边的代码，先通过 `INSERT OR IGNORE INTO branch_meta` 获取 ID 再写入

无历史包袱：直接改 schema，`codeloom clean --all` 后重新索引。

## Capabilities

### New Capabilities
- `branch-id-refactoring`: 分支存储从字符串改为整数 ID，统一引用 `branch_meta` 表

## Impact

- `src/storage/schema.rs` — 改 nodes/edges/branches 表定义 + 新增 branch_meta
- `src/storage/symbols.rs` — Symbol::insert() 改用 branch_id
- `src/storage/nodes.rs` — INSERT 改用 branch_id
- `src/storage/files.rs` — INSERT 改用 branch_id
- `src/storage/fts.rs` — INSERT 改用 branch_id
- `src/indexer/smart.rs` — 所有 branch_name 查询改为 branch_id JOIN
- `src/indexer/doc.rs` — 文档索引 INSERT 改用 branch_id
- `src/query/search.rs` — 搜索 WHERE 子句改为 branch_id
- `src/query/graph.rs` — 图查询改为 branch_id
- `src/mcp/handlers.rs` — MCP 工具分支参数解析
- `src/embedding/mod.rs` — 向量索引过滤
