# Delta for cli-mode

## MODIFIED Requirements

### Requirement: CLI 方式执行索引
CLI 索引命令 SHALL 完成代码索引和文档索引后直接结束，不再执行文档-代码关联计算。

#### Scenario: 索引完成不计算关联
- GIVEN 项目包含代码和 Markdown 文档
- WHEN 执行 `codeloom index . --branch main`
- THEN 代码符号和文档节点正常入库
- AND 不执行 `link_docs_to_symbols`
- AND 索引耗时减少（省去 O(symbols×docs) 相似度计算）
