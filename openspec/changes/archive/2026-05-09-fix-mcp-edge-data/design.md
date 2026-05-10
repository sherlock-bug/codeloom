# Design: fix-mcp-edge-data

## Context
`mcp-analysis-tools`（已归档）的 leveldb 实测暴露了 4 个 bug。经 DB 审计确认根因如下：

| Bug | 工具 | 根因 | 修复位置 |
|-----|------|------|---------|
| 1 | call_graph | LIKE `%{name}%` 匹配到 string_literal | `call_graph.rs:28` |
| 2 | neighbor_graph | get_edges 未过滤 target_id=0 断边 | `graph.rs:38-69` |
| 3 | impact_analysis | DB 中无入边——索引器层问题，非工具 bug | 本次不修 |
| 4 | inheritance_tree | inherits 查询不校验 kind | `mod.rs:521,539` |

## Goals / Non-Goals
- **Goals**: 修复 1、2、4，过滤无效数据，输出干净结果
- **Non-Goals**: 修复索引器边提取（bug 3 的根因），改进 call_graph 终端节点依赖

## Decisions

### Decision 1: call_graph LIKE 查询加 kind 过滤
选择在 SQL 中加 `AND s.kind NOT IN ('string_literal', 'enum_value')`，而非只取第一个精确匹配结果。
- 理由：LIKE 的语义是模糊回退，但应该限定在函数/方法域内
- 替代方案：改用 `resolve_symbol_id` 统一入口。拒绝原因：call_graph 的 LIKE 回退逻辑与 graph.rs 的精确匹配不同，混用会导致行为不一致

### Decision 2: 在 get_edges 中统一过滤 target_id=0
选择在 `get_edges()` 的 SQL 中加 `AND e.target_id != 0`，而非在各调用点分别过滤。
- 理由：所有图遍历工具（neighbor_graph、path_analysis、impact_analysis）都经过 get_edges，统一过滤一劳永逸
- 不影响：build_forward_adj/build_reverse_adj 不走 get_edges，但这两个用于 BFS/DFS，target_id=0 的边指向无效节点，BFS 遇到 ID=0 找名字时会自动降级为 `id_0` 占位——虽不优雅但不破坏逻辑

### Decision 3: inheritance_tree 内联 SQL 加 kind 校验
选择在现有 SQL 中追加 `AND (s.kind = 'class' OR s.kind = 'struct')`。
- 理由：最小改动，不引入新函数
- 风险：如果索引器给 class 标了错误的 kind（如 `function`），真继承关系会被误杀。这是索引器的数据质量问题，应该修源头

## Risks / Trade-offs
| 风险 | 缓解 |
|------|------|
| kind 过滤依赖索引器 kind 标签准确性 | kind 标签从 tree-sitter 节点类型映射，出错的概率极低 |
| target_id=0 过滤可能掩盖索引器 bug | 保留索引器侧的 broken edge 计数，不在此 change 中处理 |
