# Delta for fts5-fulltext-index

## MODIFIED Requirements

### Requirement: FTS5 外部内容表自动创建
系统 SHALL 在数据库 schema 初始化时创建统一 FTS5 外部内容虚拟表 `fts5_all(source_type, name, content)`，替代原有的 `fts5_sym` 和 `fts5_doc` 两张独立表。

#### Scenario: schema 初始化创建 FTS5 表
- WHEN `ensure_schema()` 在连接新数据库时执行
- THEN 创建 `fts5_all` 虚拟表（外部内容，无 content_rowid——各源使用独立 rowid 域）

#### Scenario: 旧 schema 自动迁移
- GIVEN 已有数据库包含 `fts5_sym` 或 `fts5_doc` 表
- WHEN `ensure_schema()` 执行
- THEN DROP 旧 FTS5 表
- AND 创建 `fts5_all`
- AND 从 symbols、doc_nodes、files 表重新填充内容

### Requirement: FTS5 索引在 CLI 索引时自动填充
`codeloom index` 命令 SHALL 在符号和文档索引完成后，将所有三类内容（符号名/签名/注释、文档标题/内容、文件名/摘要）写入统一的 `fts5_all` 表。

#### Scenario: 索引填充 FTS5 内容
- WHEN `codeloom index /project --repo myrepo --branch main` 完成符号和文档索引
- THEN 系统将 symbols 表内容写入 `fts5_all`（source_type='sym'）
- AND 将 doc_nodes 表内容写入 `fts5_all`（source_type='doc'）
- AND 将 files 表内容写入 `fts5_all`（source_type='file'）
