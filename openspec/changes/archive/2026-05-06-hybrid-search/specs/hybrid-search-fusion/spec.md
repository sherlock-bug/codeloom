## ADDED Requirements

### Requirement: RRF 融合 BM25 和向量排名
系统 SHALL 使用 Reciprocal Rank Fusion 算法将 BM25 FTS5 搜索结果和 vec0 向量搜索结果合并为统一排序列表。

#### Scenario: 两端都有结果
- WHEN `bm25_results = [(sym_id=1, bm25=2.5), (sym_id=3, bm25=1.8)]` 且 `vec_results = [(sym_id=2, sim=0.91), (sym_id=1, sim=0.85)]`
- THEN RRF 融合后 sym_id=1 得分最高（两边都出现）
- AND sym_id=2 和 sym_id=3 按其各自的单边排名计入 RRF 分

#### Scenario: 只有一端有结果
- WHEN BM25 返回空结果但 vec0 返回结果
- THEN 使用 `1/(k + vec_rank + 1)` 单独计算 RRF 分
- AND 结果按 RRF 分降序排序

#### Scenario: RRF 参数 k 使用标准值
- WHEN 执行 RRF 融合
- THEN k=60 作为默认参数
- AND RRF 分计算公式为 `score(d) = Σ 1/(60 + rank_i(d))`，其中 rank_i(d) 从 0 开始计数

### Requirement: 混合搜索统一入口 `codeloom_search`
MCP 工具 `codeloom_search` SHALL 内部并行执行 BM25 和向量搜索，RRF 融合后返回统一结果。

#### Scenario: 并行执行两条搜索路径
- WHEN LLM 调用 `codeloom_search(query="compaction", repo="leveldb", branch="main", limit=10)`
- THEN 系统并行执行 `fts5_search("compaction", "leveldb", "main", 20)` 和 `vector_search("compaction", "leveldb", "main", 20)`
- AND 两个查询使用 limit*2 以扩展 RRF 候选池
- THEN RRF 融合后截断为 limit 条结果返回

#### Scenario: 搜索返回结果格式
- WHEN 混合搜索完成
- THEN 每条结果包含 `[score]`（RRF 融合分）、`name`（符号名或文档标题）、`type`（code/doc）、`file:line` 位置信息

#### Scenario: vec0 不可用时的降级行为
- WHEN vec0 向量引擎未加载（`try_load()` 返回 false）
- THEN 系统 SHALL 仅返回 FTS5 BM25 结果
- AND 在结果中标注 "向量搜索不可用，仅返回关键词匹配"

#### Scenario: 查询词为空
- WHEN query 参数为空字符串
- THEN 返回错误提示 "query 不能为空"
