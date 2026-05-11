# Delta for edge-based-doc-hierarchy

## ADDED Requirements

### Requirement: 文档层级用 contains 边
文档章节与子 chunk 的层级关系 SHALL 通过 `edges` 表的 `contains` 边表达，不依赖 `nodes.parent_id` 列。

#### Scenario: 文档 chunk 关联
- GIVEN 文档 `impl.md` 解析为 1 个父节点和 5 个 chunk 子节点
- WHEN 索引完成
- THEN edges 表 SHALL 包含 5 条 edge_type='contains' 的边
- AND source_id 为父节点 id，target_id 为各 chunk id

#### Scenario: 查询文档层级
- WHEN 通过 graph query 工具查询某文档节点的 contains 边
- THEN 返回所有子 chunk 节点
- AND 与代码类的 contains 边（类→方法）使用相同的查询 API

### Requirement: neighbor-graph 支持 contains
邻接图查询 SHALL 支持 `contains` 边类型，返回节点的直接子节点。

#### Scenario: 邻接图查文档子节点
- GIVEN 文档节点有 5 个 contains 边
- WHEN 调用 neighbor-graph direction=forward edge_type=contains
- THEN 返回 5 个子节点
