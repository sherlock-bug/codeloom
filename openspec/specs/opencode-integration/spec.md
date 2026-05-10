# opencode-integration

## Purpose
CodeLoom opencode-integration 功能域。本规范描述此功能的需求和行为。

## Purpose
OpenCode 使用指引文件 .opencode/instructions.md 提供搜索优先级规则和工具速查表。

## Requirements

### Requirement: 项目根提供 OpenCode 使用指引文件
系统 SHALL 在项目根提供 `.opencode/instructions.md` 文件，引导 LLM 优先使用 CodeLoom MCP 工具进行代码搜索。

#### Scenario: LLM 读取指引文件
- WHEN LLM 打开 CodeLoom 项目
- THEN OpenCode 自动加载 `.opencode/instructions.md`，优先使用 MCP 工具而非 grep

### Requirement: 指引文件内容包含搜索优先级规则
`.opencode/instructions.md` SHALL 包含以下核心规则：
1. 任何代码搜索必须先尝试 MCP 工具
2. 第一步调用 `codeloom_overview` 了解仓库规模
3. 精确搜索用 `codeloom_search` / `codeloom_list_symbols`
4. 语义搜索用 `codeloom_semantic_search`
5. 仅在 MCP 无结果时降级到 grep

#### Scenario: 规则覆盖完整搜索场景
- WHEN LLM 需要搜索代码
- THEN 指引文件覆盖了精确搜索、模糊搜索、语义搜索、调用关系搜索的所有场景

### Requirement: 指引文件包含 MCP 工具速查
`.opencode/instructions.md` SHALL 包含一个简明的 MCP 工具速查表，列出工具名 + 用途 + 示例调用。

#### Scenario: LLM 快速查找工具
- WHEN LLM 不确定用哪个 MCP 工具
- THEN 可以在指引文件中快速查阅速查表找到合适的工具
