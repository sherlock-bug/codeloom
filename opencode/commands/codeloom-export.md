---
description: 管理员导出团队配置模板和共享知识库文件
---

Help the admin export CodeLoom team configuration and shared knowledge base for team distribution.

## Step 1: Determine what to export
Ask: "要导出什么？(all / config / db)"
- config: 导出团队配置模板文件
- db: 导出共享知识库 rag.db
- all: 两者都导出

## Step 2: Export config template
If config or all:

Read `~/.codeloom/config.yaml` and convert to a team template format:

```yaml
# CodeLoom 团队配置模板
project: <project-name>

repos_required:
  - backend
  - frontend
  # ... (list all repo names without paths)

base_db: <base_db URL from config>

# similarity_threshold: 0.75  (if customized)
```

Save to `<export-dir>/team-config.yaml`.

## Step 3: Export rag.db
If db or all:

Find the rag.db files: `ls ~/.codeloom/*.rag.db`

For each file, tell the user the size and ask:
"project.rag.db (180MB) — 导出到哪里？"
Options: "本地路径" or "上传到共享存储"

If local: `cp ~/.codeloom/project.rag.db <path>/project.rag.db`
If upload: run `codeloom push` to upload to the configured base_db URL

## Step 4: Summary
Show what was exported:
- `team-config.yaml` at <path>
- `project.rag.db` at <path> (XX MB)

Tell the admin: "团队成员可以用 `/codeloom:smart-setup` 加载 team-config.yaml 完成配置，用 `codeloom pull` 拉取共享知识库。"
