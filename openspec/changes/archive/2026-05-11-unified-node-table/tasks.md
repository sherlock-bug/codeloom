# Tasks

## 1. Schema migration (v0.9)

- [x] 1.1 `src/storage/schema.rs`: 添加 `migrate_v9()` — 创建 nodes 表，迁移 symbols/doc_nodes/files 数据，迁移 doc parent_id → edges contains，DROP 旧表，重建 FTS5
- [x] 1.2 将 `branches.symbol_id` 重命名为 `branches.node_id`，更新相关引用
- [x] 1.3 `ensure_schema()` 中初始创建 nodes 表 + fts5_all（替代旧的 fts5_sym/fts5_doc）

## 2. 统一存储层

- [x] 2.1 `src/storage/nodes.rs`: 新增 — Node struct（代替 Symbol），insert/update/query 函数
- [x] 2.2 `src/storage/symbols.rs`: 标记 deprecated，所有调用迁移到 nodes.rs
- [x] 2.3 `src/storage/fts.rs`: `fill_all_fts()` 从 nodes 表填充 fts5_all

## 3. Indexer 适配

- [ ] 3.1 `src/indexer/clang/`: 写入 nodes 表（node_type='sym'），attrs 存 signature/namespace/is_external 等
- [ ] 3.2 `src/indexer/tree_sitter/`: 同上
- [ ] 3.3 `src/doc/mod.rs`: 写入 nodes 表（node_type='doc'），attrs 存 level/section_path
- [ ] 3.4 `src/storage/files.rs`: 写入 nodes 表（node_type='file'），attrs 存 file_type

## 4. Query 层适配

- [x] 4.1 `src/query/search.rs`: bm25_precise_search + hybrid_search 改为 `fts5_all JOIN nodes ON id=rowid`，归一化公式修正
- [x] 4.2 `src/query/graph.rs`: call-graph / inheritance / neighbor / impact 查询改为读 nodes 表
- [x] 4.3 `src/query/repo.rs`: list_repos / list_branches 改为读 nodes 表

## 5. MCP / CLI 适配

- [x] 5.1 `src/mcp/mod.rs`: 所有工具改为读写 nodes 表，tool descriptions 更新
- [x] 5.2 `src/cli/mod.rs`: index / search / status / check 输出适配
- [x] 5.3 `src/calib/mod.rs`: 标定查询适配 nodes 表

## 6. edges 文档层级

- [ ] 6.1 索引文档时：对每个 section 的 chunk 子节点创建 `contains` 边
- [ ] 6.2 graph query 支持 `contains` 边类型

## 7. 测试

- [x] 7.1 `cargo test` 全量通过（适配所有测试到 nodes 表）
- [x] 7.2 集成测试：clean → index leveldb → search "compress" 验证 code 排在 doc 前

## 8. 文档

- [ ] 8.1 README.md 更新 schema 描述
- [ ] 8.2 更新 openspec specs 中引用 symbols/doc_nodes/files 的部分
