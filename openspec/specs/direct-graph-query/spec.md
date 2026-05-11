# direct-graph-query Specification

## Purpose
TBD - created by archiving change remove-legacy-tables. Update Purpose after archive.
## Requirements
### Requirement: 图查询直查 nodes
符号解析函数 SHALL 从 `nodes` 表查询符号 ID，不经过 `symbols` 表。

#### Scenario: resolve_symbol_id
- GIVEN 用户查询符号 "DB::Open" 的调用图
- WHEN resolve_symbol_id 执行
- THEN SQL SHALL 查 `SELECT id FROM nodes WHERE name=?1 AND repo=?2 AND node_type='sym'`
- AND 结合 branches 表过滤分支

#### Scenario: symbol_name_by_id
- GIVEN 需要根据 ID 获取符号名
- WHEN symbol_name_by_id 执行
- THEN SQL SHALL 查 `SELECT name FROM nodes WHERE id=?1`

