# Delta for fix-neighbor-broken-edges

## MODIFIED Requirements

### Requirement: 图遍历过滤断边
系统 SHALL 在所有基于 edges 表的图遍历查询中过滤 `target_id = 0` 的断边，避免输出空字符串邻居和无效路径节点。

#### Scenario: neighbor_graph 不含空字符串邻居
- GIVEN DBImpl::Get 的 outgoing edges 中包含 16 条 target_id=0 的断边
- WHEN 查询 `codeloom_neighbor_graph(symbol="DBImpl::Get", depth=1, direction="both")`
- THEN forward 中的 calls 列表不含空字符串条目

#### Scenario: path_analysis 不走过断边
- GIVEN 存在 source_id=A, target_id=0 的断边
- WHEN 查询 `codeloom_path_analysis(source="A", target="B")`
- THEN 路径不经过 target_id=0 的边

#### Scenario: impact_analysis 不包含断边影响
- GIVEN 符号 A 有一条 target_id=0 的 calls 边
- WHEN 查询 `codeloom_impact_analysis(symbol="A", direction="forward")`
- THEN affected 列表不含 id=0 相关的无效条目
