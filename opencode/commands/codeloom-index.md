---
description: 增量索引当前项目代码库（通过 MCP 调用）
---

Call the `codeloom_index` MCP tool to index the current project.

- If the user doesn't specify args, use `path="."` and the current git branch.
- If they mention a repo name, pass it as `repo=<name>`.
- If they mention a branch, pass it as `branch=<name>`.
- After indexing completes, call `codeloom_status` to show the updated statistics.

Do NOT use terminal/shell to run `codeloom index` — use the MCP tool directly.
