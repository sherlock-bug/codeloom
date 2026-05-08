# macro-parsing

## Purpose
CodeLoom C++ 预处理器宏解析功能域：提取有语义的宏定义为可搜索符号，过滤 include guard 和编译器内置宏等噪音，递归遍历条件编译块确保不遗漏内部宏定义。

## Requirements

### Requirement: 宏符号提取
系统 SHALL 在索引 C++ 代码时提取预处理器宏定义（`#define`、带参宏）为 `kind: "macro"` 的符号节点。

#### Scenario: 提取普通宏
- GIVEN C++ 源文件包含 `#define MY_MACRO 42`
- WHEN 执行 `codeloom index`
- THEN `codeloom search "MY_MACRO"` 返回一个类型为 `macro` 的符号

#### Scenario: 提取函数式宏
- GIVEN C++ 源文件包含 `#define MY_FUNC(x) do_something(x)`
- WHEN 执行索引
- THEN `MY_FUNC` 以 `macro` 类型出现在搜索结果中

### Requirement: 噪音宏过滤
系统 SHALL 过滤掉无搜索价值的噪音宏定义，包括 include guard（如 `MY_HEADER_H`）、编译器内置宏（`__GNUC__` 等双下划线前缀）、平台宏（`_WIN32`）、调试宏（`NDEBUG`）、以及 export/dll 宏（`LEVELDB_EXPORT`）。

#### Scenario: include guard 被过滤
- GIVEN 头文件包含 `#define MY_HEADER_H`
- WHEN 执行索引
- THEN `MY_HEADER_H` 不出现在搜索结果中

#### Scenario: 平台宏被过滤
- GIVEN 源文件包含 `#ifdef _WIN32` 对应的 `#define _WIN32`（假设存在）
- WHEN 执行索引
- THEN `_WIN32` 不出现在搜索结果中

### Requirement: 条件编译块遍历
系统 SHALL 递归遍历条件编译节点（`preproc_if`、`preproc_ifdef`、`preproc_else`）的子节点，确保 `#ifdef`/`#ifndef` 块内的宏定义不被遗漏。

#### Scenario: ifdef 块内宏被提取
- GIVEN 头文件中 `#ifdef USE_FEATURE` 块内包含 `#define FEATURE_FLAG 1`
- WHEN 执行索引且 `FEATURE_FLAG` 不是噪音宏
- THEN `FEATURE_FLAG` 作为 macro 符号出现
