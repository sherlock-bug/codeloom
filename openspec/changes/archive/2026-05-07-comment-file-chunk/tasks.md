# Tasks

## 1. Schema 迁移 (v7)

- [x] 1.1 `src/storage/schema.rs` — symbols 表加 doc_comment 列，migration 加 migrate_v7
- [x] 1.2 `src/storage/schema.rs` — doc_nodes 表加 parent_id 列 + 索引
- [x] 1.3 `src/storage/schema.rs` — 新建 files 表（repo, file_path, file_type, summary, content_hash, branch_name）
- [x] 1.4 `src/storage/schema.rs` — 新建 fts5_files FTS5 索引 (file_path, summary)
- [x] 1.5 `src/storage/schema.rs` — 新建 file_vec_* 向量表创建逻辑
- [x] 1.6 `src/storage/symbols.rs` — Symbol struct 加 doc_comment 字段 + 对应的 INSERT/UPDATE

## 2. 注释收集

- [x] 2.1 `src/indexer/queries/cpp.rs` — collect_comments() 结果存入 Symbol.doc_comment（所有 extract_* 函数）
- [x] 2.2 `src/indexer/queries/cpp.rs` — extract_decl() 的 collect_comments() 结果存入 Symbol.doc_comment
- [x] 2.3 `src/indexer/queries/cpp.rs` — 收集行内注释（节点右侧 `// ...`），存入 doc_comment
- [x] 2.4 `src/indexer/queries/cpp.rs` — 收集实现体内注释（tree-sitter comment 节点，`_ => {}` 改为匹配 comment）
- [x] 2.5 `src/storage/symbols.rs` — INSERT/ON CONFLICT 加 doc_comment 合并：`doc_comment = doc_comment || '\n' || excluded.doc_comment`
- [x] 2.6 `src/indexer/smart.rs` — 修改嵌入文本格式：`name | kind` → `name | kind | doc_comment`

## 3. FTS5 索引扩展

- [x] 3.1 `src/storage/fts.rs` — fill_symbols_fts() INSERT 加 doc_comment 列
- [x] 3.2 `src/storage/fts.rs` — fts5_sym 表重建（新增 doc_comment 列），migrate_v7 中处理
- [x] 3.3 `src/storage/fts.rs` — 实现 fill_files_fts()：填充 fts5_files
- [x] 3.4 `src/storage/fts.rs` — 实现 search_files()：FTS5 搜索文件节点

## 4. 文件节点

- [x] 4.1 `src/indexer/smart.rs` — 在索引流程结尾新增文件节点创建步骤
- [x] 4.2 `src/indexer/smart.rs` — 代码文件 summary = 所有相关符号的 doc_comment 拼接（去重）
- [x] 4.3 `src/indexer/smart.rs` — 文档文件 summary = ""（或首段摘要），file_type="doc"
- [x] 4.4 `src/storage/vector.rs` — 实现 index_file_vectors()：批量嵌入文件节点
- [x] 4.5 `src/embedding/mod.rs` — index_vectors() 或调用处增加文件向量步骤

## 5. 文档切分

- [x] 5.1 `src/doc/mod.rs` — 实现 split_at_punctuation() 切分函数
- [x] 5.2 `src/doc/mod.rs` — 实现 chunk_section()：将单个 DocSection 切分为多个 ≤500 字 chunk
- [x] 5.3 `src/doc/mod.rs` — 修改 write_doc_sections()：写入父节点 + chunk 子节点，设置 parent_id
- [x] 5.4 `src/doc/mod.rs` — 所有格式（MD/DOCX/PDF/XML/XLSX）在写入前通过统一切分器
- [x] 5.5 `src/doc/section.rs` — DocSection struct 加 parent_id 字段（可选）
- [x] 5.6 `src/storage/fts.rs` — fill_docs_fts() 只索引 chunk 节点和短段父节点（跳过有子节点的空内容父节点）

## 6. 搜索集成

- [x] 6.1 `src/query/search.rs` — run_vector_search() 增加文件向量搜索（file_vec_* 表）
- [x] 6.2 `src/query/search.rs` — FusedResult 支持 hit_type="file"
- [x] 6.3 `src/query/search.rs` — weighted_fuse() 去重 key 从 (name, file_path) 扩展为 (name, file_path, hit_type)
- [x] 6.4 `src/query/search.rs` — hybrid_search() 加入 BM25 文件搜索 + 文件向量搜索
- [x] 6.5 `src/query/search.rs` — FusedResult.snippet 回填 doc_comment（代码结果有注释时显示注释，无注释时为空）
- [x] 6.6 `src/mcp/mod.rs` — 搜索结果格式化支持 file 类型 + 注释 snippet
- [x] 6.7 `src/cli/mod.rs` — CLI 搜索结果输出支持 file 类型 + 注释 snippet

## 7. 测试

- [x] 7.1 test_comment_indexing — 端到端覆盖：中文/英文/注释搜索均通过
- [x] 7.2 test_file_node_fts — 端到端覆盖：文件 FTS5 搜索命中（搜 "thread safe" 命中 async.h 等文件节点）
- [x] 7.3 test_file_node_vector — 端到端覆盖：文件向量搜索命中
- [x] 7.4 新增 test_chunk_punctuation — 验证标点优先级切分
- [x] 7.5 新增 test_chunk_short — 验证短段不切
- [x] 7.6 新增 test_chunk_inheritance — 验证父节点属性继承

## 8. 端到端验证

- [x] 8.1 `cargo test` 全部通过（73 passed, 0 failed）
- [x] 8.2 `cargo build --release` 编译成功
- [x] 8.3 清库重验 spdlog：注释搜索（搜注释内容命中符号）、中文搜索、文档搜索全部通过
- [x] 8.4 文件搜索验证通过（搜文件名命中文件节点，带注释摘要）
- [x] 8.5 文档分段验证通过（所有 chunk ≤500 字，父子关系正确，大小文档均不出错）
- [x] 8.6 更新 README.md — 注释/文件/文档搜索 v0.5 功能说明
