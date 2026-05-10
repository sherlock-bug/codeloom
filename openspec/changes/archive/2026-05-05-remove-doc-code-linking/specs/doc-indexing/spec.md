# Delta for doc-indexing

## MODIFIED Requirements

### Requirement: 解析 Markdown 文档结构
系统 SHALL 解析 Markdown 文档的标题层级、内部链接和代码块，提取为结构化知识节点。文档节点 SHALL 默认不绑定分支（branch_name=NULL），使其对所有分支可见。

#### Scenario: 提取文档标题层级
- GIVEN 一个包含多级标题的 Markdown 文档
- WHEN 执行 `codeloom index .`
- THEN 每个标题创建为独立 doc_node
- AND doc_node 的 branch_name 为 NULL

#### Scenario: 文档在所有分支可见
- GIVEN doc_nodes 表中文档 D 的 branch_name 为 NULL
- WHEN 在任意分支查询文档
- THEN 文档 D 出现在查询结果中

## REMOVED Requirements

### Requirement: 代码块关联到代码符号
**Reason**: 随 `code-doc-linking` 功能移除。语义搜索同时返回 doc 和 code 结果，LLM 自行理解关联。
**Migration**: 删除 `doc_code_links` 相关逻辑。文档索引本身不变。
