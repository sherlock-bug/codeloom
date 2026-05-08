# CodeLoom CHANGELOG

## 2026-05-08 — add-string-literal-extraction

- **NEW**: C++ 索引器新增字符串字面量提取。所有 `string_literal`、`raw_string_literal`、`concatenated_string`、`system_lib_string` 被索引为 `kind: "string_literal"` 符号，可通过搜索和 MCP 工具查询。
- 字符串名称自动去定界符（`"..."` → 内容，`R"(...)"` → 内容，`<...>` → 内容）。
- 前置注释自动收集入 `doc_comment`。
- 改动文件：`src/indexer/queries/cpp.rs`（+55 行）
- 测试：75/75 全过

## 2026-05-08 — add-macro-parsing

- **NEW**: C++ 索引器新增宏符号提取。`#define` 和带参宏被索引为 `kind: "macro"` 符号，可通过搜索和 MCP 工具查询。
- **NEW**: 噪音宏过滤。include guard（`*_H`、`*INCLUDED*`）、编译器内置宏（`__*`）、平台宏（`_WIN32` 等）自动跳过，不入索引。
- **NEW**: 条件编译块遍历。`#ifdef`/`#ifndef`/`#else` 块内的宏定义不再被遗漏。
- 改动文件：`src/indexer/queries/cpp.rs`（+35 行，3 个新增函数）
- 测试：75/75 全过
