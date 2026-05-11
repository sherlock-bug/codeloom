# unified-fts5-index Specification

## Purpose
CodeLoom 统一全文索引：fts5_all 直连 nodes 表（rowid = id），零中间表，同时覆盖符号名/注释/文档内容/文件名搜索。替代旧的三表分离 FTS5（fts5_sym/fts5_doc/fts5_files）。
## Requirements
### Requirement: FTS5 直连节点表
系统 SHALL 使用 `fts5_all` 虚拟表直连 `nodes` 表，通过 `fts5_all.rowid = nodes.id` 关联，不需要中间映射表。

#### Scenario: FTS5 填充
- WHEN 索引完成填充 FTS5
- THEN 执行 `INSERT INTO fts5_all(rowid, name, content) SELECT id, name, content FROM nodes WHERE repo=?`
- AND fts5_all.rowid 与 nodes.id 一一对应

#### Scenario: 搜索关联
- WHEN 执行 FTS5 搜索
- THEN SQL 为 `fts5_all JOIN nodes ON nodes.id = fts5_all.rowid`
- AND 搜索结果直接包含 nodes 的所有字段

### Requirement: 统一 IDF 域
系统 SHALL 确保所有节点类型在同一 `fts5_all` 表中共享 IDF 计算域，使 BM25 分数跨类型可比。

#### Scenario: 跨类型同词排序
- GIVEN 关键词 "compress" 同时出现在符号名和文档内容中
- WHEN 执行 BM25 搜索
- THEN name 通道（×0.7）的符号 SHALL 排在 content 通道（×0.3）的文档之前
- AND 归一化公式使用 `2.0/(1.0+|score|/8.0)` 减少短名惩罚

### Requirement: 精确节点查询
MCP 工具 SHALL 支持通过 nodes.id 精确查询任意节点信息。

#### Scenario: 按 ID 查节点
- GIVEN nodes 表索引完成
- WHEN MCP 工具收到 `node_id=42` 参数
- THEN 返回 node_type='sym' 的节点完整信息（含 attrs 中的 signature/namespace 等）

