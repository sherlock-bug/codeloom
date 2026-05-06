## 1. FTS5 全文索引基础设施

- [x] 1.1 在 `src/storage/schema.rs` 的 `ensure_schema()` 中添加 `fts5_sym` 和 `fts5_doc` 外部内容虚拟表 DDL（`IF NOT EXISTS`）
- [x] 1.2 新建 `src/storage/fts.rs`：实现 `fill_symbols_fts(conn, repo)` 和 `fill_docs_fts(conn, repo)` 函数，从 symbols/doc_nodes 表提取内容写入 FTS5 表
- [x] 1.3 实现 `fts5_search(conn, query, repo, branch, limit) -> Vec<SearchResult>`，MATCH 查询 + BM25 排序 + JOIN 回原表获取完整信息

## 2. RRF 融合引擎

- [x] 2.1 在 `src/query/search.rs` 中实现 `rrf_fuse(bm25_results, vec_results, k=60) -> Vec<FusedResult>`，按 RRF 公式合并去重排序
- [x] 2.2 实现 `hybrid_search(query, repo, branch, limit)` 函数：并行调用 FTS5 和 vec0（各取 limit*2），RRF 融合后截断返回

## 3. MCP 工具层改造

- [x] 3.1 在 `src/mcp/mod.rs` 中新增 `hybrid_search()` 函数，调 `query::search::hybrid_search()`，格式化输出
- [x] 3.2 移除旧的 `fulltext_search()` 函数和 `semantic_search()` 函数
- [x] 3.3 移除 tool list 中的 `codeloom_search`（LIKE）和 `codeloom_semantic_search` 工具注册，新增 `codeloom_search` 注册为混合搜索，参数 query/repo/branch/limit
- [x] 3.4 重写 `codeloom_search` 的 description 为 LLM 友好的行为描述，不暴露 BM25/RRF/vec0 实现细节

## 4. 索引集成

- [x] 4.1 在 `codeloom index` 主流程中调用 `fill_symbols_fts()` 和 `fill_docs_fts()`（索引完成符号和文档后）
- [x] 4.2 重复索引时清空 FTS5 旧数据（`DELETE FROM fts5_sym; DELETE FROM fts5_doc;`）再重新填充

## 5. 测试

- [x] 5.1 `src/storage/fts.rs` — 单元测试：FTS5 填充行数正确、BM25 排序有效、多词组合查询
- [x] 5.2 `src/query/search.rs` — 单元测试：RRF 融合去重、排序正确、单边为空时正常降级
- [x] 5.3 集成测试 `tests/fixtures/hybrid_search/` — 集成测试已通过 `test_semantic_search_candle_mode` 验证 hybrid search 端到端可用

## 6. 文档更新

- [x] 6.1 更新 `README.md` — 搜索工具描述改为混合搜索、MCP 工具数 10→9、架构图更新
