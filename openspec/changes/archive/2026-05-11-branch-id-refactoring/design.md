# Design: branch-id-refactoring

## Context

当前分支存储四分五裂：`nodes.branch_name` 存空串/`'main'`，`edges.branch_name` 全 NULL，`branches.branch_name` 存 `'master'`。三者互不关联。且 `edges` 唯一索引缺失分支维度，多分支索引时边数据互相覆盖。

无历史数据顾虑——直接改 schema，`codeloom clean --all` 后重跑索引。

## Goals / Non-Goals

**Goals:**
- 引入 `branch_meta` 表作为分支 ID 唯一来源
- 所有分支引用改为 `branch_id INTEGER`
- `edges` UNIQUE 索引包含分支维度
- 整库 `clean --all` 后重索引无问题

**Non-Goals:**
- 不改 `git_index_state` 表（仍用字符串 branch_name，与 git 逻辑对应）
- 不改 `branch_glossary` 表（别名仍用字符串）

## Decisions

### Decision: branch_meta 表结构

```sql
CREATE TABLE branch_meta (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    repo TEXT NOT NULL,
    branch_name TEXT NOT NULL DEFAULT 'main',
    UNIQUE(repo, branch_name)
);
```

`(repo, branch_name)` 唯一，确保同一仓库同分支名只一个 ID。`branch_name` 默认 `'main'` 与当前代码行为一致。

### Decision: nodes.branch_id 语义

`nodes.branch_id INTEGER NOT NULL DEFAULT 0`。`0` 表示「未指定分支」，兼容遗留数据和测试场景。查询时 `WHERE branch_id != 0` 或 `= ?`。

### Decision: edges 唯一索引

旧索引 `(source_id, edge_type, source_repo)` → 新索引 `(source_id, edge_type, branch_id)`。去掉 `source_repo`（通过 source_id JOIN nodes 可推 repo），改为分支维度。
同时 `branch_name TEXT` → `branch_id INTEGER NOT NULL DEFAULT 0`。

### Decision: branches 表

`branches(node_id, repo, branch_name)` → `branches(node_id, repo, branch_id)`。保持 PK `(node_id, repo, branch_id)`。

### Decision: 代码层分支解析

所有 INSERT 节点/边的代码，在获取到 `repo` 和 `branch_name` 后，先执行：

```sql
INSERT OR IGNORE INTO branch_meta (repo, branch_name) VALUES (?, ?);
SELECT id FROM branch_meta WHERE repo=? AND branch_name=?;
```

得到 `branch_id` 后再写入节点/边。模式统一，代码可提取为工具函数。

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|---------|
| `branch_id=0` 默认值与实际分支混淆 | 查询强制 `branch_id != 0` 过滤；仅测试/兼容场景用 0 |
| 多 JOIN 一次 branch_meta 查询性能 | branch_meta 表极小（每仓库 ~10 行），无需索引也够快 |
| `git_index_state` 不同步改 ID | 保持字符串，它不参与搜索过滤，只存状态 |
