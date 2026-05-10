# Delta for code-indexing

## MODIFIED Requirements

### Requirement: 增量索引
系统 SHALL 支持增量索引，仅重新解析自上次索引以来修改过的文件。索引完成后 SHALL 将符号关联到当前分支（通过 `branches` 表 `branch_name` 字段记录）。

#### Scenario: 修改单个文件后增量索引
- GIVEN 项目已完成全量索引（branch='main'），用户修改了单个文件
- WHEN 再次执行索引命令 `codeloom index . --branch main`
- THEN 系统仅解析该修改文件并更新其符号
- AND 未修改文件的符号保持不变
- AND 新符号的 branch_name 记录为 'main'

### Requirement: 查询时按分支过滤代码符号
系统 SHALL 在 MCP 查询工具中按分支过滤代码符号。过滤规则与 branch-management spec 一致：`branch_name IS NULL` 的符号全分支可见；`branch_name` 有值的符号仅匹配分支可见。

#### Scenario: 按分支查询符号定义
- GIVEN 符号 `MyClass::render` 在 main 分支的索引中 `branch_name='main'`
- WHEN LLM 调用 `codeloom_get_definition(name="MyClass::render", branch="main")`
- THEN 返回该符号的定义
- WHEN LLM 调用 `codeloom_get_definition(name="MyClass::render", branch="feature-x")`
- THEN 不返回该符号（分支不匹配）
