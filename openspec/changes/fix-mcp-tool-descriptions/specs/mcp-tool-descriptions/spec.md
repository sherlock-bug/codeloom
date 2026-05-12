# Delta for mcp-tool-descriptions

## MODIFIED Requirements

### Requirement: MCP 工具描述不得引用已禁用工具
所有 MCP 工具的 description SHALL 仅引用已注册且非 DISABLED 的工具名称。引用已禁用工具的描述 SHALL 修正为指向有效替代工具。

#### Scenario: codeloom_index 不引用 codeloom_status
- GIVEN `codeloom_status` 已被 DISABLE
- WHEN LLM 读取 `codeloom_index` 工具的 description
- THEN 描述 SHALL NOT 包含 `codeloom_status`
- AND 替代指引 SHALL 指向 `codeloom_list_repos`

### Requirement: codeloom_schema 描述采用按需引导而非强制前置
`codeloom_schema` 的 description SHALL 以"拿不准参数值时调用"的按需语气描述，而非"调用某工具前必须先调用"的强制语气。

#### Scenario: LLM 拿不准边类型时调用 schema
- GIVEN LLM 不确定 `edge_filter` 参数有哪些可用值
- WHEN LLM 搜索能提供元数据信息的工具
- THEN LLM SHALL 调用 `codeloom_schema` 查看可用枚举值

### Requirement: 图分析工具描述含准确竞争区分
每个图分析 MCP 工具的 description SHALL 明确其相对于最接近的替代工具的竞争优势。

#### Scenario: call_graph 区分于 neighbor_graph
- GIVEN LLM 需要分析函数调用链
- WHEN 对比 `codeloom_get_call_graph` 和 `codeloom_neighbor_graph` 的描述
- THEN call_graph 的描述 SHALL 说明其相比 neighbor_graph 的优势（一次到位+递归深度控制）
