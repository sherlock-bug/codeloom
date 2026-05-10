# doc-dedup

## Purpose
CodeLoom doc-dedup 功能域。本规范描述此功能的需求和行为。

## Purpose
CodeLoom 文档内容去重模块。通过 blake3 哈希对文档标题、路径和内容计算 content_hash，在增量索引时自动跳过无变化文档的重复向量化，大幅减少重复计算开销。

## Requirements

### Requirement: 文档内容哈希去重
系统 SHALL 在索引文档时为每个文档节点计算 `content_hash`（基于 `title:section_path:content` 的格式化拼接），存储到 `doc_nodes` 表。再次索引同一文档时，若 hash 未变则跳过向量化。

#### Scenario: 无变化文档跳过向量化
- GIVEN 已索引 10 份 Markdown 文档
- WHEN 再次执行 `codeloom index` 且文档内容未变更
- THEN 系统 SHALL 检测到 hash 匹配，跳过所有 10 份文档的向量化，输出 "48 skipped"

#### Scenario: 部分文档变更
- GIVEN 已索引 10 份文档，其中 3 份内容发生变化
- WHEN 再次执行 `codeloom index`
- THEN 系统 SHALL 仅对 3 份变更文档重新向量化，其余 7 份跳过

### Requirement: doc_nodes UNIQUE 约束
系统 SHALL 在 `doc_nodes` 表的 `(repo, branch_name, doc_path)` 上添加 UNIQUE 约束，使用 `INSERT OR REPLACE` 或 `ON CONFLICT` 处理重复文档。

#### Scenario: 同一文档重复索引
- GIVEN 同一个 repo+branch 下存在文件 `docs/api.md`
- WHEN 两次索引均包含该文件
- THEN 第二次索引 SHALL 更新已有行而非插入重复行，content_hash 覆盖旧值

### Requirement: 存量数据库 content_hash 回填
系统 SHALL 在数据库迁移时检测 `doc_nodes` 表是否缺少 `content_hash` 列或列为 NULL，若为 NULL 则在首次 `index_doc_vectors` 时全量计算并回填。

#### Scenario: 从旧版本升级
- GIVEN 已有 CodeLoom 索引数据库，doc_nodes 表中 content_hash 均为 NULL
- WHEN 升级到 v0.4.0 后首次运行 `codeloom index`
- THEN 系统 SHALL 对 55 行文档计算并写入 content_hash，此后增量索引时正常跳过无变化行
