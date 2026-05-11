# Design: unified-node-table

## Context
CodeLoom 当前使用三张表存储不同类型节点：
- `symbols` (id, repo, name, kind, signature, sid, doc_comment, file_path, ...)
- `doc_nodes` (id, repo, title, section_path, content, level, parent_id, ...)  
- `files` (id, repo, file_path, summary, ...)

问题：
1. 三表独立 AUTOINCREMENT，id 重叠，无法全局唯一标识节点
2. FTS5 需要中间表或 offset hack 才能关联
3. `sid` 文本哈希列冗余（edges 用整数 id，不用 sid）
4. `parent_id` 可以用 `contains` 边替代

## Goals / Non-Goals
**Goals:**
- 单一 `nodes` 表，id 全局唯一
- FTS5 直连（`fts5_all.rowid = nodes.id`）
- 删除 sid、parent_id 列
- `contains` 边替代文档层级

**Non-Goals:**
- 不改 edges 表结构
- 不改 branches 表
- 不改向量表

## Database Design

### nodes 表
```sql
CREATE TABLE nodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,   -- 全局唯一键
    repo TEXT NOT NULL,
    node_type TEXT NOT NULL,   -- 'sym' | 'doc' | 'file'
    name TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',  -- FTS5 可搜索文本
    file_path TEXT NOT NULL DEFAULT '',
    line_start INTEGER DEFAULT 0,
    content_hash TEXT DEFAULT '',
    branch_name TEXT DEFAULT 'main',
    kind TEXT DEFAULT '',        -- sym only，需索引
    attrs TEXT DEFAULT '{}'      -- JSON: {signature, namespace, access, is_virtual, is_definition, is_external, template_args, level, section_path, ...}
);
CREATE INDEX idx_nodes_repo ON nodes(repo);
CREATE INDEX idx_nodes_type ON nodes(node_type);
CREATE INDEX idx_nodes_kind ON nodes(kind);
CREATE INDEX idx_nodes_hash ON nodes(content_hash);
CREATE UNIQUE INDEX idx_nodes_uniq ON nodes(repo, name, file_path, content_hash);
```

### 类型字段映射
| node_type | name | content | kind | attrs 关键字段 |
|-----------|------|---------|------|---------------|
| sym | 符号名 | kind + " " + doc_comment | function/class/... | signature, namespace, access, is_virtual, is_definition, is_external, template_args |
| doc | title(或首行) | section_path + " " + 正文 | 空 | level, section_path |
| file | file_path | summary | 空 | file_type(code/doc) |

### FTS5
```sql
CREATE VIRTUAL TABLE fts5_all USING fts5(name, content);
-- 填充：
INSERT INTO fts5_all(rowid, name, content) SELECT id, name, content FROM nodes WHERE repo=?;
-- 搜索：
SELECT fts5_all.rowid, bm25(fts5_all) as score, n.name, n.kind, n.file_path, ...
FROM fts5_all JOIN nodes n ON n.id = fts5_all.rowid
WHERE fts5_all MATCH ? ORDER BY score LIMIT ?;
```

### edges 文档层级
```sql
-- 替代 doc_nodes.parent_id
INSERT INTO edges (source_id, target_id, edge_type, source_repo)
SELECT parent.id, child.id, 'contains', parent.repo
FROM nodes parent JOIN nodes child ON child.node_type='doc' 
WHERE ...parent.id was the old parent_id...;
```

## Decisions

### Decision: id 即全局唯一键
选择 INTEGER AUTOINCREMENT 而非 UUID 文本列，因为：
- 整数主键是 SQLite 原生最优键，JOIN 快
- 三表合并后 id 天然不重叠
- 删除了 sid 冗余列（edges 用整数 id，不用 sid 文本）

### Decision: attrs JSON 列
类型特有字段（signature、namespace、level 等）存入 attrs JSON，kind 单独列为索引列。因为 kind 是最常见的 WHERE 过滤条件。

### Decision: 删除 parent_id，用 contains 边
文档层级关系是可查询的结构化关系，用 edges 统一管理，与代码的调用关系一致。

### Decision: 归一化公式修正
`1/(0.5+|score|)` → `2/(1+|score|/8.0)`，压缩 BM25 范围，减少短名惩罚。

## Migration Plan
1. v0.9 migration 函数
2. CREATE nodes 表
3. INSERT INTO nodes SELECT ... FROM symbols (node_type='sym')
4. INSERT INTO nodes SELECT ... FROM doc_nodes (node_type='doc')
5. INSERT INTO nodes SELECT ... FROM files (node_type='file')
6. 迁移 doc parent_id → edges contains
7. DROP symbols, doc_nodes, files
8. 重建 FTS5（已自动从 nodes 填充）
9. 更新 branches.symbol_id → branches.node_id

## Risks
| 风险 | 缓解 |
|------|------|
| 迁移失败数据丢失 | 迁移用事务，失败回滚 |
| 改动面大引入新 bug | 每步 compile + test，增量验证 |
| branches/edges FK 断裂 | v9 migration 处理重命名 |
| 性能下降 (JSON) | kind 单独列索引，常用字段不在 JSON |
