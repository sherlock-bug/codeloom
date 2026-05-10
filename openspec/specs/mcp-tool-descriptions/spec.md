# mcp-tool-descriptions

## Purpose
CodeLoom mcp-tool-descriptions 功能域。本规范描述此功能的需求和行为。

## Purpose
MCP 工具描述采用行为引导风格，引导 LLM 优先使用 CodeLoom 而非 grep/rg 进行代码搜索。
## Requirements
### Requirement: MCP 工具描述引导 LLM 优先使用
所有 MCP 工具的 description SHALL 以「**首选工具**」或「**优先使用**」标记开头，明确引导 LLM 优先于 grep/rg 搜索代码。

#### Scenario: LLM 需要搜索符号名
- WHEN LLM 需要查找某个函数或类的定义
- THEN description 引导 LLM 优先调用 `codeloom_search` 而非 terminal grep

#### Scenario: LLM 需要搜索功能意图
- WHEN LLM 需要用中文或英文描述搜索功能
- THEN description 告知 "query 可以是中文功能描述（如'用户认证'）或符号名（如'AuthService'）"

### Requirement: 描述包含索引优势说明
每个 MCP 工具的 description SHALL 列出其相对于 grep/文件读取的优势。

#### Scenario: codeloom_search 的优势
- WHEN LLM 阅读 `codeloom_search` 的 description
- THEN 能看到 "覆盖 #include 头文件符号，同时理解精确命名和语义意图，grep 只搜当前目录且不理解语义" 等优势说明

### Requirement: 描述包含示例查询词
每个搜索/查询类工具的 description SHALL 包含至少一个具体的 query 示例。

#### Scenario: codeloom_search 示例
- WHEN LLM 阅读 `codeloom_search` 的 description
- THEN 能看到如 `query="用户认证"`（中文语义）和 `query="AuthService"`（精确符号名）两种示例

### Requirement: 描述不暴露实现细节
`codeloom_search` 的 description SHALL 不提及 BM25、FTS5、RRF、vec0、向量 等实现细节，只描述行为。

#### Scenario: LLM 阅读 description 不看到实现术语
- WHEN LLM 阅读 `codeloom_search` 的 description
- THEN 看不到 "BM25", "FTS5", "RRF", "vec0", "向量嵌入", "ANN" 等实现术语

