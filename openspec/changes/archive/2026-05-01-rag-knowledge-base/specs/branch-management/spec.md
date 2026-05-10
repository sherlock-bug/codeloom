# Delta for Branch Management

## ADDED Requirements

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
系统 SHALL 在查询时根据当前分支或指定分支过滤结果。

#### Scenario: 在 feature 分支查询符号
- GIVEN 用户切换到 feature-api 分支视角
- WHEN 执行 `rag get_definition render`
- THEN 返回 feature-api 分支上的 render 函数定义
- AND 如果该分支没有 override，返回 base 分支的默认定义

### Requirement: 列出所有已索引分支
系统 SHALL 提供列出当前项目所有已索引分支的命令。

#### Scenario: 查看项目分支列表
- GIVEN 项目在 main、feature-auth、feature-api 三个分支上索引过
- WHEN 执行 `rag branch list`
- THEN 输出三个分支名、索引时间、符号数量
