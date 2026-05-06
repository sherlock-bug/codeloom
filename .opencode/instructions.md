# CodeLoom 项目使用指引

本项目已通过 CodeLoom MCP 进行代码索引。所有代码搜索、符号查找、调用关系分析必须优先使用 MCP 工具。

## 搜索优先级规则

1. **任何代码搜索必须先尝试 MCP `codeloom_search`，禁止直接 grep/rg。**
2. `codeloom_search` 是混合搜索引擎：自动融合关键词精确匹配（BM25）和语义理解（向量），一个工具覆盖两种需求。
3. 只有在 MCP 无结果时才降级到 terminal grep。

## 工作流

### 打开仓库后第一步
```
codeloom_list_repos        → 查看有哪些已索引的仓库
codeloom_overview(repo=)   → 了解代码库规模（符号分布、文件数）
codeloom_status(repo=)     → 检查索引完整性和统计
```

### 搜索（唯一入口）
```
codeloom_search(query="ClassName")          → 精确符号名搜索
codeloom_search(query="用户认证流程")         → 中文功能描述搜索
codeloom_search(query="memory allocation")  → 英文功能描述搜索
```
`codeloom_search` 同时利用 BM25 关键词和向量语义，返回统一排序结果。

### 获取定义
```
codeloom_list_symbols(pattern="method") → 模糊匹配找到完整符号名
codeloom_get_definition(name="ClassName::method") → 获取完整定义
```

### 关系分析（grep 无法替代）
```
codeloom_get_call_graph(name="ClassName::method", direction="callees") → 调用链
codeloom_get_call_graph(name="ClassName::method", direction="callers") → 谁调用了它
```

## MCP 工具速查（9 个工具）

| 工具 | 用途 | 必填参数 |
|------|------|---------|
| `codeloom_list_repos` | 列出所有已索引仓库 | 无 |
| `codeloom_list_branches` | 列出仓库分支 | repo |
| `codeloom_overview` | 仓库架构全貌 | branch, repo |
| `codeloom_status` | 索引状态统计 | branch, repo |
| `codeloom_search` | **唯一搜索入口**（混合 BM25+向量） | query, branch, repo |
| `codeloom_list_symbols` | 模糊搜索符号名 | pattern, branch, repo |
| `codeloom_get_definition` | 获取符号完整定义 | name, branch, repo |
| `codeloom_get_call_graph` | 调用关系分析 | name, branch, repo |
| `codeloom_index` | ⚠️ 不执行索引（需CLI） | path, branch |

## 降级规则

- `codeloom_search` 无结果 → 尝试 `codeloom_list_symbols` 模糊匹配（换更宽泛的 pattern）
- 仍无结果 → **最后才用** `grep -r "keyword" .` 或 `rg "keyword"`
