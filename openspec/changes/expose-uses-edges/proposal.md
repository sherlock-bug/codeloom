# Proposal: expose-uses-edges

## Intent
`get_terminal_deps()` 函数已在 `graph.rs` 中存在，可返回符号用到的枚举值、全局变量、字符串字面量（`uses:` 边类型）。但没有任何 MCP 工具调用它——LLM 从未看到"这个函数用到了哪些全局变量/枚举值/字符串常量"。

## Scope
In scope:
- 在 `inspect` 的通用 edges 输出中附加 `uses:` 边
- 在 `get_call_graph` 的终端节点中附加 `uses:` 边
- 涉及的 `uses:` 边子类型：`uses_type:`、枚举值 `contains:`、全局变量引用、字符串字面量引用

Out of scope:
- 不加新工具
- 不改索引层（数据已存在）
- 不改搜索工具

## Approach
在 `inspect_symbol()` 和 `traverse_calls()` 中增加对 `get_terminal_deps()` 的调用，将返回结果附加到输出中。仅读取 edges 表，无新写入。
