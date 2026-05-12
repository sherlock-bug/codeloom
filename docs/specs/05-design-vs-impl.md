# CodeLoom 设计与实现偏差报告

> 分析来源：2026-05-12 全面规格摸底
> 注意：偏差 ≠ bug，有些是规格过期、描述不准确、或注释与代码不符

---

## MCP 工具偏差

### 1. codeloom_search — kind 可选值列表不准

- **描述**: 工具定义说 kind 可选 10 种 `function,method,class,struct,enum,enum_value,field,global,static_var,variable`
- **实现**: 实际接受 15 种（多了 `template_function, macro, namespace, template_instance, typedef, string_literal`）
- **影响**: 低。LLM 传入的 kind 值会被完整校验，无效值会报错。描述不全不影响功能。
- **建议**: 同步工具描述中的 kind 列表（或在描述中写"见 codeloom_schema"）

### 2. codeloom_semantic_search — 与 BM25 搜索输出不一致

- **描述**: 搜索增强 spec 要求两者一致
- **实现**: 语义搜索结果**缺少 `id` 字段**，`type` 硬编码为 `"code"`（忽略实际 hit_type）
- **影响**: 中。MCP 用户（LLM）无法通过 id 直接跳转到 inspect，也不能区分代码和文档结果

### 3. codeloom_inspect — 缺少 template_args

- **描述**: inspect-enrichment spec 要求 class/struct 包含 `template_args`
- **实现**: 只实现了 bases, members, methods，没有模板参数查询
- **影响**: 低。模板类在 C++ 项目中常见，inspect 返回的信息不全

### 4. codeloom_list_branches — 缺少符号数

- **描述**: 工具描述说"返回分支名和符号数"
- **实现**: 只返回分支名，没有 JOIN 查询符号数量
- **影响**: 低。符号数对 LLM 判断分支活跃度有用，但非关键

### 5. codeloom_schema — 节点/边类型数量不准

- **描述**: 说"15 种节点/11 种边"
- **实现**: 实际 16 种节点（多 `template_struct`）/ 10 种边（少 1 种）
- **边类型名称差异**:

| 描述说 | 实际返回 |
|--------|---------|
| `calls, inherits, overrides, instantiates, param_type, return_type, includes, uses_type, contains, aliases, uses` | `calls, calls_override, inherits, contains, uses, references, returns, param_type, field_type, template_use` |

- **影响**: 中。LLM 可能根据描述传不存在的边类型名

---

## 内部能力偏差

### 6. hybrid_search — 硬阈值注释未实现

- **描述**: 注释第 234 行说"Hard threshold: drop results with normalized score < 0.5"
- **实现**: 实际只做了 `fused.truncate(limit)`，未过滤
- **影响**: 低。与 bm25_precise_search 的降噪逻辑不一致，但没有明显功能退化

### 7. Tree-sitter 多语言解析 — queries 未实现

- **描述**: 设计意图是 tree-sitter 做多语言符号提取
- **实现**: `src/indexer/queries/mod.rs` 为空，非 C++ 语言的符号提取是 stub
- **影响**: **高**。Python/Java/TS/Go 项目索引后无符号，搜索不到

### 8. #include 关系 — edge 无符号关联

- **描述**: 正则提取 #include 并写入 edges
- **实现**: `source_id=0, target_id=0`，无法关联到具体符号
- **影响**: 中。#include 边存在但不可查

---

## 其他偏差

### 10. codeloom_status (disabled) — 函数完整但被禁用

- 无偏差，但值得记录：`status()` 函数的 MCP dispatch 被注释，工具无法使用。同情况的还有 `get_doc` 和 `query_excel`

### 11. check 命令的工具数硬编码

- `check` 输出写到 "MCP tools: 9 tools"，实际有 16+ 工具
- **影响**: 低。纯展示问题

---

## 偏差总结

| 等级 | 数量 | 项目 |
|------|------|------|
| 🟡 中 | 3 | semantic_search 输出不一致 · schema 类型不准 · include edges 无符号关联 |
| 🟢 低 | 5 | search kind 列表 · inspect template_args · list_branches 缺数量 · hybrid 阈值注释 · check 工具数 |
| ⚪ 记录 | 3 | status/get_doc/query_excel 被禁用但代码完整 |
