# Proposal: 修复 MCP 分析工具 4 项数据质量 bug

## Why
`mcp-analysis-tools` 归档后实测发现 4 个 bug，影响工具正确性。其中 3 个是工具层逻辑缺陷，1 个是索引器边的质量缺陷（不在本次修复范围）。

## What Changes
- **call_graph LIKE fallback**：排除 `string_literal` 类型符号，避免匹配到 FLAGS 声明字符串
- **neighbor_graph / get_edges**：过滤 `target_id = 0` 的断边，消除空字符串邻居
- **inheritance_tree**：双向 `inherits` 查询增加 `class/struct` kind 检查，消除假阳性
- **impact_analysis 空结果**：经 DB 审计确认非工具 bug——`DBImpl::Get` 在索引库中确实无入边（调用方通过 `DB*` 基类指针访问，索引器未解析虚调度）。记录为已知限制，不在本次修复。

## Capabilities

### New Capabilities
- `fix-call-graph-like-fallback`: call_graph 的 LIKE 降级查询排除非函数/方法类型符号
- `fix-neighbor-broken-edges`: 图遍历工具的所有边查询过滤 target_id=0 的断边
- `fix-inheritance-kind-check`: 继承树查询增加 source/target 的 class/struct 类型校验

## Impact
- 受影响文件：`src/query/call_graph.rs`、`src/query/graph.rs`、`src/mcp/mod.rs`（inheritance_tree 内联 SQL）
- 不影响索引器、schema、CLI
- 测试：需在 leveldb 上重索引验证 3 项修复，76 现有测试必须全过
