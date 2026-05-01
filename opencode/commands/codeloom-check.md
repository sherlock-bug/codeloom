---
description: 检查 CodeLoom 安装状态和环境
---

Call `codeloom_status` MCP tool, and also verify:

1. Binary accessibility (terminal: `which codeloom`)
2. Config directory exists (`~/.codeloom/`)
3. Database files present and non-zero size
4. MCP connection is active (the fact we can call tools confirms this)

Report a clean summary: what's OK, what's missing, next steps.

Use MCP for status queries, terminal only for `which codeloom` filesystem check.
