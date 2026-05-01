---
description: 推送团队共享知识库（仅维护者）
---

Call the `codeloom_push` MCP tool to upload the base DB to team storage.

- Only the designated maintainer should use this command.
- Default: reads `base_db` URL from `~/.codeloom/config.yaml`.
- Use the `source` parameter to override the destination.
- Before pushing, validate with `codeloom status` that the main branch is indexed.
- After push, notify teammates to run `codeloom pull`.

Regular team members should NOT push — their local `codeloom index` only writes to the personal overlay DB (`project.rag.overlay.db`) and never touches the shared base DB.

Usage: call `codeloom_push` MCP tool with optional `source` parameter.
