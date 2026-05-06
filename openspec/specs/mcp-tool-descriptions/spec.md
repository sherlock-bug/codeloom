# mcp-tool-descriptions

## Purpose
MCP 工具描述采用行为引导风格，引导 LLM 优先使用 CodeLoom 而非 grep/rg 进行代码搜索。

## Requirements

### Requirement: MCP 工具描述引导 LLM 优先使用
所有 MCP 工具的 description SHALL 以「**首选工具**」或「**优先使用**」标记开头，明确引导 LLM 优先于 grep/rg 搜索代码。

#### Scenario: LLM 需要搜索符号名
- WHEN LLM 需要查找某个函数或类的定义
- THEN description 引导 LLM 优先调用 `codeloom_search` 而非 terminal grep

#### Scenario: LLM 需要理解代码结构
- WHEN LLM 需要了解调用关系或类继承关系
- THEN description 明确告知"必须先调用 codeloom_index 后才能用此工具"

### Requirement: 描述包含索引优势说明
每个 MCP 工具的 description SHALL 列出其相对于 grep/文件读取的优势。

#### Scenario: codeloom_search 的优势
- WHEN LLM 阅读 `codeloom_search` 的 description
- THEN 能看到"覆盖 #include 头文件符号，grep 只搜当前目录"等优势说明

### Requirement: 描述包含示例查询词
每个搜索/查询类工具的 description SHALL 包含至少一个具体的 query 示例。

#### Scenario: codeloom_semantic_search 示例
- WHEN LLM 阅读 `codeloom_semantic_search` 的 description
- THEN 能看到如 `query="用户认证流程"` 的具体示例
