# Tasks

## 1. 修改 inspect_symbol 按类型分支

- [x] 1.1 `src/mcp/mod.rs`: inspect_symbol() 遍历每个符号节点时，先查询 `node_type` + `kind`，按类型分支
- [x] 1.2 class/struct: 输出 bases + members + methods
- [x] 1.3 enum: 输出 values
- [x] 1.4 file: 输出 sections
- [x] 1.5 section: 输出 parent + children + chunks + siblings
- [x] 1.6 chunk: 输出 parent_section + prev/next chunk
- [x] 1.7 其余类型保持当前通用 edges 输出

## 2. 编译验证

- [x] 2.1 `cargo build` 编译通过
- [x] 2.2 `cargo test` 全部通过（64 passed, 0 failed）

## 3. 更新工具描述

- [x] 3.1 inspect MCP 工具描述同步更新
