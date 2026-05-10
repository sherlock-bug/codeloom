# Delta for enhanced-call-graph

## MODIFIED Requirements

### Requirement: 调用图终端节点增强
系统 SHALL 在调用图遍历到的每个终端节点上附加该节点使用的枚举值、引用的全局/静态变量、包含的字符串字面量。

#### Scenario: 调用分析附带依赖信息
- GIVEN 函数 `validate` 使用了 `Status::ERROR`，引用了 `g_config`，包含了字面量 `"invalid"`
- WHEN 查询 `codeloom_get_call_graph` 遍历到 `validate`
- THEN 返回结果中 `validate` 节点包含 `uses: [Status::ERROR]`、`references: [g_config]`、`string_literals: ["invalid"]`

#### Scenario: 终端节点无依赖时不输出空字段
- GIVEN 函数 `helper` 不使用任何枚举值/全局变量/字面量
- WHEN 查询遍历到 `helper`
- THEN 该节点不包含空的 `uses`/`references`/`string_literals` 字段
