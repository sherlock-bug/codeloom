# Delta for bm25-precise-search

## MODIFIED Requirements

### Requirement: 精确 BM25 搜索 MCP 工具
系统 SHALL 使用统一 FTS5 表 `fts5_all` 执行精确 BM25 搜索。搜索逻辑从 6 路查询（符号名/注释/文档标题/文档内容/文件名/文件摘要分表查询）简化为 2 路查询（name 列 ×0.7 + content 列 ×0.3），所有来源类型在同一个 FTS5 表中公平竞争。

#### Scenario: 按函数名精确搜索
- GIVEN leveldb 仓库已索引
- WHEN 调用精确搜索工具 query="DBImpl::CompactMemTable"
- THEN 结果第一项 SHALL 是 DBImpl::CompactMemTable 函数
- AND name 列命中 ×0.7 权重的符号 SHALL 排在 content 列命中 ×0.3 权重的文档之前

#### Scenario: 跨类型同一关键词排序
- GIVEN 关键词 "compress" 同时出现在符号名和文档内容中
- WHEN 执行精确搜索
- THEN 符号名命中（name 列 ×0.7）SHALL 排在文档内容命中（content 列 ×0.3）之前
- AND 无论符号名与文档标题的原始 BM25 分数差异多大，权重梯度 SHALL 确保 name 通道优先

### Requirement: 精确搜索噪音过滤
系统 SHALL 使用 BM25 专用噪音基线过滤精确搜索结果。与变更前一致，过滤逻辑不变。

#### Scenario: BM25 噪音基线过滤
- GIVEN BM25 噪音基线已标定
- WHEN 搜索结果中包含低置信度条目
- THEN z-score < 1.0 的条目 SHALL 被过滤
