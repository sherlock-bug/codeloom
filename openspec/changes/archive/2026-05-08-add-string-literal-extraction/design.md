# Design: 字符串字面量提取

## Context

CodeLoom 的 `walk_children` 对所有未显式匹配的节点返回 `{}`，`string_literal` 等字符串节点当前被跳过。这些节点在 tree-sitter-cpp 中有明确的类型定义，提取成本极低。

## Goals / Non-Goals

**Goals:**
- 提取所有 `string_literal`、`raw_string_literal`、`concatenated_string`、`system_lib_string` 节点为符号
- 收集前置注释存入 `doc_comment`
- 记录文件路径和行号

**Non-Goals:**
- 不去重（同一字符串在不同位置出现会产生多个符号，由 dedup 机制处理）
- 不分析字符串语义（是 URL 还是路径、配置键）
- 不建立边（字符串与所在函数/类的关系留待后续 usage 解析）

## Decisions

### Decision 1: 名称 = 字符串内容去引号
`string_literal` 节点的文本是 `"hello"`（含引号），符号名称取 `child_by_field_name("content")` 或手动 strip 引号后的内容 `hello`。

- `"hello"` → name: `hello`
- `R"(raw)"` → name: `raw`
- `<iostream>` → name: `iostream`
- `"a" "b"` (concatenated) → name: `ab`（拼接后）

原因：LLM 搜 `api/v1` 应命中 `"/api/v1/users"`，不应用引号干扰匹配。

### Decision 2: 四种节点类型统一处理
不区分 `string_literal` 和 `system_lib_string`，统一进 `extract_string_literal()`。噪音由搜索排序自然处理——低频字符串（`iostream`、`"ok"`）排名自然靠后。

### Decision 3: 注释收集复用现有 `collect_comments`
与 `extract_enum`/`extract_macro` 一致，调用 `collect_comments(source, node)` 获取前置注释。`body_comments` 对字符串无意义（字符串没有 body），不收集。

### Decision 4: 不建边
字符串字面量不与所在函数/类建边。原因：
- 字符串可在任意位置（宏参数、赋值右值、函数实参、全局常量）
- 确定"属于谁"需要上下文分析（属于调用路径上的哪个符号），当前不做
- 后续 function usage 解析可统一补充

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| 符号数量爆炸（大型项目数十万字符串） | FTS5 + BM25 对百万级符号仍 O(1)；字符串节点不向量化，不增加 embedding 耗时 |
| 噪音字符串（`"ok"`、`": "` 等短通用串）混入搜索结果 | 搜索排序靠 TF-IDF 自然降权；LLM 通常搜索有意义的路径/URL，很少搜 `ok` |
| `concatenated_string` 文本可能跨多行 | tree-sitter 的 `utf8_text` 返回完整文本含换行，按原样存储 |
