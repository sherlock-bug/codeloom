## MODIFIED Requirements

### Requirement: codeloom_index 工具不执行实际索引
`codeloom_index` MCP 工具 SHALL 不执行实际索引操作，而是返回 CLI 命令字符串和状态指示。

#### Scenario: 调用 codeloom_index
- WHEN LLM 调用 `codeloom_index(path="/project", repo="codeloom", branch="main")`
- THEN 返回 "索引请用 CLI: codeloom index /project --repo codeloom --branch main。如果仓库已索引，请用 codeloom_list_repos 查看。"

#### Scenario: description 明确标注不执行索引
- WHEN LLM 阅读 `codeloom_index` 的 description
- THEN 能看到 "⚠️ 此工具不执行索引（索引需用 CLI 命令）" 的标注

### Requirement: 新增 list-repos 和 list-branches MCP 工具
MCP 工具列表 SHALL 增加 `codeloom_list_repos` 和 `codeloom_list_branches` 两个工具。

#### Scenario: tools/list 返回新增工具
- WHEN MCP 客户端执行 `tools/list`
- THEN 返回的 tools 数组包含 `codeloom_list_repos` 和 `codeloom_list_branches`，共 9 个工具

### Requirement: list-repos 工具无额外参数
`codeloom_list_repos` SHALL 无需任何参数即可调用。

#### Scenario: 无参数调用
- WHEN 调用 `codeloom_list_repos`
- THEN 直接返回已索引仓库列表

### Requirement: list-branches 工具接受 repo 参数
`codeloom_list_branches` SHALL 接受 `repo`（必填）参数。

#### Scenario: 指定仓库查询分支
- WHEN 调用 `codeloom_list_branches(repo="codeloom")`
- THEN 返回 codeloom 仓库下所有已索引分支列表

## ADDED Requirements

### Requirement: 统一搜索工具 codeloom_search
MCP 工具列表 SHALL 包含 `codeloom_search` 作为唯一代码搜索入口，替代旧的 `codeloom_search`（LIKE）和 `codeloom_semantic_search`（vec0）。

#### Scenario: tools/list 只返回一个搜索工具
- WHEN MCP 客户端执行 `tools/list`
- THEN 返回的 tools 数组包含一个 `codeloom_search` 工具
- AND 不包含旧的 `codeloom_search`（LIKE 子串匹配）
- AND 不包含 `codeloom_semantic_search`

#### Scenario: codeloom_search 接受标准参数
- WHEN 调用 `codeloom_search`
- THEN 接受参数：`query`（必填，string）、`repo`（必填，string）、`branch`（必填，string）、`limit`（选填，integer，默认10）
