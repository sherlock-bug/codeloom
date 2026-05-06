# fts5-fulltext-index Specification

## Purpose
TBD - created by archiving change hybrid-search. Update Purpose after archive.
## Requirements
### Requirement: FTS5 外部内容表自动创建
系统 SHALL 在数据库 schema 初始化时为每个仓库创建 FTS5 外部内容虚拟表，覆盖 symbols 和 doc_nodes 两表。

#### Scenario: schema 初始化创建 FTS5 表
- WHEN `ensure_schema()` 在连接新数据库时执行
- THEN 创建 `fts5_sym` 虚拟表（外部内容，引用 symbols 表的 name + file_path + signature 列）
- AND 创建 `fts5_doc` 虚拟表（外部内容，引用 doc_nodes 表的 title + section_path + content 列）
- AND 表使用 `content_rowid` 参数关联到原表 rowid

#### Scenario: schema 初始化时 FTS5 表已存在
- WHEN `ensure_schema()` 在已有 FTS5 表的数据库上执行
- THEN 使用 `IF NOT EXISTS` 跳过创建，不报错

### Requirement: FTS5 索引在 CLI 索引时自动填充
`codeloom index` 命令 SHALL 在符号和文档索引完成后调用 FTS5 内容填充。

#### Scenario: 索引填充 FTS5 内容
- WHEN `codeloom index /project --repo myrepo --branch main` 完成符号和文档索引
- THEN 系统将 symbols 表中 name、file_path、signature 内容写入 `fts5_sym`
- AND 系统将 doc_nodes 表中 title、section_path、content 内容写入 `fts5_doc`
- AND 写入操作使用 `INSERT OR REPLACE` 语义（支持增量索引时的幂等更新）

#### Scenario: 符号无 signature 时的填充
- WHEN 符号的 signature 字段为 NULL 或空字符串
- THEN FTS5 只填充 name 和 file_path，不因空值报错

### Requirement: FTS5 BM25 相关性查询
系统 SHALL 支持通过 FTS5 MATCH 进行全文搜索并按 BM25 相关性排序。

#### Scenario: 关键词精确匹配
- WHEN 查询 `AuthService`
- THEN `fts5_sym MATCH 'AuthService'` 按 BM25 分数降序返回匹配的符号 rowid
- AND 通过 rowid JOIN 回 symbols 表获取完整信息（name/kind/file_path/line_start）

#### Scenario: 多词组合查询
- WHEN 查询 `memory alloc`
- THEN `fts5_sym MATCH 'memory AND alloc'` 返回同时包含两词的文档，BM25 排序

#### Scenario: 文档内容全文搜索
- WHEN 查询 `配置说明`
- THEN `fts5_doc MATCH '配置说明'` 返回匹配的文档 rowid，按 BM25 排序

### Requirement: FTS5 幂等性
重复执行索引 SHALL 不产生重复 FTS5 条目。

#### Scenario: 重复索引同一仓库
- WHEN 对同一仓库执行两次 `codeloom index`
- THEN 第二次索引前执行 `DELETE FROM fts5_sym; DELETE FROM fts5_doc;` 清空旧数据
- AND 重新填充后 FTS5 内容与 symbols/doc_nodes 表保持一致

