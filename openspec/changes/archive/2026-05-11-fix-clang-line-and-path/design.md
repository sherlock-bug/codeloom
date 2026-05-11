## Context

Clang 索引器当前使用 `clang -ast-dump=json` 子进程 + Python filter 管道来解析 C/C++ 代码。产出的 JSON 进入 Rust 侧 `ast.rs` 的 `extract_symbols_and_edges()` 提取符号和边。

现场发现两个数据质量问题：

1. **行号全为零**：`get_range()` 在 `ast.rs:669-676` 读 `range.begin.line`，但 Clang 18 AST JSON 的 `range.begin` 只有 `offset`/`col`/`tokLen`，**从来没有 `line`**。导致所有符号 `line_start=0`。

2. **头文件符号 `is_external=true`**：Python filter 在 `filter_node()` 中计算了 `abs_node_file`（绝对路径，用于 `startswith(project_root)` 判断），但子节点路径传播用的却是原始的相对 `node_file`（如 `"db/version_set.h"`）。Rust 侧 `is_project_file()` 直接用这个相对路径 `startswith(project_root)` 永远不匹配，所有头文件符号被标记为外部。

## Goals / Non-Goals

**Goals:**
- `line_start`/`line_end` 从 `range.begin.offset` 换算，非零
- 头文件符号（类、结构体等）`is_external=false`
- 头文件符号的调用关系、参数类型等边正常提取
- 新增端到端集成测试验证以上两个修复

**Non-Goals:**
- 不涉及 libclang 切换（那是另一个 change）
- 不修改其他非 C/C++ 解析器
- 不修改 DB schema

## Decisions

### Decision 1: offset→line 换算而非 loc.line

`loc.line` 确实存在并且不为零，但它指向的是**节点名字**的位置，而非**声明起始**的位置：

```cpp
int           ← range.begin.offset → 第 1 行
foo() { }     ← loc.line → 第 2 行（指向函数名 'foo'）
```

使用 offset→line 换算更准确（声明起始）。方法是读源文件建 `Vec<u32>` 换行符偏移表，`binary_search` 查 offset 对应的行号。libclang 的 `clang_getSpellingLocation()` 也是给行号，到时只需重写 `get_range()` 实现即可。

### Decision 2: Python filter + Rust 双防护

路径问题从两个方向修：

- **Python filter**（根源修复）：`filter_node()` 中子节点 `loc.file` 传播改为 `abs_node_file`（绝对路径），从源头确保 JSON 中的路径是绝对的
- **Rust `is_project_file()`**（防御性修复）：在 `startswith(project_root)` 之前，将相对路径拼上 `project_root` 转绝对再检查

这确保即使 Python filter 被绕过或替换，Rust 侧也能正确处理。

### Decision 3: `line_end` 行为

当 `range.end.offset` 存在时换算行号，不存在时回退到 `line_start`。单行声明（字段、简单函数）的 `line_start == line_end`，多行声明（长函数体、类定义）有正确的结束行号。

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|---------|
| offset→line 需读源文件，大文件频繁调用有性能开销 | 用 `LazyLock<HashMap<path, LineIndex>>` 缓存换行符表，每个文件只扫一次 |
| Python filter 改子节点路径传播可能影响其他消费方 | 已验证只有 Rust ast.rs 消费，无其他下游 |
| `is_project_file()` 新增路径归一化可能影响非 Clang 路径（如 tree-sitter 的绝对路径） | 判断当前路径是否绝对，相对才拼接，绝对不变——对已有绝对路径无影响 |
