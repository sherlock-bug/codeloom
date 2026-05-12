# Design: fix-include-edges

## 现状

`cli/mod.rs` 中 `index_includes()` 遍历 C/C++ 源文件，正则提取 `#include` 指令后直接写 `(0, 0, "includes:foo.h", repo)` 到 edges 表。此时 file nodes 已经入库（`index_docs` 先于 `index_includes` 执行），但函数未尝试查表获取真实 ID。

UNIQUE 索引 `idx_ed_unique(source_id, edge_type, branch_id)` 在 `source_id=0` 时导致所有 include 边共享同一 source_id——`INSERT OR IGNORE` 只保留第一条，其余静默丢弃。

## 修改方案

### `index_includes()` 重写

1. **签名变更**：增加 `branch_id: i64`，返回类型从 `usize` 改为 `anyhow::Result<usize>`
2. **source_id 解析**：对每个源文件，`strip_prefix(dir)` 得相对路径，查 `nodes` 表获取 file node ID
3. **target_id 解析**：对 `#include "foo.h"`，先同目录查，再从根查；系统路径 `<...>` 返回 `None`
4. **INSERT**：写入真实 source_id、target_id（或 0）、edge_type、repo、branch_id

### UNIQUE 索引修正

```sql
DROP INDEX IF EXISTS idx_ed_unique;
CREATE UNIQUE INDEX idx_ed_unique ON edges(source_id, target_id, edge_type, branch_id);
```

### 辅助函数

- `resolve_source_id()`: 路径→file node ID
- `resolve_include_target()`: include 路径→file node ID（两段式查找）
- `normalize_path()`: 处理 `../` 和 `./`

## 调用处修改

`cli/mod.rs` 中调用 `index_includes` 的位置需传入 `branch_id`。当前索引流程中已有 `branch_id` 变量可用。
