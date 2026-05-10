## Why

当前搜索分成两条独立的通路——`codeloom_search`（LIKE 子串匹配，无相关性排序）和 `codeloom_semantic_search`（vec0 向量 ANN）——各走各的，LLM 需要在两个工具之间做选择，且选了其中一个就丢了另一个的视角。关键词搜不到语义相关结果，语义搜不到精确符号名命中。需要一套混合检索，同时利用 BM25 关键词排序和向量语义相似度，用 RRF 融合成统一排名，对外只暴露一个工具。

## What Changes

- 创建 FTS5 全文索引虚拟表（覆盖 symbols 名称/签名 + doc_nodes 标题/内容），替代当前 naive LIKE 匹配
- 实现 Reciprocal Rank Fusion（RRF）融合算法，合并 BM25 排名和 vec0 向量相似度排名
- 统一 MCP 工具：移除 `codeloom_search`（旧 LIKE）和 `codeloom_semantic_search`，新增单个 `codeloom_search`，内部并行跑 BM25 + vec0 → RRF 融合 → 返回统一结果
- 写 LLM 友好的工具描述，不暴露实现细节（BM25/FTS5/RRF/vec0），只描述行为——"搜代码和文档，同时理解精确命名和语义意图"
- CLI 索引时自动创建 FTS5 表并填充内容

## Capabilities

### New Capabilities
- `fts5-fulltext-index`: 在 symbols 和 doc_nodes 上创建 FTS5 全文索引虚拟表，支持 BM25 相关性排序
- `hybrid-search-fusion`: RRF 融合算法，将 BM25 关键词排名和 vec0 向量语义排名合并为统一排序结果

### Modified Capabilities
- `mcp-server`: 移除 `codeloom_search`（LIKE）和 `codeloom_semantic_search` 两个 MCP 工具，新增单个 `codeloom_search` 混合搜索工具
- `mcp-tool-descriptions`: 更新 `codeloom_search` 描述为行为引导风格，不暴露实现

## Impact

- `src/mcp/mod.rs` — 替换 fulltext_search + semantic_search 两个函数为单个 hybrid_search
- `src/storage/schema.rs` — 新增 FTS5 虚拟表 DDL
- `src/storage/` — 新增 fts.rs（FTS5 索引填充和查询）
- `src/query/search.rs` — 从 stub 变为 RRF 融合实现
- `src/indexer/` — 索引时触发 FTS5 内容填充
- MCP 工具数：10 → 9（移除 2 个旧搜索工具，新增 1 个混合搜索工具）
