---
description: 在索引过的代码库中搜索符号、定义、调用关系。优先使用 codeloom MCP 工具
---

## branch 参数

所有 codeloom 工具接受 `branch` 参数（当前 git 分支名）。用户未明确指定时，用 `git branch --show-current` 获取并传入。不同用户在不同分支工作，branch 确保返回正确分支的索引数据。

## 工具优先级

1. `codeloom_semantic_search` — 自然语言描述意图（"数据压缩怎么实现"）
2. `codeloom_list_symbols` / `codeloom_search` — 搜已知符号名/关键词
3. `codeloom_get_definition` — 看完整定义（替代 read_file）
4. `codeloom_get_call_graph` — 调用链分析（替代 grep）
5. `codeloom_overview` — 全局架构

仅在 codeloom 未索引该仓库或返回空结果时降级到 grep。
