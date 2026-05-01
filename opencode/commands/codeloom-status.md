---
description: 查看 CodeLoom 索引状态（通过 MCP 调用）
---

Call the `codeloom_status` MCP tool.

- If the user wants a specific repo, pass `repo=<name>`.
- Present the multi-repo statistics clearly: symbol counts per repo, cross-repo edges, storage size.
- If the user asks "are there any issues" or "is everything indexed", check for repos with zero symbols or stale timestamps.

Do NOT use terminal/shell — use the MCP tool directly.
