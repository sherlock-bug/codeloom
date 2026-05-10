# Design: 类型感知调用解析 + 内置符号 + call_graph 模块化

## Context

当前调用边提取在 `src/indexer/queries/cpp.rs:extract_calls()` 中，纯名称匹配：

```
call_expression → function 子节点 → utf8_text → "method" → edges("calls:method", target=usize::MAX)
```

C++ 中大量调用是成员调用（`obj->method()`），`function` 子节点是 `field_expression`，包含对象信息和成员名。现有代码丢弃了类型信息。

调用图遍历代码位于 `src/mcp/mod.rs:get_call_graph()` / `traverse_calls()`，约 70 行。`src/query/call_graph.rs` 是 stub。

## Goals / Non-Goals

**Goals:**
- `obj->method()` 解析为 `calls:MyClass::method`，限定到类
- `this->method()` 用现有 `parent_class` 限定
- 函数参数 `void f(MyClass* p) { p->method(); }` 在定义处解析
- ~70 个 std 内置符号作为合成 target 节点
- `call_graph.rs` 从 stub 变可复用模块

**Non-Goals:**
- 不跨函数追踪类型（`auto x = getFoo(); x->method()`）
- 不解析虚函数展开
- 不处理重载消歧（参数类型区分）
- 内置符号不参与 FTS5/向量搜索（零性能影响）

## Decisions

### Decision 1: 局部声明扫描 + HashMap

选择在 `extract_calls_with_types()` 中先扫描函数体建立 `HashMap<String, String>`（变量名→类型名），而非做全量 AST 类型推断。

**方案对比：**
- 局部 HashMap：O(N) 单遍扫描，处理 80% 用例，代码 ~50 行
- 全量类型推断：需要 SSA/CFG 分析，复杂度过高，树栖解析器不支持
- 选局部 HashMap 是因为：够用、简单、零依赖

### Decision 2: 声明扫描覆盖的节点类型

```
declaration: TypeName* var;        → type 字段 + init_declarator
declaration: TypeName var;         → 同上
parameter_declaration: TypeName p  → 函数参数，函数签名中遍历
field_expression: obj->method()    → 查表，查不到用 parent_class（this 指针场景）
```

`auto x = new MyClass()` → 简单 case 可解析 `new` 表达式类型。`auto x = func()` → 不追跨函数。

### Decision 3: 内置符号存储策略

```
repo = "__builtin__"
content_hash = "builtin:" + 序号  (保证唯一，不与真实代码哈希冲突)
language = "cpp"
kind = "method" / "function"
branch: 不插入（NULL，所有分支可见）
```

约 70 个符号，插入时机：`smart_index` 首次索引前 `insert_builtin_symbols()`。

**不参与搜索的机制：**
- FTS5 填充：`fill_symbols_fts()` 用 `WHERE s.repo=?1`，`__builtin__` 天然过滤
- 向量索引：`index_vectors()` 同样 `WHERE s.repo=?1` 过滤
- 搜索结果：`hybrid_search()` 的 FTS5/向量查询都带 repo 过滤

### Decision 4: 边标签格式

成员调用边标签从 `calls:method` 改为 `calls:ClassName::method`：

```
// 成员调用
obj->push_back() → calls:std::vector::push_back

// 自由函数调用（不变）
free_function()  → calls:free_function

// this 指针
this->DoWork()   → calls:MyClass::DoWork  (用 parent_class)
```

`resolve_target()` 已有 `%::name` 后缀匹配，`ClassName::method` 会匹配到 `MyClass::method` 符号。

### Decision 5: call_graph.rs 模块 API

```rust
pub fn get_call_graph(
    conn: &Connection,
    name: &str,
    repo: &str,
    branch: &str,
    direction: &str,  // "callers" | "callees"
    max_depth: usize,
) -> String;

fn traverse_calls(
    conn, sym_id, direction, max_depth, depth, visited, out
) -> ();
```

MCP 层的 `get_call_graph()` 改为调用此模块，保持 JSON-RPC 格式不变。

## Project Directory Structure

```
src/
  query/
    call_graph.rs     — 从 stub 变真模块（+~80 行）
  indexer/
    queries/
      cpp.rs         — 新增 scan_local_declarations() + 改造 extract_calls()（+~80 行）
  mcp/
    mod.rs           — get_call_graph → 调用 query::call_graph 模块（-70 行，+3 行）
  storage/
    symbols.rs       — 新增 insert_builtin_symbols()（+~100 行）
```

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|---------|
| 局部声明扫描遗漏部分类型（宏包裹的类型名、typedef） | 未查到的变量 fallback 到纯名称，不降级现有行为 |
| 内置符号 content_hash 与真实代码冲突 | 用 "builtin:N" 前缀，SHA256 不可能碰撞 |
| `__builtin__` repo 被用户误搜索 | 搜索按真实 repo 名过滤，不会混入 |
| 多文件间的类型引用无法跨文件追踪 | v1 明确 Non-Goal，后续迭代 |
