# CodeLoom 已知问题 (Bug)

> 生成时间：2026-05-12
> 最后更新：2026-05-12（已修复 BUG-01/02/03/05/06/07）

---

## 一、输出不一致

### BUG-01: codeloom_semantic_search 输出与 BM25 搜索不一致

- **涉及**: `codeloom_semantic_search` MCP 工具
- **状态**: ✅ **已修复** — 2026-05-12
- **修复**:
  - 添加 `id` 字段（`"id": r.id`）
  - 移除 `"type": "code"` 硬编码，改用 `"type": r.hit_type`

---

### BUG-02: codeloom_list_branches 描述与实际不符

- **涉及**: `codeloom_list_branches` MCP 工具
- **状态**: ✅ **已修复** — 2026-05-12
- **修复**: 工具描述去掉"及各自符号数量"和"返回分支名和符号数"，统一为"列出指定仓库的所有已索引分支。返回分支名"

---

## 二、功能缺失

### BUG-03: codeloom_inspect class/struct 缺少 template_args

- **涉及**: `codeloom_inspect` MCP 工具
- **状态**: ✅ **已修复** — 2026-05-12
- **修复**: 在 class/struct 分支中增加查询 `json_extract(n.attrs, '$.template_args')` 的逻辑

---

### BUG-04: #include 边的 source_id/target_id 为 0

- **涉及**: 内部能力（include 索引）
- **现状**: 正则提取 `#include` 后写入 edges 表，但 `source_id=0, target_id=0`，无法关联到具体符号
- **预期**: include 关系应关联到符号（至少可查，能用于路径分析）
- **代码位置**: `src/cli/mod.rs:864-886`
- **建议**: 需要从 #include 的文件路径解析出符号节点 ID，再写入 edges
- **状态**: ⏳ 待修（需要符号解析逻辑，非简单改动）

---

## 三、展示问题

### BUG-05: codeloom_check 工具数硬编码

- **涉及**: `codeloom check` CLI 命令
- **状态**: ✅ **已修复** — 2026-05-12
- **修复**: `"9 tools"` → `"12 tools"`

---

## 四、代码残留

### BUG-06: 三个 MCP 工具被禁用但代码完整

- **涉及**: `codeloom_status`, `codeloom_get_doc`, `codeloom_query_excel`
- **状态**: ✅ **已清理** — 2026-05-12
- **修复**: 删除三个函数的完整实现、tools_list 中的 DISABLED 注释、handle_tool_call 中的禁用匹配臂

---

## 五、工具描述不精确

### BUG-07: codeloom_search kind 可选值列表与实现不符

- **涉及**: `codeloom_search` MCP 工具
- **状态**: ✅ **已修复** — 2026-05-12
- **修复**: 工具描述中 kind 枚举列表改为"可用类型见 codeloom_schema"

---

## 六、实测发现

### BUG-08: inspect class Compaction enrichment 字段未生效

- **发现时间**: 2026-05-12（手工测试）
- **测试用例**: MCP-12
- **涉及**: `codeloom_inspect` MCP 工具
- **根因**: 双重问题：
  1. 索引器写入的 `edge_type` 格式为 `contains:MethodName`（含目标名后缀），查询端用 `='contains:'` 精确匹配永远不匹配
  2. 索引器 INSERT edges 时未设置 `branch_id`（全部为 DEFAULT 0），查询端过滤 `e.branch_id={id}` 查不到任何边
- **修复**（最终方案 — 索引器端根治）:
  - `clang/ast.rs`: 3 处 `edge_type` 去掉 `:Name` 后缀（`contains:Name` → `contains`，`inherits:Name` → `inherits`，`overrides:Name` → `overrides`）
  - `clang/mod.rs`: INSERT edges 增加 `branch_id` 列
  - 查询端使用 `='contains'` 精确匹配 + `e.branch_id={id}`，零 workaround
- **验证**（reindex leveldb 后）: `codeloom_inspect Compaction` 正确输出 methods，无 `"edges":{}` 回退
- **影响**: 🟡 中 — 已修复所有 class/struct/enum 类型符号的 inspect 增强
- **状态**: ✅ **已修复** — 2026-05-12（最终方案）

---

## Bug 汇总

| ID | 条目 | 类型 | 优先级 | 状态 |
|----|------|------|--------|------|
| BUG-01 | semantic_search 输出不一致 | 输出不一致 | 🟡 中 | ✅ 已修复 |
| BUG-02 | list_branches 描述有符号数但不实现 | 描述不实 | 🟢 低 | ✅ 已修复 |
| BUG-03 | inspect 缺 template_args | 功能缺失 | 🟢 低 | ✅ 已修复 |
| BUG-04 | #include 无符号关联 | 功能缺失 | 🟡 中 | ⏳ 待修 |
| BUG-05 | check 工具数硬编码 9 tools | 展示错误 | 🟢 低 | ✅ 已修复 |
| BUG-06 | 三个工具被禁用（代码残留） | 代码残留 | ⚪ 记录 | ✅ 已清理 |
| BUG-07 | search kind 列表与实际不符 | 描述不实 | 🟢 低 | ✅ 已修复 |
| BUG-08 | inspect class enrichment 未生效 | 功能缺陷 | 🟡 中 | ✅ 已修复 |
