# list-branches

## Purpose
提供仓库下已索引分支列表的查询能力，支持 MCP 工具和 CLI 命令。

## Requirements

### Requirement: MCP 工具列出仓库下已索引分支
系统 SHALL 提供 `codeloom_list_branches` MCP 工具，参数 `repo`（必填），返回该仓库下所有已索引分支的名称。

#### Scenario: 列出指定仓库的分支
- WHEN LLM 调用 `codeloom_list_branches(repo="codeloom")`
- THEN 返回分支名列表，如 `["main", "feature/vector-search", "release/0.3.5"]`

#### Scenario: 仓库不存在
- WHEN repo 参数指定的仓库未索引
- THEN 返回错误 "未找到仓库 'xxx'。可用仓库: [...]"，触发 `codeloom_list_repos` 的结果

### Requirement: CLI 命令列出分支
系统 SHALL 提供 `codeloom list-branches <repo>` CLI 命令。

#### Scenario: 终端列出分支
- WHEN 用户执行 `codeloom list-branches codeloom`
- THEN 输出每行一个分支名，按字母排序

### Requirement: 分支数据来源
系统 SHALL 通过查询 `branches` 表的 `DISTINCT branch_name` 获取分支列表。

#### Scenario: branches 表查询
- GIVEN branches 表中存在 `branch_name='main'`、`branch_name='dev'`、`branch_name IS NULL`
- WHEN 执行 list-branches
- THEN 返回 `["main", "dev"]`（过滤掉 NULL）
