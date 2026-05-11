# Tasks

## 1. Schema 改表结构

- [x] 1.1 在 `src/storage/schema.rs` 新增 `branch_meta` 表
- [x] 1.2 nodes：`branch_name TEXT` → `branch_id INTEGER NOT NULL DEFAULT 0`
- [x] 1.3 edges：`branch_name TEXT` → `branch_id INTEGER NOT NULL DEFAULT 0`
- [x] 1.4 branches：`branch_name TEXT` → `branch_id INTEGER`
- [x] 1.5 edges UNIQUE 索引改为 `(source_id, edge_type, branch_id)`
- [x] 1.6 nodes UNIQUE 约束改为 `(content_hash, file_path, name, branch_id, repo)`

## 2. 工具函数

- [x] 2.1 新增 `resolve_branch_id(conn, repo, branch_name) -> i64`，带内存缓存

## 3. 代码层适配

- [x] 3.1 `src/storage/symbols.rs` — Symbol::insert() 调用 resolve_branch_id
- [x] 3.2 `src/storage/nodes.rs` — INSERT 调用 resolve_branch_id
- [x] 3.3 `src/storage/files.rs` — INSERT 调用 resolve_branch_id
- [x] 3.4 `src/storage/fts.rs` — INSERT 调用 resolve_branch_id
- [x] 3.5 `src/indexer/smart.rs` — 所有 branch_name 查询改为 branch_id JOIN
- [x] 3.6 `src/indexer/doc.rs` — 文档索引适配 branch_id
- [x] 3.7 `src/embedding/mod.rs` — 向量索引查询适配 branch_id
- [x] 3.8 `src/query/search.rs` — 搜索 WHERE 子句适配 branch_id
- [x] 3.9 `src/query/graph.rs` — 图查询适配 branch_id
- [x] 3.10 `src/mcp/handlers.rs` — MCP 工具参数解析适配
- [x] 3.11 `src/cli/mod.rs` — status/show 命令适配

## 4. 验证

- [x] 4.1 `cargo build` 编译通过
- [x] 4.2 `cargo test` 全部通过
- [x] 4.3 `codeloom clean --all && codeloom index` leveldb 全流程跑通
- [x] 4.4 验证 nodes.branch_id、edges.branch_id、branches.branch_id 全部为有效值
- [x] 4.5 更新 README.md（如涉及）
