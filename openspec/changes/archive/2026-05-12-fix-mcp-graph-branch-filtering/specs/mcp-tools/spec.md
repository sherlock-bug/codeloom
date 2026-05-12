# Delta for mcp-tools

## MODIFIED Requirements

### Requirement: 图分析工具按分支隔离边查询
所有图分析类 MCP 工具 SHALL 在查询 `edges` 表时包含 `AND branch_id = ?` 过滤条件，确保多分支索引的边数据互不污染。

#### Scenario: 双分支独立图查询
- GIVEN 仓库 leveldb 在 `main` (branch_id=387) 和 `main1` (branch_id=1) 两个分支均已索引
- AND `main` 分支的 `DB::Open` 有 `calls:Get` 边
- AND `main1` 分支的 `DB::Open` 有 `calls:Put` 边
- WHEN 查询 `main1` 分支的 `DB::Open` 的调用者
- THEN 结果 SHALL 仅包含 `calls:Put` 边
- AND 不包含 `main` 分支的 `calls:Get` 边

#### Scenario: 兼容未指定分支的旧数据
- GIVEN `edges` 表中存在 `branch_id = 0` 的旧边数据
- WHEN 图查询按 `branch_id = ?` 过滤
- THEN 查询条件 SHALL 为 `AND (branch_id = ? OR branch_id = 0)` 以确保向后兼容

### Requirement: 边查询不依赖 branch_name 字符串
所有 `edges` 表的查询 SHALL 使用整数 `branch_id` 作为查询参数，而非字符串 `branch_name`，以匹配 `edges.branch_id` 列的 INTEGER 类型。

#### Scenario: 参数化查询
- GIVEN MCP 工具的 `branch` 参数为 "main1"
- WHEN 解析为 SQL 查询参数
- THEN SHALL 通过 `resolve_branch_id()` 获取整数 ID 再传参
- AND 不得使用字符串拼接或 `branch_name` 进行边过滤
