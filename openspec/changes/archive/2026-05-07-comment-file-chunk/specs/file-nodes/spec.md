# Delta for file-nodes

## ADDED Requirements

### Requirement: 文件节点存储
系统 SHALL 维护 files 表存储每个已索引文件的节点信息，包含 file_path、file_type（code/doc）、summary（代码文件为注释摘要，文档文件为空或首段摘要）和 content_hash（增量更新用）。

#### Scenario: 代码文件节点
- GIVEN leveldb 源码文件 `db/db_impl.cc` 包含多处注释
- WHEN 索引该文件
- THEN files 表 SHALL 新增一行，file_type="code"，summary 包含该文件所有注释的拼接文本

#### Scenario: 文档文件节点
- GIVEN `doc/impl.md` 被索引
- WHEN 文档索引完成
- THEN files 表 SHALL 新增一行，file_type="doc"，file_path="doc/impl.md"

### Requirement: 文件 FTS5 索引
系统 SHALL 创建 fts5_files FTS5 索引，包含 file_path 和 summary 列，使文件名和代码文件注释可被关键词搜索命中。

#### Scenario: 按文件名搜索
- GIVEN 用户搜索 "db_impl"
- WHEN FTS5 搜索 files
- THEN `db/db_impl.cc` SHALL 出现在结果中

#### Scenario: 按代码注释搜索文件
- GIVEN 文件 `benchmarks/db_bench.cc` 的 summary 包含 "Performance benchmark"
- WHEN 用户搜索 "benchmark"
- THEN 该文件 SHALL 出现在 FTS5 结果中

### Requirement: 文件向量嵌入
系统 SHALL 创建 file_vec_* 向量表，为每个文件节点生成向量嵌入，嵌入文本格式为 `file_path | summary`。

#### Scenario: 文件语义搜索
- GIVEN 文件 `db/recover.cc` 的注释包含 "恢复数据库"
- WHEN 用户用中文搜索 "恢复"
- THEN 向量语义搜索 SHALL 返回该文件节点

### Requirement: 搜索结果包含文件类型
系统 SHALL 在混合搜索结果中返回文件类型结果（hit_type="file"），与 code 和 doc 结果一同排序显示。

#### Scenario: 混合搜索含文件
- GIVEN 用户搜索 "compaction"
- WHEN 混合搜索执行
- THEN 结果 SHALL 包含 code、doc 和 file 三种类型，按加权融合分数排序
