# Delta for unified-fts5-table

## ADDED Requirements

### Requirement: 统一 FTS5 外部内容表
系统 SHALL 使用单一 FTS5 外部内容虚拟表 `fts5_all` 存储所有可搜索内容，替代 `fts5_sym`、`fts5_doc`、`fts5_files` 三张独立表。

#### Scenario: 表结构创建
- WHEN `ensure_schema()` 首次执行或检测旧表存在时
- THEN 创建 `CREATE VIRTUAL TABLE fts5_all USING fts5(source_type, name, content)`
- AND 三列分别为来源类型、可搜索名称、可搜索内容

#### Scenario: 三源统一存储
- GIVEN 符号表有 2000 行、文档表有 200 行、文件表有 50 行
- WHEN 索引完成并填充 `fts5_all`
- THEN fts5_all SHALL 包含约 2250 行
- AND source_type 列区分 sym/doc/file

### Requirement: 跨类型 BM25 分数可比
同一关键词在 `fts5_all` 中的 BM25 分数 SHALL 跨来源类型可比，不因表不同而产生偏差。

#### Scenario: 同词跨类型排序
- GIVEN 搜索 "compress" 同时命中符号名和文档内容
- WHEN 对 name 列执行 `fts5_all MATCH`
- THEN 符号的 name 命中 BM25 分数 SHALL 与文档的 title 命中在同一 IDF 尺度下计算
- AND 分数排序 SHALL 仅由关键词在各文档中的 IDF×TF 决定，不因来源表不同而偏移

### Requirement: 按来源类型过滤
系统 SHALL 支持通过 `source_type` 列过滤 FTS5 搜索结果，限定在特定来源类型。

#### Scenario: 只搜代码符号
- GIVEN fts5_all 包含 sym、doc、file 三类内容
- WHEN 执行 `fts5_all MATCH 'source_type:sym AND name:compress*'`
- THEN 结果 SHALL 只包含 source_type 为 sym 的条目

### Requirement: 索引填充幂等性
重复索引 SHALL 不产生重复的 FTS5 条目。

#### Scenario: 重复索引
- GIVEN 仓库已索引一次
- WHEN 再次执行 `codeloom index`
- THEN 索引前执行 `DELETE FROM fts5_all` 清空旧数据
- AND 重新填充后内容与各源表一致
