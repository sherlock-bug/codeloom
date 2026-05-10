# Design: fix-search-ranking

## Context
CodeLoom v0.5.0 的混合搜索（hybrid_search）使用 RRF 融合 BM25 + vec0 向量结果，但存在 5 个结构 bug 导致搜索结果不可用。实测 leveldb（1972 符号）搜索 "DoCompactionWork" 精确函数名，该函数排第 6，分数 0.016，前 5 名全是文档节点和无关枚举值。

## Goals / Non-Goals
**Goals:**
- 精确函数名搜索排第一
- 文档节点不重复出现
- code 搜索结果带摘要
- 多词 OR 搜索 + kind 过滤

**Non-Goals:**
- 不改变 vec0 存储格式
- 不改变 MCP 工具接口（新增可选参数）
- 不引入新的 embedding 模型

## Decisions

### Decision 1: RRF 分数使用 min-max 归一化的 BM25 + cosine 组合
当前 RRF 使用纯排名（`1/(k+rank+1)`）丢弃了 BM25 的真实分数。改为：
- BM25 侧：保留原始 bm25() 值（负数，0=最佳），取 `1/(1+|score|)` 归一化到 (0,1]
- 向量侧：保留原始 cosine similarity，已经是 (0,1]
- 融合：`final = w_bm25 * norm_bm25 + w_vec * norm_vec`（权重各 0.5）
- 如果某侧无结果，权重全给另一侧

选择加权求和而非保持 RRF 的原因：RRF 适合"两边排序不一致需要折中"，但我们的 BM25 和向量相似度都是连续分数，加权求和更直观且区分度高。

### Decision 2: 文档节点名称统一
去掉 `run_vector_search` 中 `format!("📄 {}", title)` 的 emoji 前缀。改为在 hit_type 中区分 code/doc，format 层负责显示标记。

### Decision 3: 同名 section 防累加
在 RRF 融合的 HashMap 中，`name` 相同时只保留最高分而非累加。改用 `entry.score = entry.score.max(new_score)` 替代 `entry.score += new_score`。

如果同名 section 来自不同文件（如两个 repo 都有 README），用 `(name, file_path)` 替代 `name` 作为去重 key。

### Decision 4: FTS5 重建符号索引
`fts5_sym` 当前列：`name, file_path, signature`
改为：`name, file_path, signature, definition, kind`

需要 `DROP + CREATE` 重建虚拟表（FTS5 不支持 ALTER ADD COLUMN），`codeloom index` 时自动重建。

### Decision 5: kind 过滤作为 SQL WHERE 条件
不在 FTS5 MATCH 层面过滤，而在 SQL JOIN 时加 `AND s.kind = ?`。支持的 kind 值列表：`function, method, class, struct, enum, enum_value, field, global, static_var, variable`。`kind` 为可选参数，不传则不过滤。

## System Architecture
搜索管线（修改部分）：

```
codeloom_search(query, repo, branch, limit, kind?)
  │
  ├─ FTS5 BM25 (fts_sym + fts_doc)
  │    ├─ search_symbols: + definition/kind 列, + kind WHERE 过滤
  │    └─ search_docs: 不变
  │
  ├─ vec0 ANN (symbol_vec + doc_vec)
  │     ├─ KNN search: 不变
  │     └─ name: 去掉 📄 前缀
  │
  └─ RRF fuse
       ├─ BM25: 1/(1+|score|) 归一化
       ├─ vec: cosine similarity (已有)
       ├─ merge: weighted sum, key=(name, file_path), 取 max 非累加
       └─ code results: + snippet (definition 前200字符)
```

## API Design
### codeloom_search 参数变化
```json
{
  "query": "compaction",
  "repo": "leveldb",
  "branch": "main1",
  "limit": 10,
  "kind": "function"  // 新增可选参数
}
```

### 搜索结果格式变化
```
// 改前
[0.016] DBImpl::DoCompactionWork [code] @ ./db/db_impl.cc:898

// 改后
[0.087] DBImpl::DoCompactionWork [code|function] @ ./db/db_impl.cc:898
  snippet="void DBImpl::DoCompactionWork(CompactionState* compact) { mutex_.AssertHeld(); ..."
```

## Risks / Trade-offs
| 风险 | 缓解措施 |
|------|---------|
| FTS5 DROP+CREATE 重建耗时 | 1972 符号重建 <2s，可接受 |
| 加权求和权重选择影响排序 | 默认 0.5/0.5，可通过 config 调 |
| OR 语义增加结果数量 | limit 参数限流，BM25 自然排序高相关优先 |
| kind 过滤不传时行为不变 | 向后兼容 |
