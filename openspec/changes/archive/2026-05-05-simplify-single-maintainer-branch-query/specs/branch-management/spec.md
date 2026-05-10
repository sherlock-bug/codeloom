# Delta for branch-management

## MODIFIED Requirements

### Requirement: 查询时按分支过滤
系统 SHALL 在查询时应用统一的分支过滤规则：`branch_name IS NULL` 的数据对所有分支可见；`branch_name` 有值的数据仅对匹配的分支可见。代码和文档不区别对待。

#### Scenario: 查询指定分支的符号
- GIVEN symbols 表中有 branch_name='feature-x' 的符号 A 和 branch_name IS NULL 的符号 B
- WHEN 以 branch='feature-x' 过滤查询
- THEN 返回符号 A 和符号 B
- AND 不返回 branch_name='main' 的符号

#### Scenario: 查询不同分支的符号
- GIVEN symbols 表中有 branch_name='main' 的符号 C
- WHEN 以 branch='feature-x' 过滤查询
- THEN 不返回符号 C

#### Scenario: 文档在所有分支可见
- GIVEN doc_nodes 表中有一个文档，其 branch_name 为 NULL
- WHEN 在任意分支（'main'、'feature-x'、'release'）查询文档
- THEN 该文档均出现在结果中
