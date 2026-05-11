# Tasks

## 1. storage/schema.rs 简化

- [ ] 1.1 移除 symbols 表 CREATE TABLE
- [ ] 1.2 移除 fts5_sym/fts5_doc CREATE TABLE
- [ ] 1.3 简化 branches 表直接使用 node_id（去掉旧的 symbol_id → node_id 迁移）
- [ ] 1.4 移除 migrate_v5, migrate_v6, migrate_v7, migrate_v8, migrate_v9, cleanup_v9 函数
- [ ] 1.5 git_index_state + branch_glossary + doc_images + fts5_all 保留不变

## 2. Symbol struct 改写为写入 nodes 表

- [ ] 2.1 `Symbol` struct 保留，`insert()` 方法改为 INSERT INTO nodes (repo, node_type='sym', name, content, ...) RETURNING id
- [ ] 2.2 `insert_builtins()` 改为写入 nodes 表
- [ ] 2.3 `populate_branches()` 查 nodes 而非 symbols
- [ ] 2.4 `make_sid()` 不变

## 3. storage/fts.rs 简化

- [ ] 3.1 移除 `migrate_to_nodes()` 函数（不再需要从旧表迁移）
- [ ] 3.2 `fill_all_fts()` 不变（已正确从 nodes 读取）
- [ ] 3.3 保留 `search_fts5_name/content()` 和 `search_hits()` 不变（已正确 JOIN nodes）
- [ ] 3.4 移除 backward-compat stubs（fill_symbols_fts/fill_docs_fts/fill_files_fts）
- [ ] 3.5 测试代码更新（直接插 nodes 而非 symbols）

## 4. 边表 resolve_target 改为查 nodes

- [ ] 4.1 `src/indexer/smart.rs` 中 `resolve_target()` 的 5 个 DB 查询全部改为 `SELECT id FROM nodes WHERE name=?1 AND repo=?2 AND node_type='sym'`
- [ ] 4.2 `id_map` 语义处理：传入的 id_map 现在是 nodes.id

## 5. 向量索引从 nodes 读取

- [ ] 5.1 `src/embedding/mod.rs` 中 `index_vectors()` 的 SELECT 改为 `SELECT id, name, kind, json_extract(attrs,'$.signature') FROM nodes WHERE repo=?1 AND node_type='sym' AND is_external=0 AND kind NOT IN ('template_instance','namespace')`
- [ ] 5.2 `index_file_vectors()` 改为查 nodes 表（node_type='file'）
- [ ] 5.3 vec0 INSERT 使用 nodes.id 作为 rowid

## 6. 向量搜索直连 nodes

- [ ] 6.1 `src/query/search.rs` 中 `vector_semantic_search()` 的 KNN 结果改为直接 JOIN nodes
- [ ] 6.2 去掉 symbols 表的中转 JOIN

## 7. 图查询改查 nodes

- [ ] 7.1 `src/query/graph.rs` 中 `resolve_symbol_id()` 改为 `SELECT n.id FROM nodes n JOIN branches b ON b.node_id = n.id WHERE n.name=?1 AND n.repo=?2 AND (b.branch_name=?3 OR b.branch_name IS NULL) AND n.node_type='sym'`
- [ ] 7.2 `symbol_name_by_id()` 改为 `SELECT name FROM nodes WHERE id=?1`

## 8. CLI status 命令改查新表

- [ ] 8.1 `src/cli/mod.rs` 中 status 的 `SELECT COUNT(*) FROM symbols` → `SELECT COUNT(*) FROM nodes WHERE repo=?1 AND node_type='sym'`
- [ ] 8.2 `SELECT COUNT(*) FROM doc_nodes` → `SELECT COUNT(*) FROM nodes WHERE repo=?1 AND node_type='doc'`
- [ ] 8.3 移除旧 FTS5 表计数（fts5_sym, fts5_doc）
- [ ] 8.4 `SELECT COUNT(*) FROM edges` 不变（仍有效）
- [ ] 8.5 vec0 表计数更新（symbol_name_vec_* 仍然有效，symbol_comment_vec_*/doc_vec_* 移除引用）
- [ ] 8.6 reset/reset --all 命令不再清理旧表

## 9. index 流程简化

- [ ] 9.1 CLI index handler 中移除 `migrate_to_nodes()` 调用
- [ ] 9.2 `index_includes()` 确认是否写旧表并修复
- [ ] 9.3 `fill_all_fts()` 保留

## 10. 验证

- [ ] 10.1 `cargo check` 通过（0 error, warnings 可接受）
- [ ] 10.2 索引 leveldb（clean → index） 成功
- [ ] 10.3 FTS5 搜索验证：`codeloom_search` 搜索 "compaction" 准确返回结果
- [ ] 10.4 向量搜索验证：`codeloom_semantic_search` 返回有效结果（无报错）
- [ ] 10.5 `cargo test` 全量通过

## 11. 更新 README

- [ ] 11.1 更新 README.md 反映新的表结构和架构
