# Proposal: 修复 Clang 解析器行号为零和头文件路径匹配 bug

## Why

Clang 索引器存在两个 bug 导致符号数据质量严重退化：

1. **行号全为零** — `get_range()` 从 `range.begin.line` 读行号，但 Clang 18 AST JSON 的 `range.begin` 从来不输出 `line` 字段（只有 offset/col/tokLen），导致所有符号 `line_start=0`。影响了搜索结果的跳转准确性和可用性。

2. **头文件符号被错误标记为外部** — Python filter 子节点路径传播用的是相对路径（如 `"db/version_set.h"`），Rust 侧 `is_project_file()` 直接用相对路径 `startswith(project_root)` 检查，永远不匹配，导致所有头文件符号 `is_external=true`，调用关系、参数类型等边全部丢失。且符号的 `file_path` 存的是相对路径，消费侧无法直接使用。

## What Changes

- **`ast.rs` — `get_range()` 改用 offset→line 换算**：Clang AST JSON 的 `range.begin`/`range.end` 有可靠的 `offset`（文件级别字节偏移），但无 `line`。改为读取源文件建换行符偏移表，`binary_search` 将 offset 换算为行号
- **`ast.rs` — `get_loc()` 不做变更**：`loc.line` 作为 `get_range` 的 fallback（用于有 loc 但 range 不存在的节点）
- **`ast.rs` — `is_project_file()` 支持相对路径**：在 `startswith(project_root)` 前，将相对路径拼成绝对路径再比较
- **`ast.rs` — Python filter 子节点路径传播改为绝对路径**：避免后续依赖此路径的消费方再次遇到相对/绝对混用问题
- **Python filter `clang_filter.py` — 子节点 `loc.file` 传播使用绝对路径而非原始相对路径**

- **新增集成测试**：针对两个 bug 的端到端测试
  - 测试 1：索引一个手写 C++ fixture（含 class + method + 头文件 include），验证符号 `line_start`/`line_end` 非零
  - 测试 2：验证头文件中的类符号 `is_external=false`，且边（如 param_type）正确提取

## Capabilities

### New Capabilities
- `fix-clang-line-and-path`: 修复 Clang 索引器行号为零和头文件路径匹配问题，同时添加端到端集成测试确保质量门禁

### Modified Capabilities
- `clang-subprocess-parser`: 修改 Requirement「从 AST 提取函数声明和定义」和「从 AST 提取类/结构体声明」，补充行号正确性和外部/内部判定场景

## Impact

- `src/indexer/clang/ast.rs` — `get_range()`、`is_project_file()`、`get_node_file()` 相关逻辑
- `~/.codeloom/scripts/clang_filter.py` — 子节点路径传播改为绝对路径
- `tests/` — 新增 `fixtures/line_and_path/` 测试素材 + 集成测试
- 无 schema 变更，无 new dependency
- 之前索引的 DB 行号仍为 0，需要 `codeloom clean && codeloom index` 重新索引
