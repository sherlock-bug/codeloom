# Delta for doc-indexing

## ADDED Requirements

### Requirement: 文档索引自动化验证
系统 SHALL 提供自动化测试验证 Markdown 文档索引功能。

#### Scenario: 索引 flatbuffers 文档
- GIVEN flatbuffers 源码在 D:/code/flatbuffers（含 63 个 .md 文件）
- WHEN 执行文档索引
- THEN doc_nodes 表中文档数 > 50
