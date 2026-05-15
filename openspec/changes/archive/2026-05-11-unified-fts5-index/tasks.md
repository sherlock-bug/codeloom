# Tasks

## 1. Schema 迁移

- [ ] 1.1 `src/storage/schema.rs`: 添加 v0.7 migration — DROP `fts5_sym`/`fts5_doc`/`fts5_files`，CREATE `fts5_all(source_type, name, content)`
- [ ] 1.2 `src/storage/schema.rs`: `ensure_schema()` 中自动检测旧表并执行 migration
- [ ] 1.3 创建测试 fixture 验证 migration 正确性（旧 DB → 新 schema 无报错）

## 2. FTS5 填充逻辑

- [ ] 2.1 `src/storage/fts.rs`: 重写 `rebuild_fts5()` 为填充 `fts5_all`（三源：sym/doc/file）
- [ ] 2.2 `src/storage/fts.rs`: 删除旧的 `populate_fts5_sym()`、`populate_fts5_doc()`、`populate_fts5_files()`
- [ ] 2.3 `src/storage/fts.rs`: 删除 `clear_fts5()`，改为 `DELETE FROM fts5_all`

## 3. 搜索函数重构

- [ ] 3.1 `src/storage/fts.rs`: 将 6 个分表搜索函数（search_symbols_name/comment、search_docs_title/content、search_files_name/summary）合并为 2 个：`search_fts5_name()` 和 `search_fts5_content()`
- [ ] 3.2 `src/query/search.rs`: `bm25_precise_search()` 改为调用新的 2 通道函数
- [ ] 3.3 `src/query/search.rs`: `hybrid_search()` 同步调整

## 4. CLI 与 MCP

- [ ] 4.1 `src/cli/mod.rs`: 确认 `codeloom search` 输出与新结果一致
- [ ] 4.2 `src/mcp/mod.rs`: 更新 MCP tool descriptions（如有引用旧搜索行为）
- [ ] 4.3 `cargo test` 全量通过

## 5. 集成验证

- [ ] 5.1 清空 leveldb DB，重新 `codeloom index` + `codeloom search "compress"`，确认代码符号排在文档之前
- [ ] 5.2 确认 `codeloom check` 报告 FTS5 状态正常
- [ ] 5.3 确认标题提取不可比问题已消除（doc 不再因 raw score 优势排到 code 前面）

## 6. 文档

- [ ] 6.1 更新 README.md 中 FTS5 相关描述和 MCP 工具数量
