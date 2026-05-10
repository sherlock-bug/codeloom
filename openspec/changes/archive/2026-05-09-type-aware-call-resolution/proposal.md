# Proposal: 类型感知调用解析 + 内置符号 + call_graph 模块化

## Why

当前调用解析是纯名称匹配——`extract_calls()` 从 `call_expression` 提取函数名字符串，`resolve_target()` 用 LIKE 模糊匹配。`obj->method()` 被解析为 `calls:method`，与任何自由函数 `method()` 无法区分。C++ 项目中大量成员调用（`vec.push_back()`、`db->Put()`）要么解析错误，要么 unresolved（to=0），调用图不可用。

## What Changes

1. **成员调用类型感知解析** — `obj->method()` 时检测 `field_expression`，扫描声明建立变量→类型映射，生成 `calls:ClassName::method` 而非 `calls:method`
2. **std 内置符号** — 预置 ~70 个 C++ 标准库符号（容器方法 + 算法函数），不参与 FTS5/向量索引，仅作为调用图 target 节点
3. **call_graph.rs 模块化** — 将 `mcp/mod.rs` 中的 `get_call_graph` / `traverse_calls` 搬至 `src/query/call_graph.rs`，变为可复用模块
4. **this 指针处理** — `this->method()` 用现有 `parent_class` 直接限定

## Capabilities

### New Capabilities

- `type-aware-call-resolution`: 成员调用通过局部变量类型声明解析到正确的类方法，而非 fallback 到同名自由函数
- `builtin-symbols`: 预置 C++ 标准库符号节点（std::vector::push_back 等），repo="__builtin__"，不参与搜索和向量化
- `call-graph-module`: 调用图遍历逻辑从 mcp/mod.rs 提取到 src/query/call_graph.rs，API 清晰可复用

### Modified Capabilities

- 无 — 这是新增能力，不修改现有 spec 级行为。MCP 工具 `codeloom_get_call_graph` 接口不变，内部调用 call_graph 模块。

## Impact

- **代码改动**：`src/indexer/queries/cpp.rs`（scan_local_declarations + 改造 extract_calls）、`src/query/call_graph.rs`（从 stub 变真模块）、`src/mcp/mod.rs`（改为调 call_graph 模块）、`src/storage/symbols.rs`（insert_builtin_symbols）
- **Schema**：symbols 表插入 __builtin__ repo 的合成符号，不新增列/表
- **向后兼容**：MCP/CLI 接口不变，搜索行为不变（__builtin__ 不过 FTS5/向量索引）
