# rrf-ranking-fix

## Purpose
CodeLoom rrf-ranking-fix 功能域。本规范描述此功能的需求和行为。

## Purpose
混合搜索加权融合排名修复：用 0.5*BM25_norm + 0.5*cosine_sim 替代纯 RRF，搜索结果返回 snippet + doc_id 可追溯，使 LLM 能直接获取文档完整内容。

## Requirements

### Requirement: 混合搜索加权融合
系统 SHALL 使用加权融合算法（weighted fusion）将 BM25 FTS5 结果与向量语义搜索结果合并，权重为 0.5*BM25_norm + 0.5*cosine_sim，替代纯 RRF 排名融合。

#### Scenario: 精确匹配排名最高
- GIVEN 用户搜索 "DoCompactionWork"，该函数在 BM25 中精确匹配
- WHEN 混合搜索执行加权融合
- THEN 精确匹配的符号 SHALL 排名第一，与模糊匹配的符号有显著分数差距

#### Scenario: 单边无结果时回退
- GIVEN 用户用中文搜索 "压缩"，BM25 返回 0 条
- WHEN 混合搜索执行加权融合
- THEN 系统 SHALL 回退到纯向量搜索结果（w_vec=1.0），不因 BM25 为空而丢弃向量结果

### Requirement: 搜索结果包含 doc_id 可追溯
系统 SHALL 在文档搜索结果的 CLI 和 MCP 输出中包含 doc_id（doc_nodes.rowid），使 LLM 可通过 codeloom_get_doc 工具获取文档完整内容。

#### Scenario: CLI 输出 doc_id
- GIVEN 用户搜索 "记录格式" 命中 log_format.md
- WHEN CLI 格式化搜索结果
- THEN 文档结果 SHALL 显示 `[doc [id:N]]` 格式，其中 N 为 doc_nodes.rowid

#### Scenario: MCP 输出 doc_id
- GIVEN 用户通过 MCP 搜索文档
- WHEN MCP 格式化搜索结果
- THEN 文档结果 SHALL 包含 `|doc_id:N` 字段

### Requirement: 搜索结果包含内容摘要
系统 SHALL 在搜索结果中返回文档内容摘要（snippet），doc 结果取前 200 字符，使 LLM 无需额外查询即可判断相关性。

#### Scenario: 文档结果带摘要
- GIVEN 用户搜索 "记录格式" 命中文档节点
- WHEN 搜索结果格式化
- THEN 每条文档结果 SHALL 包含 snippet 字段，内容为文档正文的前 200 字符
