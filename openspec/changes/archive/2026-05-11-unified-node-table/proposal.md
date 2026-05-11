# Proposal: unified-node-table

## Intent
合并 `symbols`、`doc_nodes`、`files` 三张表为单一 `nodes` 表，以 `id INTEGER PRIMARY KEY AUTOINCREMENT` 作为全局唯一键，消除跨表 ID 冲突、`sid` 哈希列、`parent_id` 冗余列、中间映射表，实现 FTS5 直连节点表，支持 MCP 精确节点查询。

## Scope

In scope:
- 创建 `nodes` 表，字段：id、repo、node_type、name、content、file_path、line_start、content_hash、branch_name、kind、attrs(JSON)
- 迁移三张旧表数据到 nodes，保留原有 branches / edges / 向量表
- 删除 `sid` 列（用 id 替代）、`parent_id` 列（用 edges contains 边替代）
- `fts5_all` 通过 `fts5_all.rowid = nodes.id` 直连（id 全局唯一，不需要中间表）
- 更新 indexer 层（clang/tree-sitter/doc parsers）写入 nodes
- 更新 query 层（图分析、搜索）读取 nodes
- 更新 MCP/CLI 工具适配新 schema
- 搜索排序归一化公式修正（解决 BM25 跨源不可比）

Out of scope:
- 不改动向量搜索（vec0 表）
- 不改动 edges 表结构
- 不改动 branches 表结构
- 不改动标题提取逻辑（另案）
- 不改动日志/标定逻辑

## Approach
1. Schema v0.9 migration：创建 nodes 表，迁移旧数据，DROP 旧表
2. 统一存储层：`src/storage/nodes.rs` 代替 `symbols.rs`
3. Indexer 适配：所有解析器写入 nodes 而非 symbols/doc_nodes/files
4. edges 文档层级：doc section→chunk 用 `contains` 边
5. FTS5 简化：`fts5_all JOIN nodes ON nodes.id = fts5_all.rowid`
6. 归一化公式修正：`2.0/(1.0+|score|/8.0)`

## Capabilities

### New Capabilities
- `unified-node-schema`: 单一 nodes 表，id 全局唯一，attrs JSON 存类型特有字段
- `unified-fts5-index`: fts5_all 直连 nodes 表，统一 IDF 域搜索
- `edge-based-doc-hierarchy`: 文档章节层级用 edges contains 边替代 parent_id

### Modified Capabilities
- 所有涉及 symbols/doc_nodes/files 的代码模块
