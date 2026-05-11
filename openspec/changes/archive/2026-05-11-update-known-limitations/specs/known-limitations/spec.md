# Delta for known-limitations

## MODIFIED Requirements

### Requirement: 向量索引用旧 symbols.rowid（v0.9 遗留）✅ 已修复
向量表（symbol_name_vec_*, file_vec_*）SHALL 使用 nodes.id 作为主键以消除对旧 symbols/files 表的依赖。
~~当前偏差：migrate_to_nodes 每次索引从旧表迁移数据到 nodes，vector KNN 返回旧 symbols.rowid 需 JOIN symbols 表映射。symbols/files 表因此不能删除。~~
修复后：embedding/mod.rs 已改用 `SELECT id FROM nodes` 获取 rowid，vec0 表以 `nodes.id` 为行 ID，KNN 搜索直接 JOIN nodes 表。旧表 symbols/doc_nodes/files 均已删除。

### Requirement: MCP 工具仍引用 doc_nodes 表（v0.9 遗留）✅ 已修复
MCP 文档查询工具（codeloom_get_doc 等）SHALL 从 nodes 表查询文档内容。
~~当前偏差：codeloom_get_doc/list_doc_nodes/get_doc_section 直接 SQL 查询 doc_nodes 表（id/title/section_path/content），doc_nodes 因此不能删除。~~
修复后：doc_nodes 表已删除，codeloom_get_doc 和 codeloom_query_excel 已 DISABLED（代码中注释掉）。

### Requirement: Indexer 暂未直写 nodes 表（v0.9 遗留）✅ 已修复
Indexer（Clang/tree-sitter/doc/files）SHALL 写入 nodes 表作为主存储。
~~当前偏差：indexer 写入旧表（symbols/doc_nodes/files），通过 migrate_to_nodes（DELETE+REPLACE）同步到 nodes。旧表作为暂存区。~~
修复后：migrate_to_nodes 已删除，所有 Indexer 直接 INSERT 到 nodes 表（node_type='sym'/'doc'/'file'）。旧表 symbols/doc_nodes/files/fts5_sym/fts5_doc/fts5_files 均已删除。

### Requirement: 边的 UNIQUE 索引缺失分支维度 ✅ 已修复
`edges` 表 SHALL 确保同一 repo 的不同分支之间的边不互相冲突。
~~当前偏差：`CREATE UNIQUE INDEX idx_ed_unique ON edges(source_id, edge_type, source_repo)` 不包含 `branch_name` 列。~~
修复后：`idx_ed_unique ON edges(source_id, edge_type, branch_id)` 已包含 `branch_id` 列。
