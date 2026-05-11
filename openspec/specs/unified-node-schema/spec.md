# unified-node-schema Specification

## Purpose
CodeLoom 统一节点表：用单一 nodes 表（id INTEGER PRIMARY KEY AUTOINCREMENT）存储符号/文档/文件三类节点，替代原来的 symbols/doc_nodes/files 三表分离架构。
## Requirements
### Requirement: 统一节点表
系统 SHALL 使用单一 `nodes` 表存储所有可索引节点（符号/文档/文件），以 `id INTEGER PRIMARY KEY AUTOINCREMENT` 作为全局唯一标识。

#### Scenario: 节点表创建
- WHEN `ensure_schema()` 首次执行
- THEN 创建 nodes 表，含 id/repo/node_type/name/content/file_path/line_start/content_hash/branch_name/kind/attrs 列
- AND id 为自增整数主键，三源节点统一编号

#### Scenario: 符号节点存储
- GIVEN Clang 解析器提取函数 `DB::Open`
- WHEN 写入 nodes 表
- THEN node_type SHALL 为 'sym'
- AND name 为符号名，content 为 kind+doc_comment
- AND kind 为 'function'，attrs 含 signature/namespace/access 等

#### Scenario: 文档节点存储
- GIVEN markdown 文档解析为多个 section
- WHEN 写入 nodes 表
- THEN node_type SHALL 为 'doc'
- AND name 为标题或首行，content 为 section_path+正文
- AND attrs 含 level/section_path

### Requirement: 删除冗余列
系统 SHALL 不保留 `sid` 哈希列（用 id 替代）和 `parent_id` 列（用 edges contains 边替代）。

#### Scenario: sid 列删除
- GIVEN v0.9 migration 执行后
- THEN nodes 表 SHALL 不含 sid 列
- AND edges 表继续使用整数 source_id/target_id 引用 nodes.id

#### Scenario: parent_id 用边替代
- GIVEN 文档章节 chapter-1 包含 3 个 chunk
- WHEN 索引完成
- THEN edges 表 SHALL 包含 3 条 edge_type='contains' 的边
- AND chapter-1 的 chunk 查询通过 graph query 走 edges

