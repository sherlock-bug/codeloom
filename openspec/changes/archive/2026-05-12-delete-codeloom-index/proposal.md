# Proposal: delete-codeloom-index

## Intent
`codeloom_index` 是一个 MCP 工具，但它的 handle 函数 100% 返回"请走 CLI"的错误提示。LLM 每次调用它都是浪费一次工具调用的 token 和延迟。且它的 description 中还包含了已 DISABLE 的 `codeloom_status` 引用（虽然已修复过一次）。

## Scope
In scope:
- 从 `tools_list()` 函数中移除 `codeloom_index` 工具定义
- 从 `handle_tool_call()` 中移除对应的分发分支
- 在 `codeloom_list_repos` 的 description 中补充一句引导（"索引需通过 CLI 执行：codeloom index <path> --repo <name> --branch <branch>"）

Out of scope:
- 不改其他工具

## Approach
注释掉 `tools_list()` 中 codeloom_index 的 JSON 块，删除 `handle_tool_call()` 中对应的 match 分支。
