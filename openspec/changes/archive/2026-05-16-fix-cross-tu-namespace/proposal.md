# Proposal: fix-cross-tu-namespace

## Intent
修复 CodeLoom 索引引擎的两个缺陷：(1) 节点名称不包含 namespace（如 `Handler::handle` 应为 `cross_tu::Handler::handle`），导致跨 TU 调用边的 target_name 与 DB 节点名不匹配；(2) 跨 TU 调用边缺失——当函数在另一 TU 中定义时，`name_to_id` 找不到目标节点，需 fallback 到 DB 查询。

## Scope
In scope:
- 节点命名：函数/方法的 `sym.name` 包含完整 namespace 前缀（如 `cross_tu::Handler::handle`）
- 跨 TU 调用边：当 `name_to_id` 查找失败时，fallback 到 SQLite DB 查询（精确匹配 + 前缀剥离 LIKE 兜底）
- line_end schema 迁移：消除 `attrs` JSON 双写，`line_end` 列是唯一来源
- 验证：断言仓 194/194 全绿（含 G5d line_end 修复 + G17 namespace 修复）

Out of scope:
- namespace 字段存储格式变更（保留 `sym.namespace`）
- 非 C++ 语言（Python/Go 等）的跨 TU 边问题
- 抽象接口指针的跨 TU 调用边缺失（G18 — 已知限制，独立 fix）

## Approach
1. **节点命名**：修改 `ast.rs` 中 `qname` 的构建逻辑，将 namespace 前缀加入 `sym.name`。对 mangled name 解析，取所有 namespace+class 组件拼接；对 inline 定义，从 `parent_class` 中提取。
2. **跨 TU 边匹配**：在 `clang/mod.rs` 中实现 `find_cross_tu_target`——当 `name_to_id.get(&target_name)` 为 `None` 时，先用精确名称查 DB，再逐层剥离 namespace 前缀用 LIKE 兜底。
3. **line_end schema 迁移**：`line_end` 从 `attrs` JSON 双写迁移到列唯一存储，更新 INSERT/SELECT/merge 路径和所有阅读端（MCP、CLI）。
4. **断言更新**：G17 全部改为硬断言，FQN 含 namespace；G5d line_end 修复。
5. **验证**：先跑断言仓（194/194 全绿），再索引 leveldb 确认效果。

## Capabilities

### New Capabilities
- `fqn-node-naming`: 函数/方法节点名包含完整 namespace 前缀，与 extractor 的 target_name 一致
- `cross-tu-edge-fallback`: 跨 TU 调用边在 `name_to_id` 查找失败时 fallback 到 DB 查询，适配 FQN 命名
- `line-end-schema-cleanup`: 消除 `line_end` 列与 `attrs` JSON 的双写不一致，列是唯一可靠来源
