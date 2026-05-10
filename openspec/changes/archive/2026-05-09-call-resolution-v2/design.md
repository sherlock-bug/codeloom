# Design: 调用解析 v2

## Context

v1 实现了 `extract_calls_with_types()` 三层解析：field_expression 显式成员调用、this 指针、隐式 this 裸调用。已有基础设施：

- `extract_type_edges()` 生成 `returns:TypeName` 边
- `extract_type_edges()` 生成 `overrides:methodName` 边
- `extract_class_impl()` 生成 `inherits:BaseClass` 边
- `call_expression` 的 CST 有 `arguments` 子节点 → `argument_list`

## Goals / Non-Goals

**Goals:**
- `auto x = getFoo()` 查 `getFoo` 的 `returns` 边拿类型
- `base->virtualMethod()` 检测 override 并生成派生类调用边
- 同名重载按参数个数附加 `(N)` 后缀
- 改动集中在 `cpp.rs`，不新增模块

**Non-Goals:**
- 不做完整类型推断（隐式转换、模板推导）
- 不跨函数追踪嵌套 `auto`（`auto x = getBar(); auto y = x->foo()`）
- 不做参数类型精确匹配（只做计数）

## Decisions

### Decision 1: auto 通过 returns 边追溯
`try_resolve_auto()` 新增分支：当右侧为 `call_expression` 时，提取被调函数名，在 `symbols` 中查找 `returns` 边。找到则用返回类型，否则 fallback `"auto"`。

- 方案：查同文件 `symbols` 中的 `returns` 边（同文件优先），查不到则查全局 DB
- 限制：只追溯一层（`auto x = getFoo()`），不做嵌套 auto（`auto y = x->bar()`）
- 前提：`getFoo()` 必须在当前文件或已索引

### Decision 2: 虚函数展开在 extract_calls_with_types 做
当 `obj->method()` 解析到 `BaseClass::method` 后，额外检查该方法是否有 `overrides` 边。如果有，生成额外调用边指向 override 版本。

- 生成 `calls:BaseClass::method`（主边）+ `calls:Derived::method`（override 边）
- 用不同 edge_type 标签区分：`calls_override:Derived::method`
- 解析时需查 symbols 表确认 override 存在
- `call_graph.rs` 的 `traverse_calls` 同时匹配 `calls:%` 和 `calls_override:%`

### Decision 3: 重载消歧用参数计数后缀
`call_expression` 的 `arguments` 字段指向 `argument_list`，计数字节点数即可。

- 边标签：`calls:func(2)` 表示 2 参数版本
- `resolve_target` 的匹配逻辑：先精确匹配 `func(2)`，再 LIKE fallback `%::func(2)`
- 自由函数和成员调用都适用
- Template 参数计为 1（简单计数）

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| `auto x = getFoo()` 而 `getFoo` 未索引 | fallback `"auto"`，不降级 |
| 虚函数展开产生大量边 | 仅 v1 已解析的成员调用触发，量可控 |
| 参数计数对默认参数不准确 | 不计默认参数（CST 只含实参），实际足够 |
| `returns` 边可能匹配错误函数（同名） | 同文件 symbols 优先，再查 DB |
