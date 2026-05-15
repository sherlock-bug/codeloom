# 专家测试用例规格审查报告

## 审阅原则
- ✅ 用例合理，预期正确
- ⚠️ 有小问题需修正
- ❌ 预期/引据有误
- ➕ 缺少的场景

---

## 一、codeloom_schema（用例 1-5）

### ✅ 用例 1：节点类型数量 15 种
正确。schema-metadata/spec.md 明确说 15 种。

### ❌ 用例 2：description 与输出不一致标注为 🐛
**问题**：此用例将"实际输出 15 种与 description 声明不一致"标记为 🐛（bug）。但：
- 实际输出了 15 种（没错）
- 只是名称不同（namespace→实际无, variable→实际有）
- 规格冲突导致，不是实现 bug
- **修正**：应标记为 ⚠️ "规格冲突"，去掉 🐛

### ❌ 用例 3：边类型 10 vs 11 标注为 🐛
**问题**：schema-metadata/spec.md 本身就说"9 种边类型"且列表明确：`calls, calls_override, inherits, contains, uses, references, returns, param_type, field_type`。schema 实际输出就是这 9 种，**与 schema-metadata 规格一致**。
extended-edge-types/spec.md 定义了另外一套 11 种命名体系（overrides、aliases、includes 等）。
**这是规格之间的冲突，不是实现 bug**。
**修正**：改为 ⚠️ "规格冲突"，去掉 🐛

### ✅ 用例 4：node_kinds 条目结构
正确。schema-metadata/spec.md 明确要求 name/description/example。

### ✅ 用例 5：edge_types 条目结构
正确。schema-metadata/spec.md 明确要求 prefix/direction/source_kinds/target_kinds。

---

## 二、codeloom_inheritance_tree（用例 6-12）

### ❌ 用例 6：HybridLogger 向上查父类 — 预期格式有误
**问题**：预期输出写为 `{root: "HybridLogger", children: [...]}` 扁平格式。
但 inheritance-tree/spec.md 明确说**嵌套树格式**：
> "返回嵌套树：`DB → children: [DBImpl → children: [LevelDBImpl]]`"

每个节点自己就包含 symbol+children，不是扁平 root+children 两层。
实际格式应该是：
```json
{"symbol": "HybridLogger", "children": [
  {"symbol": "FileLogger", "children": []},
  {"symbol": "ConsoleLogger", "children": []}
]}
```
**修正**：调整预期格式为嵌套树

### ✅ 用例 7：HybridLogger 向下查子类（叶子）
概念正确，但格式同上问题。HybridLogger 确实无人继承它。

### ✅ 用例 8：Logger 向上查父类（根类）
概念正确。Logger 确实无父类。但格式问题同用例 6。

### ❌ 用例 9：Logger 向下查子类
正确——Logger 应有 FileLogger 和 ConsoleLogger 为其子孙。
但格式问题同用例 6——预期应为嵌套格式。

### ❌ 用例 11：默认方向
**问题**：说 "spec 说默认为 both，MCP tool description 说默认为 down。两个规格矛盾。⚠️"
- inheritance-tree/spec.md 明确："默认方向为双向" 
- 规格和工具 description 有矛盾，但**规格是唯一判据**。所以应确认默认 direction=both ✅
- **修正**：去掉"矛盾"，以规格为准，标注为 ✅

### ✅ 用例 12：FileLogger 方向 both
概念正确——应同时返回父类和子类。格式问题同用例 6。

---

## 三、codeloom_inspect（用例 13-19）

### ❌ 用例 13-15 引据错误
**问题**：引用了 mcp-full-attribute-json/spec.md 作为输出格式依据。
但 mcp-full-attribute-json/spec.md 是**关于 SEARCH 工具的输出格式**，不是 inspect 工具！

> 该 Spec 标题："MCP 搜索返回完整 JSON 属性"
> 第一句："系统 SHALL 在 **MCP 搜索工具**返回的 JSON 中包含节点的所有属性"

search 和 inspect 是两个不同的 MCP 工具。用 search 的 spec 来要求 inspect 是不正确的。

**正确依据**：codeloom_inspect 的输出格式由 MCP 工具**工具说明**定义（codeloom_schema 中的 tool descriptions）。

**修正**：去掉对 mcp-full-attribute-json 的引用，改为引用 MCP 工具定义。若工具定义也没说 bases/values，就标注"规格不清"

---

## 四、codeloom_neighbor_graph（用例 20-26）

### ⚠️ 用例 22：via_member 前缀非规格定义
**问题**：预期输出含 `via_member:param_type: [LogLevel]`。
neighbor-graph/spec.md 只说"按方向+边类型分组"，没有定义 `via_member:` 这个前缀。
这是实现细节，不是规格要求。
**修正**：改为只断言分组中包含 param_type 邻居，不指定前缀名称。

### ✅ 用例 26：默认 direction=both
正确。neighbor-graph/spec.md 明确："默认方向为双向"。

---

## 五、codeloom_call_graph（用例 27-30）

### ✅ 用例 29：返回文本格式
正确。call-graph-module/spec.md 明确："返回文本格式的调用图"。

### ⚠️ 用例 30：终端节点增强
正确引用了 enhanced-call-graph/spec.md。但需要明确：enhanced-call-graph/spec.md §1.9 说"终端节点附加 uses/references/literals 依赖项"，这是规格要求。

---

## 六、codeloom_impact_analysis（用例 31-33）

### ✅ 用例 33：默认 direction=reverse
正确。impact-analysis/spec.md 明确："默认方向为反向"。

---

## 七、codeloom_path_analysis（用例 34-37）

### ✅ 用例 36：无路径返回空
正确。path-analysis/spec.md 明确：
> "返回 `{\"paths\": [], \"total_found\": 0}`"

### ✅ 用例 37：edge_filter
正确。path-analysis/spec.md 明确：
> "BFS 只沿 calls 和 calls_override 边遍历，不穿类或其他关系"

---

## 八、codeloom_search / list_symbols（用例 38-43）

### ✅ 用例 41：精确名称搜索排第一
正确。bm25-precise-search/spec.md 明确：
> "结果第一项 SHALL 是 DBImpl::CompactMemTable 函数"

### ✅ 用例 43：搜索结果含增强字段
正确。search-enrichment/spec.md 明确：
> "结果 JSON SHALL 包含 `members`（成员字段名，逗号分隔）和 `methods`（方法名，逗号分隔）"

### ✅ 用例 44：空结果
正确。bm25-precise-search/spec.md 明确："返回结果 SHALL 为空数组"。

---

## 九、codeloom_semantic_search（用例 46-47）

### ✅ 用例 47：只搜索符号，不含文档
正确。vector-semantic-search/spec.md 明确：
> "只搜索符号名称通道（symbol_name_vec），不搜索文档、注释或文件"

---

## 十、检测到的缺失场景

### ➕ 需要补充

| # | 工具 | 缺失场景 | 依据 |
|---|------|---------|------|
| M1 | schema | schema 调用不需要 repo/branch 参数 | schema-metadata "无需任何参数" |
| M2 | neighbor_graph | 枚举值的邻居查询（LOG_INFO→LogLevel） | neighbor-graph "枚举值反查使用方" |
| M3 | inheritance_tree | direction=both 在确有父子和叶子的类上 | 当前用例没有同时有父子和叶子的类 |
| M4 | path_analysis | direction=both 的默认值验证 | path-analysis "默认 direction=both" |
| M5 | neighbor_graph | 结构体 BaseConfig 的字段类型邻居（field_type→LogLevel）| 规格要求 field_type 边 |
| M6 | search | kind 过滤参数（search Logger --kind class）| 工具参数定义 |
| M7 | inspect | 自由函数的 inspect（无 parent_class）| 符号属性要求 |
| M8 | call_graph | direction=both（既有 caller 又有 callee 的符号）| 工具参数定义 |

---

## 汇总

| 判定 | 数量 | 主要问题 |
|------|------|---------|
| ✅ 正确 | 28 个 | 基本概念和预期方向正确 |
| ❌ 需要修正 | 8 个 | 引据错误、预期格式错误、规格冲突标注不当 |
| ⚠️ 小问题 | 4 个 | via_member 非规格、引据不精确等 |
| ➕ 缺失 | 8 个 | 需补充 |
