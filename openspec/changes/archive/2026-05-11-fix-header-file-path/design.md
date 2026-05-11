# Design: fix-header-file-path

## Context

Clang `-ast-dump=json` 输出的 AST JSON 中，`loc` 对象的 `file` 字段只在文件上下文切换时出现（例如 `#include` 进入一个新的头文件）。对于同一文件内的子节点，`file` 字段被省略。

当前 `ast.rs` 中，`extract_node()` 从 `loc.file` 获取符号的声明文件路径，为空时回退到 `self.file`（翻译单元文件路径）。这导致所有头文件声明的符号（类定义、枚举、函数声明、typedef 等）的 `file_path` 指向 `.cpp` 文件。

## Goals / Non-Goals

Goals:
- 头文件符号的 `file_path` 指向正确的 `.h` 文件
- 外部符号判定（`is_external`）使用正确的文件上下文
- 全量测试通过，无回归

Non-Goals:
- 不修改 Clang 调用方式
- 不修改 `clang_filter.py`

## System Architecture

改动集中在 `ExtractCtx` 的 `cur_file` 字段管理：

```
extract_symbols_and_edges(file_path)
  └─ ExtractCtx { cur_file: file_path, ... }
      └─ extract_node(node, ns, parent)
           ├─ 解析 loc.file → 有则更新 self.cur_file
           ├─ 符号写入 → file_path = cur_file
           └─ 递归子节点 → cur_file 传递给子树
```

## Decisions

### Decision: 追踪 `cur_file` 而非 `self.file` 回退
选择在 `ExtractCtx` 中新增 `cur_file` 字段，代替原先所有 `self.file` 回退点。

理由：改动最小（6 处替换 + 1 处字段追加），不改变方法签名，不破坏现有递归结构。

### Decision: 不回退到 `includedFrom`
Clang 的 `-ast-dump=json` 实际上不包含 `includedFrom` 字段（与 known-limitations 旧描述不同）。唯一可行方案是 AST 上下文追踪。

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|---------|
| 头文件中的系统符号 `file_path` 也变成 system header 路径 | 外部判定逻辑也改用 `cur_file`，判定结果正确 |
| 测试覆盖不足 | clang_test fixture 含头文件和 TU，覆盖了实测 |

## Migration Plan

1. `ast.rs`: ExtractCtx 加 `cur_file` 字段并初始化
2. `ast.rs`: 所有 `self.file` 回退改为 `self.cur_file`
3. `ast.rs`: `is_external` 判定改为 `self.cur_file`
4. 清旧 DB 重索引验证
