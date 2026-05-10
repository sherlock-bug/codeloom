# Delta for opencode-commands

## MODIFIED Requirements

### Requirement: 自定义命令注册
系统 SHALL 提供 OpenCode 自定义命令文件，支持在 OpenCode 内通过斜杠命令操作知识库。可用命令 SHALL 包括：`/codeloom:index`、`/codeloom:status`、`/codeloom:branch`、`/codeloom:check`。

#### Scenario: `/codeloom:index` 命令
- GIVEN OpenCode 已加载 codeloom 命令
- WHEN 用户在 OpenCode 中输入 `/codeloom:index`
- THEN OpenCode 执行 `codeloom index . --branch $(git branch --show-current)`
- AND 输出索引结果到 OpenCode 对话中

### Requirement: 命令帮助
系统 SHALL 在 OpenCode 中键入 `/codeloom:` 后显示所有可用 codeloom 命令的列表和简短描述。

#### Scenario: 查看可用命令
- GIVEN OpenCode 已加载 codeloom 命令
- WHEN 用户输入 `/codeloom:`
- THEN 显示可用命令列表: index, status, branch, check

## REMOVED Requirements

### Requirement: `/codeloom:pull` 和 `/codeloom:push` 命令
**Reason**: Pull/Push 功能不再需要。
**Migration**: 删除 `codeloom-pull.md` 和 `codeloom-push.md` 命令文件。
