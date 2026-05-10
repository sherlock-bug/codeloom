# Proposal: 搜索排名优化 — 符号名与注释分离加权

## Why

当前搜索中，符号名和注释在 BM25（FTS5）和向量搜索（vec0）中共用同一权重通道。注释中出现关键词的符号（如 `RandomGenerator` 的注释含 "compression"）与符号名直接命中的结果（如 `Snappy_Compress`）排名不分上下。中文搜索英文代码时尤为严重——语义搜索匹配到注释中出现的英文关键词，而符号名是中文无关的，导致大量低价值结果排前。

之前的「子串命中 +0.3 boost」机制更进一步放大了这个问题。

## What Changes

1. **去掉子串 boost** — `weighted_fuse()` 中 `query_lower.contains(name)` 的 `+0.3`
2. **FTS5 符号搜索拆分为名搜索和注释搜索** — `search_symbols()` 拆为 `search_symbols_name()`（MATCH name+signature）和 `search_symbols_comment()`（MATCH doc_comment）
3. **向量搜索同步拆分** — 名嵌入向量与注释嵌入向量分开建索引、分开搜
4. **加权融合** — 名命中权重 0.7，注释命中权重 0.3（可配置），doc/file 保持等权

## Capabilities

### New Capabilities
- `search-name-comment-split`: FTS5 + 向量搜索按符号名和注释分通道搜索，名命中权重高于注释命中

### Modified Capabilities
- 无 — 这是内部搜索算法的改进，不影响 MCP/CLI 接口

## Impact

- **代码改动**：`src/query/search.rs`、`src/storage/fts.rs`、`src/storage/vector.rs`、`src/indexer/`（向量索引拆分）
- **索引重建**：需要重建向量索引（新增注释独立向量表）
- **向后兼容**：MCP/CLI 接口不变，输出格式不变
