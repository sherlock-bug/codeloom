# Proposal: 宏解析

## Why

CodeLoom 的 C++ 索引器完全跳过预处理器宏（`preproc_def`、`preproc_function_def`、`preproc_call`），导致以下问题：
1. 项目中的有语义宏（如 `LEVELDB_EXPORT`、配置开关宏）无法被搜索
2. 注册宏调用（如 `REGISTER_COMMAND(MyClass)`）不产生任何边，LLM 无法知道 MyClass 被框架注册为入口点
3. 条件编译块（`#ifdef`、`#ifndef`）内的代码完全不被遍历，遗漏内部符号

本次变更在不引入 clang 的前提下，用 tree-sitter 的 `preproc_*` 节点提取有语义的宏符号和注册关系。

## What Changes

- **NEW**: `walk_children` 增加 `preproc_def`、`preproc_function_def`、`preproc_if`/`ifdef`/`else` 三个 match arm，不再彻底跳过预处理器节点
- **NEW**: `extract_macro()` — 从 `preproc_def`/`preproc_function_def` 节点创建 `kind: "macro"` 的符号
- **NEW**: `is_noise_macro()` — 过滤 include guard、平台宏、编译器内置宏、export 宏等无搜索价值的宏
- **NEW**: 条件编译块递归遍历（`preproc_if`/`ifdef`/`else` 直接遍历子节点），确保 `#ifdef` 内宏定义不被遗漏

## Capabilities

### New Capabilities
- `macro-parsing`: C++ 预处理器宏的符号提取、噪音过滤、注册宏 entrypoint 边

### Modified Capabilities
无 — 本次为纯新增功能，不修改已有规范的行为。

## Impact

- 受影响文件：`src/indexer/queries/cpp.rs`（+55 行）
- 数据库不变：macro 符号写入已有 `symbols` 表，type 字段为 `macro`
- 测试：75/75 全过，新增代码被既有测试覆盖（walk_children 结构不变）
- 搜索：`codeloom search "MY_MACRO"` 可返回 macro 类型结果
