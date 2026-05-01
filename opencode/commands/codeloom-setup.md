---
description: 交互式引导配置 CodeLoom（代码路径、共享知识库等）
---

You are helping the user set up CodeLoom configuration. Guide them step by step, asking for each piece of information. After collecting everything, write the config to `~/.codeloom/config.yaml`.

Do NOT write the config until the user confirms all values.

## Step 1: Project name
Ask: "项目组叫什么名字？（如 my-project）"
Default: current directory name

## Step 2: Code repositories
Ask: "有哪些代码仓需要索引？（输入格式: `仓名=路径`，每行一个，空行结束）"
Example:
```
backend=/home/user/work/backend
frontend=/home/user/work/frontend
shared-lib=/home/user/work/shared-lib
```
Tell the user: "至少输入一个，完成后输入空行"

## Step 3: Shared knowledge base
Ask: "团队共享知识库的地址是什么？（s3://、https:// 或本地路径，没有可跳过）"
Example: `s3://team-rag/my-project.rag.db`
If skipped: "跳过团队共享配置，仅本地使用"

## Step 4: Similarity threshold
Ask: "文档-代码自动关联的相似度阈值？（0.5-1.0，默认 0.75）"
Explain: "索引代码和文档时，如果某个文档段落的语义向量与某个代码符号的语义向量足够接近（余弦相似度），系统会自动把它们关联起来。阈值越高越严格（只有高度相关的才关联），越低越宽松（会关联更多但可能不准确）。0.75 是推荐值。"
Default if skipped: 0.75

## Step 5: Branch aliases (optional)
Ask: "有没有分支惯用叫法需要添加？（格式: `别名=分支名`，每行一个，空行结束）"
Example:
```
23B=release/2023-B-sprint-patch-4
24A=release/2024-A-mainline
```
If skipped: "跳过"

## Step 6: Confirm and write
Show the full config to the user:

```yaml
projects:
  <name>:
    repos:
      backend:
        root: /home/user/work/backend
      frontend:
        root: /home/user/work/frontend
    base_db: s3://...
    similarity_threshold: 0.75
```

Ask: "确认写入 ~/.codeloom/config.yaml 吗？（yes/no）"

After confirmation:
1. Write the config file
2. Run `codeloom branch set-alias <alias> <branch> --repo <name>` for each alias
3. Run `codeloom check` to verify
4. Tell the user: "配置完成！运行 `codeloom index` 开始索引，或 `/codeloom:index` 在 OpenCode 中索引"

If the user answers no: ask what to change and re-show.
