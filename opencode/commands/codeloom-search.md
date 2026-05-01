---
description: 在索引过的代码库中搜索符号、定义、调用关系。优先使用 codeloom MCP 工具而非 grep
---

## 调用前必须确定 branch

**每次调用 codeloom 工具前，先获取当前分支**：
```bash
git branch --show-current
```
将结果作为 `branch` 参数传入所有 codeloom 工具。如果用户明确指定了分支则使用用户指定的。

## 工具优先级

1. `codeloom_semantic_search` — 自然语言描述意图（"数据压缩怎么实现"）
2. `codeloom_list_symbols` / `codeloom_search` — 搜已知符号名/关键词
3. `codeloom_get_definition` — 看完整定义（替代 read_file）
4. `codeloom_get_call_graph` — 调用链分析（替代 grep）
5. `codeloom_overview` — 全局架构

## 降级策略
仅在以下情况使用 grep：
- codeloom 未索引该仓库
- codeloom 返回空结果
- 需要搜索未被索引的文件（配置、脚本）
