# Delta for OpenCode Commands

## ADDED Requirements

### Requirement: 自定义命令注册
系统 SHALL 提供 OpenCode 自定义命令文件，支持在 OpenCode 内通过斜杠命令操作知识库。

#### Scenario: `/rag:index` 命令
- GIVEN OpenCode 已加载 rag 命令
- WHEN 用户在 OpenCode 中输入 `/rag:index`
- THEN OpenCode 执行 `rag index . --branch $(git branch --show-current)`
- AND 输出索引结果到 OpenCode 对话中

#### Scenario: `/rag:pull` 命令
- GIVEN 团队配置了共享 DB 地址
- WHEN 执行 `/rag:pull`
- THEN 拉取最新共享 DB 并覆盖本地 base 层
- AND 输出拉取结果

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
系统 SHALL 在 OpenCode 中键入 `/rag:` 后显示所有可用 rag 命令的列表和简短描述。

#### Scenario: 查看可用命令
- GIVEN OpenCode 已加载 rag 命令
- WHEN 用户输入 `/rag:`
- THEN 显示可用命令列表: index, reindex, pull, status, branch
