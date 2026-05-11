# Proposal: unified-fts5-index

## Intent
解决跨 FTS5 表 BM25 分数不可比导致搜索排序错误的问题。当前 `fts5_sym`、`fts5_doc`、`fts5_files` 三张独立 FTS5 表各自计算 IDF，导致相同关键词在不同表中的 BM25 raw score 尺度不同，无法公平排序。（实测：英文 noise probe 也对所有表返回 0 结果，但 `fts5_sym` 和 `fts5_doc` 对真实词 `compress` 的 raw score 分别为 -11.78 和 -3.25，doc 以 ×0.3 权重也能赢过 sym ×0.7。）

## Scope

In scope:
- 创建统一 FTS5 外部内容表 `fts5_all(source_type, name, content)`，替代 `fts5_sym`、`fts5_doc`、`fts5_files` 三张独立表
- 搜索查询改为针对 `fts5_all` 的 name 列（×0.7）和 content 列（×0.3），统一 IDF 域
- Schema migration：自动检测并迁移已有 DB
- CLI search、MCP search、calibration 全部使用新表

Out of scope:
- 不改动向量搜索（vec0 表不在本变更范围）
- 不改动 title 提取逻辑（另案处理）
- 不改动日志默认行为
- 不改动 BM25 噪音标定空结果逻辑

## Approach
将三张独立 FTS5 外部内容表合并为一张 `fts5_all`，用 `source_type` 列区分来源。各源类型映射：
- sym → name=符号名, content=signature+kind+doc_comment
- doc → name=title, content=section_path+content
- file → name=file_path, content=summary

搜索按两个通道查询：name 列（×0.7）和 content 列（×0.3），所有来源在同一 IDF 域公平竞争。方案 A 的 per-type boost 不再需要——统一表天然解决跨表不可比问题。

## Capabilities

### New Capabilities
- `unified-fts5-table`: 单一 `fts5_all` 外部内容表，覆盖符号/文档/文件三类内容，统一 IDF 域确保跨类型 BM25 分数可比

### Modified Capabilities
- `fts5-fulltext-index`: FTS5 表结构从三表改为单表，索引填充逻辑同步调整
- `search-name-comment-split`: 搜索函数改为针对 `fts5_all` 的 name/content 列查询
- `bm25-precise-search`: 去掉跨表归一化 hack，直接用统一表的 BM25 分数 × 通道权重
