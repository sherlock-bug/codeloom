# Design: 搜索排名优化 — 符号名与注释分离加权

## Context

当前搜索架构：
```
hybrid_search(query) {
    bm25_symbols = search_symbols(fts5_sym)  // name+sig+comment 混搜
    bm25_docs    = search_docs(fts5_doc)
    bm25_files   = search_files(fts5_files)
    vec_results  = run_vector_search(vec0)   // name+sig+comment 混嵌入
    fused = weighted_fuse(bm25_all, vec_results)
    boost_substring_match(fused)             // ← 要移除
}
```

问题：注释和符号名在同一权重通道竞争，注释命中的噪音符号排名过高。

## Goals / Non-Goals

**Goals:**
- 名命中权重 > 注释命中权重
- 中文搜索能定位到语义相关的英文符号名
- 去掉子串 boost

**Non-Goals:**
- 不改 MCP/CLI 接口
- 不改 doc/file 搜索
- 不改加权融合框架

## System Architecture

```
hybrid_search(query) {
    // FTS5 — 拆分
    bm25_name    = search_symbols_name(fts5_sym MATCH 'name + signature')
    bm25_comment = search_symbols_comment(fts5_sym MATCH 'doc_comment')
    bm25_docs    = search_docs(fts5_doc)
    bm25_files   = search_files(fts5_files)

    // 向量 — 拆分
    vec_name    = knn_search(symbol_name_vec_{repo}, query_emb)
    vec_comment = knn_search(symbol_comment_vec_{repo}, query_emb)

    // 融合（名权重 0.7，注释权重 0.3）
    fused = weighted_fuse_multi([
        (bm25_name,    vec_name,    0.7),  // 高权重
        (bm25_comment, vec_comment, 0.3),  // 低权重
        (bm25_docs,    vec_docs,    0.5),
        (bm25_files,   vec_files,   0.5),
    ])
}
```

## Database Design

### 新向量表

```sql
-- 替代 symbol_vec_{repo}，拆为两张表
CREATE VIRTUAL TABLE symbol_name_vec_{repo} USING vec0(
    embedding float[512]
);

CREATE VIRTUAL TABLE symbol_comment_vec_{repo} USING vec0(
    embedding float[512]
);
```

- `symbol_name_vec_{repo}` 的 rowid = symbols.rowid，嵌入 `name + signature` 文本
- `symbol_comment_vec_{repo}` 的 rowid = symbols.rowid，嵌入 `doc_comment` 文本
- doc_comment 为空的符号不插入注释表
- 删旧的 `symbol_vec_{repo}` 表

### FTS5 列过滤

FTS5 表 `fts5_sym(name, file_path, signature, kind, doc_comment)` 已有多列设计，可直接用列过滤：

```sql
-- 名搜索：MATCH name + signature 列
SELECT ... WHERE fts5_sym MATCH 'name:snappy OR signature:snappy' ...

-- 注释搜索：MATCH doc_comment 列
SELECT ... WHERE fts5_sym MATCH 'doc_comment:snappy' ...
```

## Decisions

### Decision: FTS5 用列过滤而非拆表
选择在已有 `fts5_sym` 表上用 MATCH 列过滤，而非创建 `fts5_sym_name` / `fts5_sym_comment` 两张新 FTS5 表。因为：
- 列过滤是 FTS5 原生支持，零额外存储
- 拆分表需重建所有符号数据，改动大
- 两表查询后 JOIN back 到 symbols 增加复杂度

### Decision: 向量表拆分而非列过滤
选择新建 `symbol_name_vec_{repo}` 和 `symbol_comment_vec_{repo}` 两张表。因为 vec0 不支持列过滤，且嵌入文本内容和维度与 FTS5 不同——名嵌入只需 name+sig，注释嵌入只需 doc_comment。

### Decision: 名权重 0.7 vs 0.6
选 0.7/0.3 而非 0.6/0.4。因为用户的主要使用场景是精确搜符号名或中文语义搜英文名，注释应该是辅助信号。0.7 确保 3 个名命中就能压制 7 个注释命中。

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|---------|
| doc_comment 为空的符号在注释搜索中缺失 | 名搜索覆盖这些符号，不影响召回 |
| 向量索引重建耗时长 | 增量索引，只重建符号向量（~5s for leveldb） |
| 拆分后代码复杂度增加 | weighted_fuse 支持多通道，扩展轻量 |
