# Proposal: fix-header-file-path

## Intent

Clang `-ast-dump=json` 输出中，`#include` 进来的头文件中声明的符号缺失 `loc.file` 字段（只给 `line` 和 `col`），导致 `file_path` 回退到翻译单元文件（.cpp），搜索结果跳转到错误文件。

## Scope

In scope:
- `src/indexer/clang/ast.rs` 中追踪 AST 遍历时的 `cur_file` 上下文
- 头文件符号的 `file_path` 正确指向 `.h` 文件而非 `.cpp` 文件
- 外部符号判定（is_external）使用正确的文件上下文
- 同步更新 `known-limitations` spec 标记为已修复

Out of scope:
- `clang_filter.py` 修改
- Clang 命令行参数变更
- 其他已知限制项目的修复

## Approach

Clang 的 `-ast-dump=json` 在进入新文件（`#include`）时 emit 一次 `loc.file`，但子节点不重复携带。因此需要在 `ExtractCtx` 中添加 `cur_file: String` 字段，遍历 AST 时遇到带 `loc.file` 的节点更新当前文件，子节点继承上下文。所有 `file_path` 回退逻辑从 `self.file`（TU 文件）改为 `self.cur_file`（当前文件上下文）。

## Capabilities

- `header-file-path-resolution`: 头文件符号的 `file_path` 正确指向实际定义文件而非翻译单元文件
