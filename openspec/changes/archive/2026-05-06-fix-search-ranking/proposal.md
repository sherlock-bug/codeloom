# Proposal: fix-search-ranking

## Why
CodeLoom 的 hybrid search（BM25 + 向量 RRF 融合）在实际使用中完全不可用。精确搜索函数名返回的排名还不如随机枚举值，文档节点重复出现且分数虚高，大模型只能放弃 codeloom 改用 grep。根因是 RRF fusion 的 3 个结构 bug + FTS5 索引覆盖不足 + 查询语义不匹配 LLM 使用习惯。

## What Changes

1. **修复 RRF fusion 3 个结构 bug**：去掉 doc 名称的 `📄 ` 前缀使 BM25/vec 正确合并；同名 section 不再累加分数；RRF 使用 BM25/相似度的真实分数而非纯排名
2. **FTS5 符号索引扩展**：`fts5_sym` 增加 `definition` + `kind` 列，使函数体内容和类型可搜索；code 搜索结果返回 snippet
3. **kind 过滤支持**：`codeloom_search` MCP 工具新增可选 `kind` 参数，支持按符号类型过滤

## Capabilities

### New Capabilities
- `rrf-ranking-fix`: 修复 RRF 融合的 3 个 bug——去掉 `📄 ` 前缀、防同名 section 累加、用真实分数替代纯排名；新增 `kind` 过滤支持
- `fts5-enhanced-index`: FTS5 符号索引扩展 `definition`+`kind` 列，code 搜索结果返回前 200 字 snippet

## Impact
- **修改**：`src/query/search.rs`（RRF fusion）、`src/storage/fts.rs`（escape_fts5 + 索引列）、`src/mcp/mod.rs`（search handler + 参数 schema）、`src/storage/schema.rs`（fts5 表结构需重建）
- **重构**：现有 leveldb/bf/doc 等 repos 的 fts5 索引需重建（`codeloom index` 自动处理）
- **无 BREAKING**：MCP 工具 `codeloom_search` 新增可选参数 `kind`，旧调用方式不受影响
