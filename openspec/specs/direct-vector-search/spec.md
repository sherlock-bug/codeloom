# direct-vector-search Specification

## Purpose
TBD - created by archiving change remove-legacy-tables. Update Purpose after archive.
## Requirements
### Requirement: 向量索引用 nodes.id
向量索引 SHALL 从 `nodes` 表读取符号，以 `nodes.id` 作为 vec0 表的 rowid。

#### Scenario: 向量索引一致性
- GIVEN nodes 表中有 1000 个 node_type='sym' 的符号
- WHEN `index_vectors()` 执行
- THEN 向量表 symbol_name_vec_{repo} SHALL 包含 1000 行
- AND 每行的 rowid SHALL 等于对应 nodes.id

### Requirement: 向量搜索直连 nodes
语义搜索 SHALL 从 vec0 KNN 结果直接 JOIN `nodes` 表，不经过 symbols 表中转。

#### Scenario: 向量搜索查询
- GIVEN 用户执行语义搜索 "compaction"
- WHEN KNN 在 vec0 中找到 10 个最近邻
- THEN 结果 SHALL 直接 JOIN nodes ON nodes.id = vec0.rowid
- AND 返回 SHALL 包含 name、kind、file_path、signature、doc_comment 等完整信息
- AND 排序 SHALL 按 cosine 距离升序（近 → 远）

