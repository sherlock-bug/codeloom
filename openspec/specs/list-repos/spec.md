# list-repos

## Purpose
CodeLoom list-repos 功能域。本规范描述此功能的需求和行为。

## Purpose
提供已索引仓库列表的查询能力，包括 MCP 工具和 CLI 命令两种接口。

## Requirements

### Requirement: MCP 工具列出已索引仓库
系统 SHALL 提供 `codeloom_list_repos` MCP 工具，返回所有已索引仓库的名称列表。

#### Scenario: 列出已索引仓库
- WHEN LLM 调用 `codeloom_list_repos`
- THEN 返回 JSON 数组格式的仓库名列表，如 `["codeloom", "leveldb", "spdlog"]`

#### Scenario: 无已索引仓库
- WHEN 没有任何仓库被索引过
- THEN 返回空数组 `[]` 并提示"未找到已索引的仓库"

### Requirement: CLI 命令列出已索引仓库
系统 SHALL 提供 `codeloom list-repos` CLI 命令，在终端打印已索引仓库列表。

#### Scenario: 终端列出仓库
- WHEN 用户在终端执行 `codeloom list-repos`
- THEN 输出格式为每行一个仓库名，附 DB 文件大小

### Requirement: 仓库名从数据库文件名推导
系统 SHALL 通过扫描 `~/.codeloom/` 下 `*.rag.db` 文件，从文件名中提取仓库名。

#### Scenario: 推导仓库名
- GIVEN 存在文件 `~/.codeloom/codeloom.rag.db`
- WHEN 执行 list-repos
- THEN 仓库名应为 `codeloom`
