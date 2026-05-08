# Proposal: 字符串字面量提取

## Why

CodeLoom 完全跳过 C++ 源码中的字符串字面量（`"hello"`、`"/api/v1/users"`、`R"(raw)"` 等），导致 URL 路径、配置键、注册标识符等嵌入字符串中的关键信息无法被搜索。

LLM Agent 面对陌生代码库时，搜 `/api/v1` 找不到任何结果——尽管源码中 `REGISTER_URL(UserController, "/api/v1/users", "POST")` 就在眼前。字符串字面量是理解代码库的重要信息载体，应当进入搜索体系。

## What Changes

- **NEW**: `walk_children` 增加 `string_literal`、`raw_string_literal`、`concatenated_string`、`system_lib_string` 四种节点类型的 match arm，调用 `extract_string_literal()`
- **NEW**: `extract_string_literal()` — 创建 `kind: "string_literal"` 的符号节点，名称即为字符串内容（去引号），记录所在文件/行号，收集前置注释

## Capabilities

### New Capabilities
- `string-literal-extraction`: 提取所有 C++ 字符串字面量为可搜索符号

### Modified Capabilities
无。

## Impact

- 受影响文件：`src/indexer/queries/cpp.rs`（新增 ~15 行）
- 数据库不变：string_literal 写入已有 `symbols` 表
- 符号数量增加：取决于项目规模，字符串字面量通常远超函数/类数量
- 搜索：`codeloom search "/api/v1"` 可返回命中
