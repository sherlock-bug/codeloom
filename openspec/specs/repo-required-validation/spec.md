# repo-required-validation

## Purpose
repo 参数从可选改为必填，空值不再静默 fallback。支持非 git 目录的全局数据库索引。

## Requirements

### Requirement: JSON Schema 要求 repo 必填
所有搜索/查询类 MCP 工具的 JSON Schema `required` 数组 SHALL 包含 `repo`。
LLM 不传 repo 时客户端 SHALL 提示缺少必填参数。

#### Scenario: LLM 调用工具不传 repo
- WHEN 客户端根据 JSON Schema 校验参数
- THEN 缺少 repo 时返回参数错误，不进入后端处理

### Requirement: repo 参数接受空字符串（全局数据库）
`validate_repo()` SHALL 接受空字符串为有效 repo 名（对应 `.rag.db` 全局数据库）。
非 git 目录 index 时若未指定 `--repo`，自动使用 `""` 作为 repo。

#### Scenario: 非 git 目录索引
- WHEN 用户对非 git 目录执行 `codeloom index .`
- THEN 自动使用 `repo=""` 存入 `.rag.db`

#### Scenario: 空 repo 校验通过
- GIVEN `.rag.db` 存在
- WHEN 调用 `validate_repo("")`
- THEN 返回 `Ok("")`

### Requirement: 错误信息包含可用仓库列表
repo 参数对应 DB 不存在时，SHALL 在错误信息中附带 `list_repos()` 的输出。

#### Scenario: 错误信息展示可用仓库
- GIVEN 已索引仓库为 `["codeloom", "leveldb"]`
- WHEN LLM 传入不存在的 repo
- THEN 错误信息包含 `可用仓库: codeloom, leveldb`

### Requirement: 全局数据库查询自动 ATTACH
查询非空 repo 时，`open_repo_db()` SHALL 自动 ATTACH 全局 `.rag.db`。
非 git 数据在所有仓库查询中可见。

#### Scenario: 查询 codeloom 仓库时附带全局数据
- GIVEN `.rag.db` 存在且包含符号 `parse_json`
- WHEN LLM 调用 `codeloom_search(query="parse_json", repo="codeloom")`
- THEN 返回结果包含来自 `.rag.db` 的全局数据
