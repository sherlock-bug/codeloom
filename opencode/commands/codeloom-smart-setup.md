---
description: 从团队配置文件导入，自动扫描代码仓，引导完成配置
---

Guide the user through a smarter CodeLoom setup using a team-provided template config file. The user doesn't need to manually type repo paths — just provide the root directory and let auto-scan do the work.

## Step 1: Load team template
Ask: "请提供团队配置文件路径（本地文件或 URL）"
The template should be a YAML file like:
```yaml
project: my-project
repos_required:
  - backend
  - frontend
  - shared-lib
  - admin
base_db: s3://team-rag/my-project.rag.db
```

After loading, tell the user: "这个项目需要 X 个代码仓: [列出所有 repo 名称]"

## Step 2: Root directory
Ask: "所有代码仓在哪个根目录下？（如 /home/user/work/）"

Scan the directory (using `ls` or `find -maxdepth 2 -type d`) and match subdirectories against the required repo names. Report:

```
扫描结果:
  /home/user/work/backend     ✅ 匹配
  /home/user/work/frontend    ✅ 匹配
  /home/user/work/shared-lib  ✅ 匹配
  admin                       ❌ 未找到
  mobile-api                  ❌ 未找到

已找到 3/5 个代码仓。
```

## Step 3: Handle missing repos
If some repos are missing, ask: "缺少 admin, mobile-api。要继续仅配置已找到的 3 个吗？(yes/no/手动指定)"

- If yes: proceed with found repos
- If no: ask user to provide the path for each missing repo manually
- If 手动指定: ask for paths one by one

## Step 4: Branch aliases (optional)
Ask: "有没有分支惯用叫法？（别名=分支名，每行一个，空行结束）"

## Step 5: Confirm and write
Show the final config with found repos + manually specified ones:

```yaml
projects:
  my-project:
    repos:
      backend:
        root: /home/user/work/backend
      frontend:
        root: /home/user/work/frontend
      shared-lib:
        root: /home/user/work/shared-lib
    base_db: s3://team-rag/my-project.rag.db
    similarity_threshold: 0.75
```

Ask: "确认写入 ~/.codeloom/config.yaml？（yes/no）"

After confirmation:
1. Write config
2. For each branch alias: `codeloom branch set-alias <alias> <branch> --repo my-project`
3. Run `codeloom check`
4. Tell user: "配置完成！运行 `/codeloom:index` 开始索引"

If missing repos were skipped, remind: "admin, mobile-api 未配置。之后可用 `codeloom branch set-alias` 或重新运行 `/codeloom:smart-setup` 补充。"
