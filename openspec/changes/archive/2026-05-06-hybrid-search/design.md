## Context

CodeLoom 当前有两个独立的搜索工具：`codeloom_search`（LIKE '%query%' 子串匹配，无相关性排序）和 `codeloom_semantic_search`（vec0 ANN 向量搜索，余弦相似度排序）。两者互不融合——LLM 必须在两个工具之间做选择，选了关键词就丢失语义视角，选了语义就丢失精确匹配。

数据库已有 `symbols`（name/kind/file_path/line_start/signature）和 `doc_nodes`（title/section_path/content）表，但没有全文索引。SQLite 内置 FTS5 全文搜索引擎，零外部依赖，且支持 `bm25()` 相关性排序函数。

本次设计将两者统一为单个 `codeloom_search` MCP 工具，内部并行跑 BM25 FTS5 和 vec0 ANN，用 Reciprocal Rank Fusion（RRF）融合排名。

## Goals / Non-Goals

**Goals:**
- 创建 FTS5 虚拟表覆盖 symbols（名称+签名+文件路径）和 doc_nodes（标题+内容）
- 实现 RRF 融合算法合并 BM25 排名和 vec0 相似度排名
- 统一 MCP 工具为单个 `codeloom_search`，移除旧的 `codeloom_search`（LIKE）和 `codeloom_semantic_search`
- LLM 友好的工具描述——只描述行为，不暴露 BM25/RRF/vec0 实现细节
- FTS5 索引在 CLI `codeloom index` 时自动创建和填充
- 性能：混合搜索延迟控制在 <100ms（典型 2K-10K 符号规模）

**Non-Goals:**
- 不实现 fallback 模式（一个挂了用另一个）——两条路径同时跑，RRF 融合
- 不改变现有 embedding 流水线（bge-small-zh + vec0）
- 不改变现有 symbols/doc_nodes 表结构
- 不引入外部搜索引擎（Elasticsearch/Meilisearch）——全 SQLite 内完成

## Decisions

### Decision 1: FTS5 内容表 vs 外部内容 FTS5

选择 **外部内容 FTS5 表**（`CREATE VIRTUAL TABLE ... USING fts5(content, content_rowid=...)`），而非独立 FTS5 表。

理由：
- 不需要重复存储数据——symbols 和 doc_nodes 已有内容
- 外部内容模式：`fts5_sym(content='...', content_rowid=symbols.rowid)`，查询返回 `content_rowid` 直接 JOIN 回原表
- 独立表需要同步维护两份数据（INSERT/UPDATE/DELETE），容易产生不一致
- 外部内容表的大小远小于独立表（只存索引，不存原始内容）

### Decision 2: FTS5 索引内容

**symbols 表 FTS5 索引列：** `name` + `file_path`（+ `signature` 如有）
**doc_nodes 表 FTS5 索引列：** `title` + `section_path` + `content`

符号的 signature 字段包含函数签名（如 `bool Load(const std::string& path)`），对 BM25 关键词匹配很有价值——搜索 "Load" 或 "string" 都能命中。

### Decision 3: RRF (Reciprocal Rank Fusion) 而非加权分数融合

选择 RRF 而非加权分数组合。

理由：
- BM25 分数和 cosine 相似度的量纲不同，无法直接线性加权
- RRF 天然无量纲：`score = Σ 1/(k + rank)`，k=60（标准值）
- 不需要对分数做归一化处理
- 行业验证：Elasticsearch 8.x 的 hybrid search 默认使用 RRF

算法伪代码：
```
rrf(results_bm25, results_vec, k=60):
    scores = {}
    for rank, doc in enumerate(results_bm25):
        scores[doc.id] += 1/(k + rank + 1)
    for rank, doc in enumerate(results_vec):
        scores[doc.id] += 1/(k + rank + 1)
    return sorted(scores, by score desc)[:limit]
```

结果取两个列表的并集（去重），每个文档的 RRF 分数 = BM25 端贡献 + vec0 端贡献。两端都排名靠前的文档得分最高。

### Decision 4: MCP 工具数变化

移除 `codeloom_search`（LIKE）和 `codeloom_semantic_search`，新增统一 `codeloom_search`。

MCP 工具数：10 → 9。

新的 `codeloom_search`：
- 参数：`query` (必填), `repo` (必填), `branch` (必填), `limit` (默认10)
- 内部：并行执行 `fts5_search(query, repo, branch, limit*2)` + `vector_search(query, repo, branch, limit*2)` → RRF 融合 → 取 top limit
- limit*2 是为了给 RRF 更多候选，融合后再截断

### Decision 5: 工具描述风格

不暴露 BM25/FTS5/RRF/vec0 等技术实现细节。描述语言：
- "搜索代码符号和文档" 
- "同时理解精确命名和语义意图"
- "query 可以是中文描述或符号名"
- 附带示例：`query="用户认证"`, `query="AuthService"`

### Decision 6: 索引时机

FTS5 虚拟表的创建和填充在 `codeloom index` 命令中触发，与 vec0 向量索引同步进行。扩展 `schema.rs` 的 `ensure_schema()` 添加 FTS5 DDL，扩展索引主流程添加 FTS5 内容填充调用。

不新开单独命令（如 `codeloom index-fts5`），减少用户认知负担。

## Project Directory Structure

```
src/
├── storage/
│   ├── schema.rs      # + 新增 FTS5 虚拟表 DDL
│   ├── fts.rs         # 新建：FTS5 索引填充 + 查询
│   └── ...
├── query/
│   ├── search.rs      # 从 stub 变为 RRF 融合实现
│   └── ...
└── mcp/
    └── mod.rs         # 移除 fulltext_search() + semantic_search()，新增 hybrid_search()
```

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| FTS5 索引增加 index 时间 | 实测增量 <1s（2K-10K 符号），FTS5 是 SQLite 内置，极快 |
| RRF 参数 k 不合适 | k=60 是业界标准值，后续可配置化 |
| FTS5 中文分词不完美 | SQLite FTS5 默认按 Unicode 分词，中文按单字 tokenize；对符号名搜索（英文为主）影响小，中文查询走 vec0 为主 |
| 旧 reindex 需要重建 FTS5 | 首次升级需 `codeloom index` 重新跑，和 vec0 同步重建 |
