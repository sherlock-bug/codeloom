---
description: 切换或查看 CodeLoom 索引分支（通过 MCP 调用）
---

If the user doesn't specify a branch name:
- Call `codeloom_status` MCP tool to list all indexed branches across repos.
- Present them grouped by repo.

If the user specifies a branch name:
- Call the appropriate MCP tool to switch branch context.
- Subsequent queries will filter to this branch.

Do NOT use terminal/shell — use MCP tools directly.
