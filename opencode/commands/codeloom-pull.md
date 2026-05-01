---
description: 拉取团队共享知识库（通过 MCP 调用）
---

Call the `codeloom_pull` MCP tool to download the shared database.

- Check `~/.codeloom/config.yaml` for the configured `base_db` URL.
- If no source is configured, ask the user for the URL.
- After pulling, call `codeloom_status` to verify the updated data.

Do NOT use terminal/shell — use the MCP tool directly.
