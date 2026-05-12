# Tasks

## 1. graph.rs: get_edges() 加 branch_id 参数

- [x] 1.1 `get_edges()` 新增 `branch_id: i64` 参数，forward/reverse SQL 均加 `AND (e.branch_id = ? OR e.branch_id = 0)`
- [x] 1.2 更新调用方 `neighbor_map()` 和递归调用点

## 2. graph.rs: build_forward_adj / build_reverse_adj 加 branch_id 参数

- [x] 2.1 `build_forward_adj()` 新增 `branch_id: i64` 参数，SQL 加 `AND (branch_id = ? OR branch_id = 0)`
- [x] 2.2 `build_reverse_adj()` 同样加 `branch_id`
- [x] 2.3 更新所有调用方：`bfs_path_search()`、`transitive_closure()` 透传 branch_id

## 3. graph.rs: get_terminal_deps 加 branch_id 过滤

- [ ] 3.1 跳过——get_terminal_deps 未被任何代码调用

## 4. call_graph.rs: traverse_calls() 加 branch_id 过滤

- [x] 4.1 `get_call_graph()` 已有 `branch_id` resolve → 传入 `traverse_calls`
- [x] 4.2 `traverse_calls()` 改为 `branch_id: i64` 参数，caller/callee SQL 均加 `AND e.branch_id = ?`

## 5. mcp/mod.rs: 各工具调用的边查询加 branch_id 过滤

- [x] 5.1 `inspect_symbol()` 的 edge SQL 加 `AND e.branch_id = ?`（format! {1}）
- [x] 5.2 `inheritance_tree()` 的 build_tree/get_overrides SQL 加 `AND e.branch_id = ?`
- [x] 5.3 `path_analysis()` → 传 branch_id 给 bfs_path_search
- [x] 5.4 `impact_analysis()` → 传 branch_id 给 transitive_closure
- [x] 5.5 `neighbor_graph()` → 传 branch_id 给 neighbor_map

## 6. 编译验证

- [x] 6.1 `cargo check` 通过
- [x] 6.2 `cargo test` 全部通过（64 passed, 12 ignored）
- [x] 6.3 `cargo build --release` 通过

## 7. 修复 inheritance_tree 硬编码 branch_id ⚡ 审查发现后补

- [x] 7.1 `build_tree` 嵌套函数加 `branch_id: i64` 参数
- [x] 7.2 `get_overrides` 嵌套函数加 `branch_id: i64` 参数
- [x] 7.3 替换所有硬编码 `branch_id_0 = 0` 为实际 branch_id（up/down/override 共4处）
- [x] 7.4 递归调用透传 branch_id
- [x] 7.5 `cargo check` + `cargo test` 验证
