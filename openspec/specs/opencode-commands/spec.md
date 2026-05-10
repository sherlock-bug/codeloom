# opencode-commands

## Purpose
CodeLoom opencode-commands 功能域。本规范描述此功能的需求和行为。

## Purpose
提供 OpenCode 自定义命令集成，支持在 OpenCode 内通过 /codeloom:index 等斜杠命令操作知识库。

## Requirements

### Requirement: 自定义命令注册
系统 SHALL 提供 OpenCode 自定义命令文件，支持在 OpenCode 内通过斜杠命令操作知识库。可用命令 SHALL 包括：`/codeloom:index`、`/codeloom:status`、`/codeloom:branch`、`/codeloom:check`。

#### Scenario: `/codeloom:index` 命令
- GIVEN OpenCode 已加载 codeloom 命令
- WHEN 用户在 OpenCode 中输入 `/codeloom:index`
- THEN OpenCode 执行 `codeloom index . --branch $(git branch --show-current)`
- AND 输出索引结果到 OpenCode 对话中

### Requirement: 在不退出 OpenCode 的情况下触发索引操作
系统 SHALL 允许 Agent 在不退出 OpenCode 会话的情况下，触发知识库的索引、更新、查询等全部操作。

#### Scenario: LLM 主动索引未知项目
- GIVEN 用户在 OpenCode 中提问项目相关问题
- AND 项目尚未索引
- WHEN LLM 检测到需要知识库支持
- THEN LLM 调用 rag_index MCP 工具
- AND 索引完成后 LLM 基于新的知识图谱回答问题
- AND 整个过程用户无需退出 OpenCode

### Requirement: 命令帮助
系统 SHALL 在 OpenCode 中键入 `/codeloom:` 后显示所有可用 codeloom 命令的列表和简短描述。

#### Scenario: 查看可用命令
- GIVEN OpenCode 已加载 codeloom 命令
- WHEN 用户输入 `/codeloom:`
- THEN 显示可用命令列表: index, status, branch, check

