# CodeLoom CHANGELOG

## 2026-05-08 — add-template-parsing

- **NEW**: C++ 索引器新增模板解析。`template_declaration` 下的类/结构体/函数被索引为 `kind: "template_class"` / `template_struct` / `template_function`。
- **NEW**: 模板参数边。每个模板参数变为 `template_param:NAME[KIND]` 边（KIND=`type`/`non_type`）。
- **NEW**: std 容器关系边。18 种 std 容器自动建类间关系边：`aggregate`（vector/list/set 等）、`owns`（unique_ptr）、`shares`（shared_ptr/weak_ptr）、`map_key`+`map_value`（map/multimap 等）。嵌套模板类型正确解析（`std::vector<Box<std::string>>` → `aggregate:Box<std::string>`）。
- **NEW**: 自定义模板使用检测。字段类型中的 `MyVector<User>` 若 MyVector 是项目内 `template_class`，则建 `template_use:MyVector<User>` 边。
- **NEW**: 双模式符号解析。所有容器/模板边的 `to` 字段：项目内符号用真实 ID，否则 `usize::MAX`（存储为 0）。
- **FIX**: `smart.rs` 的 `resolve_target` 不再无视提取器传的 `usize::MAX`；LIKE 模式从 `%X%` 改为 `%::X` 后缀匹配防误碰；map 的 key/value 分别解析。
- 改动文件：`src/indexer/queries/cpp.rs`（+150 行）、`src/indexer/smart.rs`（修改 resolve_target + edge loop）
- 测试：75/75 全过

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
