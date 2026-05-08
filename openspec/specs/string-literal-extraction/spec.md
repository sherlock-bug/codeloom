# string-literal-extraction

## Purpose
CodeLoom 字符串字面量提取功能域：将 C++ 代码中所有字符串字面量索引为可搜索符号，支持普通字符串、原始字符串、拼接字符串和系统库字符串四种类型，收集前置注释，记录文件路径和行号。

## Requirements

### Requirement: 字符串字面量符号提取
系统 SHALL 在索引 C++ 代码时将所有字符串字面量（`string_literal`、`raw_string_literal`、`concatenated_string`、`system_lib_string`）提取为 `kind: "string_literal"` 的符号节点，符号名称 SHALL 为去掉引号后的字符串内容。

#### Scenario: 普通字符串被提取
- GIVEN C++ 源文件包含 `"/api/v1/users"`
- WHEN 执行 `codeloom index`
- THEN `codeloom search "/api/v1"` 返回一个类型为 `string_literal` 的符号，其 `name` 为 `/api/v1/users`

#### Scenario: 原始字符串被提取
- GIVEN C++ 源文件包含 `R"(SELECT * FROM users)"`
- WHEN 执行索引
- THEN 该字符串以 `string_literal` 类型出现，名称为 `SELECT * FROM users`

#### Scenario: 系统库字符串被提取
- GIVEN 头文件包含 `#include <iostream>`
- WHEN 执行索引
- THEN `iostream` 以 `string_literal` 类型出现在搜索结果中

### Requirement: 字符串注释提取
系统 SHALL 收集字符串字面量节点前的注释（`///`、`//`、`/**` 等），存储于符号的 `doc_comment` 字段中。

#### Scenario: 带注释的字符串
- GIVEN 代码 `// User API endpoint` 下一行为 `"/api/v1/users"`
- WHEN 执行索引
- THEN 该字符串符号的 `doc_comment` 包含 `// User API endpoint`

### Requirement: 字符串位置记录
系统 SHALL 记录每个字符串字面量符号所在的文件路径和起始行号。

#### Scenario: 位置可查
- GIVEN 文件 `src/server.cpp` 第 42 行包含 `"/health"`
- WHEN 执行索引并 inspect 该符号
- THEN 返回 `file_path: "src/server.cpp"` 和 `line_start: 42`
