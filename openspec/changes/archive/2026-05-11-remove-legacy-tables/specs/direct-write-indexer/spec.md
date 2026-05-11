# Delta for direct-write-indexer

## ADDED Requirements

### Requirement: 索引器直写 nodes 表
索引器（Clang/tree-sitter/doc）SHALL 直接将符号/文档/文件写入 `nodes` 表，不再经过 symbols/doc_nodes/files 旧表。

#### Scenario: Clang 解析器写 nodes
- GIVEN Clang 解析器完成一个翻译单元的符号提取
- WHEN 写入数据库
- THEN INSERT SHALL 进入 `nodes` 表，node_type='sym'
- AND `branches` 表同步插入对应记录（node_id, repo, branch_name）

#### Scenario: 文档索引写 nodes
- GIVEN 文档解析器处理一个文档文件
- WHEN 写入数据库
- THEN INSERT SHALL 进入 `nodes` 表，node_type='doc'
- AND 内容 SHALL 存储在 content 字段

#### Scenario: 文件节点写 nodes
- GIVEN 索引器扫描到一个代码/文档文件
- WHEN 记录文件节点
- THEN INSERT SHALL 进入 `nodes` 表，node_type='file'

### Requirement: edges 引用 nodes.id
边表 SHALL 以 `nodes.id` 作为 source_id/target_id 的来源，resolve_target SHALL 查 `nodes` 而非 `symbols`。

#### Scenario: 边写入一致性
- GIVEN 索引器解析到函数 A 调用函数 B
- WHEN 写入边记录
- THEN source_id SHALL 是 nodes 表中函数 A 的 id
- AND target_id SHALL 是 nodes 表中函数 B 的 id

### Requirement: 数据库初始化简化
schema.rs SHALL 只创建 nodes/edges/branches/fts5_all/git_index_state/branch_glossary/doc_images 等当前使用的表。

#### Scenario: 初始化新 DB
- GIVEN 一个空数据库文件
- WHEN 运行 schema::run()
- THEN 不创建 symbols、doc_nodes、files、fts5_sym、fts5_doc、fts5_files 等旧表
- AND 不运行任何迁移函数
