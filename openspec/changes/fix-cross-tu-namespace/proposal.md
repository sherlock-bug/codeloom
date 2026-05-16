# Proposal: fix-cross-tu-namespace

## Intent
修复 CodeLoom 索引引擎的两个缺陷：(1) 节点名称不包含 namespace（如 `Handler::handle` 应为 `cross_tu::Handler::handle`），导致跨 TU 调用边的 target_name 与 DB 节点名不匹配；(2) 跨 TU 调用边缺失——当函数在另一 TU 中定义时，`name_to_id` 找不到目标节点，需 fallback 到 DB 查询。

## Scope
In scope:
- 节点命名：函数/方法的 `sym.name` 包含完整 namespace 前缀（如 `cross_tu::Handler::handle`）
- 跨 TU 调用边：当 `name_to_id` 查找失败时，fallback 到 SQLite DB 查询（精确匹配 + 前缀剥离 LIKE 兜底）
- 验证：G17 全部 6 个断言通过（189→195）
- 覆盖 leveldb 真实场景（`Env::GetChildren` 等缺失边补齐）

Out of scope:
- namespace 字段存储格式变更（保留 `sym.namespace`）
- 非 C++ 语言（Python/Go 等）的跨 TU 边问题
- MCP 工具的接口变更

## Approach
1. **节点命名**：修改 `ast.rs` 中 `qname` 的构建逻辑，将 namespace 前缀加入 `sym.name`。对 mangled name 解析，取所有 namespace+class 组件拼接；对 inline 定义，从 `parent_class` 中提取。
2. **跨 TU 边匹配**：恢复 `edge_index.rs` 中的 `find_cross_tu_target` 逻辑——当 `name_to_id.get(&target_name)` 为 `None` 时，先用精确名称查 DB，再逐层剥离 namespace 前缀用 LIKE 兜底。
3. **断言更新**：G17 全部改为硬断言，FQN 含 namespace。
4. **验证**：先跑断言仓（195 条全绿），再索引 leveldb 确认边补齐。

## Capabilities

### New Capabilities
- `fqn-node-naming`: 函数/方法节点名包含完整 namespace 前缀，与 extractor 的 target_name 一致
- `cross-tu-edge-fallback`: 跨 TU 调用边在 `name_to_id` 查找失败时 fallback 到 DB 查询，适配 FQN 命名
