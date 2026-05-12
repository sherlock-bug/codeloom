# Proposal: fix-include-edges

## Intent

修复 BUG-04：`index_includes()` 将 `#include` 边写入 edges 表时 `source_id=0, target_id=0`，无法关联到任何符号。同时 UNIQUE 索引 `(source_id, edge_type, branch_id)` 在 source_id=0 时导致所有 include 边互相冲突，实际只保留了第一条。

## Scope

In scope:
- `index_includes()` 从硬编码 0,0 改为查询 file node ID 作为 source_id/target_id
- 函数签名增加 `branch_id` 参数
- edges 表 UNIQUE 索引重建，加入 `target_id` 列
- known-limitations 更新

Out of scope:
- 系统头文件（`<vector>` 等）的 target_id 解析（保持 0）
- 调用图/路径分析工具的联动修改
- 性能优化（prepared statement 已做，HashMap 缓存不做）
