# Proposal: 调用解析 v2 — auto 跨函数 + 虚函数展开 + 重载消歧

## Why

v1 的类型感知调用解析覆盖了显式 `obj->method()` 和 `this->method()` 以及隐式 this 调用。但三个常见 C++ 场景尚未覆盖：

1. **`auto x = getFoo(); x->method()`** — `auto` 变量的类型无法从右侧表达式推断（非 `new` 表达式），当前降级为 `auto`，调用回落为纯名称匹配
2. **`base->virtualMethod()`** — 虚函数调用只解析到基类的声明，不展开派生类 override，调用图不完整
3. **`func(1)` vs `func("s")`** — 同名重载函数无法按参数区分，总是解析到第一个匹配

## What Changes

1. **auto 跨函数类型追踪** — `try_resolve_auto()` 扩展：查被调函数的 `returns` 边，拿返回类型
2. **虚函数展开** — `extract_calls_with_types()` 检测到基类方法有 `overrides` 边时，额外生成指向所有 override 的调用边
3. **重载参数计数消歧** — 解析 `call_expression` 的 `arguments` 子节点，按参数个数生成 `calls:func(2)` 限定边标签

## Capabilities

### New Capabilities

- `auto-cross-function`: `auto x = getFoo()` 通过符号表的 `returns` 边推断类型
- `virtual-dispatch`: `base->method()` 当基类方法被 override 时展开所有派生类方法
- `overload-disambiguation`: 同名重载按参数个数区分，边标签带 `(N)` 后缀

## Impact

- **代码改动**：`cpp.rs`（try_resolve_auto 扩展、virtual dispatch 逻辑、arguments 解析）
- **call_graph.rs**：虚函数展开的遍历逻辑
- **向后兼容**：MCP/CLI 接口不变，output 格式增量（override 条目显示）
