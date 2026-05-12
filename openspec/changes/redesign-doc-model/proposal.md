# Proposal: redesign-doc-model

## Intent
当前文档索引模型缺乏边连接，section 节点与 chunk 节点之间、file 与 section 之间全部依赖隐式字段（`attrs.parent_id`、`file_path`），图工具无法感知文档结构。需将隐式关系显式化为 `contains:` 边，实现统一的文档导航能力。

## Scope
In scope:
- 给现有 `file → section`、`section → section(子标题)`、`section → chunk`、`section → image` 建立 `contains:` 边
- 给 chunk 节点新增 `idx` 字段（同一父节点下的顺序索引）
- 归并两套 section 节点类型：统一为 `node_type='section'`，废弃 `node_type='doc'`
- 索引层修改：写入 section/chunk 时同时建立 `contains:` 边
- 迁移脚本：将现有 `attrs.parent_id` 的数据转为 `contains:` 边

Out of scope:
- 不改图片索引逻辑（`doc_images` 表结构不动）
- 不改搜索工具（FTS5 已覆盖 doc 内容）
- 不改 `inspect`、`search` 等工具——那是其他 change 的事

## Approach
索引器写入 section 和 chunk 后，立即 INSERT `contains:` 边（source=父节点，target=子节点）。迁移阶段执行一次 SQL：从 `attrs.parent_id` 提取关系批量写入 edges 表。chunk 的 `idx` 在分块时计数赋值。
