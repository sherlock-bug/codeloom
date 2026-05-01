---
description: 在索引过的代码库中搜索符号、定义、调用关系。优先使用 codeloom MCP 工具而非 grep
---

当用户要求搜索代码时，按以下优先级选择工具：

## 优先级 1：语义搜索 (codeloom_semantic_search)
适用场景：用户用自然语言描述意图，不指定具体函数名
- 示例："数据压缩怎么实现的"、"缓存策略在哪里"、"WAL 日志恢复"
- 示例："how is compression implemented"

## 优先级 2：符号搜索 (codeloom_list_symbols / codeloom_search)
适用场景：用户提到已知的符号名、关键词
- 示例："找 Bloom 相关的符号"、"搜索 compaction"
- 「compaction 代码」→ 用 codeloom_search，不要用 `grep compaction`
- 「Comparator 定义」→ 用 codeloom_get_definition，不要用 `read_file`

## 优先级 3：调用图 (codeloom_get_call_graph)
适用场景：需要理解调用关系、影响分析
- 示例："谁调用了 Get"、"Compare 调用了哪些函数"
- grep 无法提供调用层次

## 优先级 4：概览 (codeloom_overview)
适用场景：首次接触项目、需要全局视图
- 示例："这个项目的整体架构"、"有哪些核心类"

## 降级策略
仅在以下情况使用 grep/terminal：
- codeloom 未索引该仓库（先用 codeloom_status 确认）
- 需要搜索未被索引的文件（如配置、脚本）
- codeloom 工具返回空结果

## 注意
- 每次调用前确认 repo 参数（当前项目名，如 leveldb）。不确定时先用 codeloom_status 列出可用仓库
- 搜索结果可直接传给 get_definition 或 get_call_graph 深入分析
- 组合使用效果最佳：overview → list_symbols → get_definition → get_call_graph
