# Design: unified-fts5-index

## Context
CodeLoom 当前使用三张独立 FTS5 外部内容表进行 BM25 关键词搜索：
- `fts5_sym(name, file_path, signature, kind, doc_comment)` — 符号
- `fts5_doc(title, section_path, content)` — 文档
- `fts5_files(file_path, summary)` — 文件

BM25 的 IDF（逆文档频率）分量在每张表内独立计算。由于表行数不同（sym 2076、doc 240、file ~60），相同关键词在不同表中得到不可比的 raw score。现有 `1/(0.5+|score|)` 归一化无法消除此偏差。

## Goals / Non-Goals

**Goals:**
- 创建统一 `fts5_all(source_type, name, content)` 表，所有内容共享同一 IDF 域
- 搜索查询简化为针对 name 列（×0.7）和 content 列（×0.3）两个通道
- 兼容已有 DB 的自动 migration

**Non-Goals:**
- 不改动向量搜索（vec0）
- 不改动 title 提取
- 不改动日志/标定逻辑

## Database Design

### 新表
```sql
CREATE VIRTUAL TABLE fts5_all USING fts5(source_type, name, content);
```

### 内容映射
| source_type | name 列 | content 列 |
|---|---|---|
| sym | symbol.name | signature + " " + kind + " " + doc_comment |
| doc | doc_nodes.title | section_path + " " + content |
| file | files.file_path | summary |

### 填充
```sql
-- Symbol
INSERT INTO fts5_all(source_type, name, content)
SELECT 'sym', s.name, COALESCE(s.signature,'') || ' ' || s.kind || ' ' || COALESCE(s.doc_comment,'')
FROM symbols s JOIN branches b ON b.symbol_id=s.id WHERE s.repo=? AND b.branch_name=?;

-- Doc
INSERT INTO fts5_all(source_type, name, content)
SELECT 'doc', d.title, COALESCE(d.section_path,'') || ' ' || d.content
FROM doc_nodes d WHERE d.repo=?;

-- File
INSERT INTO fts5_all(source_type, name, content)
SELECT 'file', f.file_path, COALESCE(f.summary,'')
FROM files f WHERE f.repo=?;
```

### Migration
1. 检测旧 FTS5 表存在 → DROP `fts5_sym`, `fts5_doc`, `fts5_files`
2. 创建 `fts5_all`
3. 重新填充（使用 rebuild_fts5 流程）

## Decisions

### Decision: 统一表而非 per-type boost
选择方案 B（统一 FTS5 表）而非方案 A（per-type score boost），因为：
- 方案 A 的 boost 系数需要手动调参，换一个仓库 ID 可能不适用
- 统一表从根本上消除跨表不可比问题，不依赖经验参数
- 搜索逻辑更简洁：单表查询 vs 当前 6 路查询

### Decision: content 列合并多字段
将 signature、kind、doc_comment 合并为 content 列而非独立列，因为：
- FTS5 列过滤查询的性能不受列数影响
- 但 content 列的排序统一，不需要在搜索代码中做多列 OR
- 保持 schema 简单（3 列而非 7+ 列）

### Decision: 保留 source_type 列
`source_type` 列为 FTS5 过滤列，允许按来源过滤（如只搜代码）。当前搜索不强制过滤，但为未来扩展留空间。

## Migration Plan
1. `ensure_schema()` 中检测 `fts5_sym` 存在 → 执行 migration
2. Migration：DROP 旧表 → CREATE `fts5_all` → 重建填充数据
3. 首次索引后自动填充，无需用户干预

## Risks / Trade-offs
| 风险 | 缓解措施 |
|------|---------|
| FTS5 表重建期间搜索不可用 | 索引操作本身是批量写入，重建在索引末尾完成 |
| 旧 DB 无 `fts5_all` 搜索报错 | Migration 在 `ensure_schema()` 自动执行 |
| content 列过长影响 BM25 | FTS5 BM25 自带长度归一化，长文本不会主导排序 |
