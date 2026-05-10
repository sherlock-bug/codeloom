# branch-management

## Purpose
CodeLoom branch-management 功能域。本规范描述此功能的需求和行为。

## Purpose
管理代码库的多分支和版本标记，自动检测 git 分支，支持手动指定标签，按分支过滤查询结果，提供分支列表查看功能。

## Requirements

### Requirement: 自动识别当前 Git 分支
系统 SHALL 在索引时自动检测 git 仓库的当前分支名，并作为符号的分支标签。

#### Scenario: 在 git 仓库中执行索引
- GIVEN 当前项目是一个 git 仓库，位于 feature-api 分支
- WHEN 执行索引命令
- THEN 所有新索引的符号被标记 branch_name='feature-api'
- AND 符号归属记录在 branches 表中

### Requirement: 支持手动指定分支标签
系统 SHALL 允许用户通过 --branch 参数手动指定分支标签，覆盖自动检测。

#### Scenario: 为非 git 目录指定版本标签
- GIVEN 当前目录不在 git 仓库中
- WHEN 执行 `rag index --branch v2.3.1`
- THEN 所有索引符号标记 branch_name='v2.3.1'

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

### Requirement: 列出所有已索引分支
系统 SHALL 提供列出当前项目所有已索引分支的命令。

#### Scenario: 查看项目分支列表
- GIVEN 项目在 main、feature-auth、feature-api 三个分支上索引过
- WHEN 执行 `rag branch list`
- THEN 输出三个分支名、索引时间、符号数量
