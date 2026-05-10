# Delta for bm25-precise-search

## ADDED Requirements

### Requirement: 精确 BM25 搜索 MCP 工具
系统 SHALL 提供独立的精确 BM25 搜索工具，使用 FTS5 BM25 关键词匹配，不掺杂向量语义搜索。搜索覆盖符号名称、注释、文档内容和文件信息，所有节点类型统一名称通道 ×0.7 和内容通道 ×0.3 的权重逻辑。

#### Scenario: 按函数名精确搜索
- GIVEN leveldb 仓库已索引
- WHEN 调用精确搜索工具 query="DBImpl::CompactMemTable"
- THEN 结果第一项 SHALL 是 DBImpl::CompactMemTable 函数
- AND 来自名称通道命中的结果 SHALL 排在注释通道命中之前

#### Scenario: 按关键词搜索注释
- GIVEN leveldb 仓库已索引
- WHEN 调用精确搜索工具 query="压缩"
- THEN 注释中包含"压缩"的符号 SHALL 出现在结果中
- AND 名称中直接包含"压缩"的符号 SHALL 排在注释命中的符号之前

#### Scenario: 搜索文档
- GIVEN 文档已索引
- WHEN 调用精确搜索工具 query="架构设计"
- THEN 标题或内容匹配"架构设计"的文档节点 SHALL 出现在结果中

#### Scenario: 空结果
- GIVEN leveldb 仓库已索引
- WHEN 调用精确搜索工具 query="xyxxy_no_match_xyz"
- THEN 返回结果 SHALL 为空数组

### Requirement: 精确搜索噪音过滤
系统 SHALL 使用 BM25 专用噪音基线过滤精确搜索结果中的低置信度条目。

#### Scenario: BM25 噪音基线过滤
- GIVEN BM25 噪音基线已标定（top1_mean=0.3, top1_std=0.1）
- WHEN 某搜索结果 score=0.25
- THEN 该结果 SHALL 被过滤（z-score = (0.25-0.3)/0.1 = -0.5 < 1.0）
