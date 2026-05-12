# Proposal: fix-mcp-tool-descriptions

## Intent

当前 MCP 工具的 `description` 存在指向已禁用工具的无效引用、前置指令不统一、以及少量输出格式噪音，导致 LLM 在调用工具时可能走弯路或遇到死链。本次变更旨在清除这些缺陷，让 LLM 能准确、高效地选择和使用 MCP 工具。

## Scope

In scope:
- 清除 `codeloom_index` 中指向已 DISABLED 的 `codeloom_status` 的引用
- 统一 `codeloom_schema` 的"前置调用"指令：改为"拿不准参数值时调用查看可用类型和枚举"，不做强制要求
- 精简 `codeloom_list_repos` 的输出格式描述噪音
- 提升 `codeloom_get_call_graph` 的描述精确度

Out of scope:
- 不改 MCP 工具的输入 Schema 或 required 字段（已验证全部正确）
- 不改工具的实现逻辑或返回值格式
- 不新增或删除工具

## Approach

纯 description 文本修改，仅改动 `src/mcp/mod.rs` 中 `tools_list()` 函数的工具描述字符串。每个问题一行 patch。修改完成后 `cargo build --release` 编译验证，然后通过 MCP tools/list 接口拉取实际返回的描述确认生效。
