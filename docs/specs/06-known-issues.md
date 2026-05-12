# CodeLoom 已知问题 (Bug)

> 生成时间：2026-05-12
> 来源：全面规格摸底后用户审阅确认

---

## 一、输出不一致

### BUG-01: codeloom_semantic_search 输出与 BM25 搜索不一致

- **涉及**: `codeloom_semantic_search` MCP 工具
- **现状**: 
  - 缺少 `id` 字段（BM25 搜索有）
  - `type` 硬编码为 `"code"`，忽略实际 `hit_type`（BM25 搜索区分 code/doc/file）
- **预期**: 搜索增强 spec 要求两种搜索工具输出格式一致
- **代码位置**: `src/mcp/mod.rs:466-477`
- **建议**: `semantic_search()` 的输出 JSON 构造需要加 `id` 字段，并移除 `"type": "code"` 硬编码，改用 `r.hit_type`

---

### BUG-02: codeloom_list_branches 描述与实际不符

- **涉及**: `codeloom_list_branches` MCP 工具
- **现状**: 工具描述说"返回分支名和符号数"，实际只 `SELECT DISTINCT branch_name`，未查询 COUNT(*)
- **预期**: 规格已改为"只返回分支名"，需要同步工具描述
- **代码位置**: `src/mcp/mod.rs:207-234`
- **建议**: 方式一——改工具描述去掉"符号数"；方式二——加 JOIN 查询实现符号数

---

## 二、功能缺失

### BUG-03: codeloom_inspect class/struct 缺少 template_args

- **涉及**: `codeloom_inspect` MCP 工具
- **现状**: inspect-enrichment spec 要求 class/struct 包含 `template_args` 字段，但实现中未查询
- **预期**: inspect class/struct 时应输出 `template_args`（模板参数）
- **代码位置**: `src/mcp/mod.rs:372-421`（class/struct 分支）
- **建议**: 在 class/struct 分支中增加查询 `json_extract(n.attrs, '$.template_args')` 的逻辑

---

### BUG-04: #include 边的 source_id/target_id 为 0

- **涉及**: 内部能力（include 索引）
- **现状**: 正则提取 `#include` 后写入 edges 表，但 `source_id=0, target_id=0`，无法关联到具体符号
- **预期**: include 关系应关联到符号（至少可查，能用于路径分析）
- **代码位置**: `src/cli/mod.rs:864-886`
- **建议**: 需要从 #include 的文件路径解析出符号节点 ID，再写入 edges

---

## 三、展示问题

### BUG-05: codeloom_check 工具数硬编码

- **涉及**: `codeloom check` CLI 命令
- **现状**: 输出写死 `"MCP tools: 9 tools"`，实际注册 16+ 工具
- **预期**: 应动态获取工具数或更新为实际数量
- **代码位置**: `src/cli/mod.rs:509`
- **建议**: 改为硬编码为实际数量，或找个计数方式来自动更新

---

## 四、代码残留

### BUG-06: 三个 MCP 工具被禁用但代码完整

- **涉及**: `codeloom_status`, `codeloom_get_doc`, `codeloom_query_excel`
- **现状**: 三个工具的函数已完整实现，但 MCP dispatch 入口（`tools_list` 和 `handle_tool_call`）被注释，工具不可用
- **预期**: 需要明确后集中清理——要么恢复启用，要么删除残留代码
- **代码位置**:
  - `status()`: `src/mcp/mod.rs:268-284`
  - `get_doc()`: `src/mcp/mod.rs:845-916`
  - `query_excel()`: `src/mcp/mod.rs:920-1085`
- **建议**: 统一删除这三个函数及其关联代码

---

## 五、工具描述不精确

### BUG-07: codeloom_search kind 可选值列表与实现不符

- **涉及**: `codeloom_search` MCP 工具
- **现状**: 工具描述列了 10 种 kind，实现实际接受 15 种
- **预期**: 规格已改为"可用类型见 codeloom_schema"，需要同步工具描述
- **代码位置**: `src/mcp/mod.rs:34-36`
- **建议**: 改工具描述，删掉 kind 枚举列表，用"可用类型见 codeloom_schema"代替

|---

## 六、实测发现

### BUG-08: inspect class Compaction  enrichment 字段未生效

- **发现时间**: 2026-05-12（手工测试）
- **测试用例**: MCP-12
- **涉及**: `codeloom_inspect` MCP 工具
- **现状**: inspect leveldb 的 `Compaction` 类时，返回 `"edges":{}`（通用 edges 回退），未输出预期的 `bases`/`members`/`methods` 字段
- **预期**: class 应输出 `bases`、`members`、`methods`（和已记录的 `template_args`）
- **代码位置**: `src/mcp/mod.rs:372-421`
- **可能原因**: class 分支的条件判断可能未命中，或 DB 中该类无 `contains:` 边
- **影响**: 🟡 中 — class 类型符号的 inspect 增强可能对部分类不生效
- **状态**: open

---

## Bug 汇总

| ID | 条目 | 类型 | 优先级 | 涉及代码行 |
|----|------|------|--------|-----------|
| BUG-01 | semantic_search 输出不一致 | 输出不一致 | 🟡 中 | mcp:466-477 |
| BUG-02 | list_branches 描述有符号数但不实现 | 描述不实 | 🟢 低 | mcp:207-234 |
| BUG-03 | inspect 缺 template_args | 功能缺失 | 🟢 低 | mcp:372-421 |
| BUG-04 | #include 无符号关联 | 功能缺失 | 🟡 中 | cli:864-886 |
| BUG-05 | check 工具数硬编码 9 tools | 展示错误 | 🟢 低 | cli:509 |
| BUG-06 | 三个工具被禁用（代码残留） | 代码残留 | ⚪ 记录 | mcp:268/845/920 |
| BUG-07 | search kind 列表与实际不符 | 描述不实 | 🟢 低 | mcp:34-36 |
| BUG-08 | inspect class enrichment 未生效 | 功能缺陷 | 🟡 中 | mcp:372-421 |
