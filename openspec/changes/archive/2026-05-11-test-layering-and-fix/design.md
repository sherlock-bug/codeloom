# Design: test-layering-and-fix

## Context

当前 CodeLoom 有 65 个单元测试和 10 个集成测试。单元测试在 0.05s 内全部通过，但集成测试混在一起跑需要 ~5s，且需要先 `cargo build --release`（~54s）。这意味着每次改代码后的完整验证链路长达 1 分钟。

此外，2 个集成测试失败：
- `test_mcp_missing_branch_error`：硬编码 `repo:"leveldb"`，该 repo 在测试 DB 中不存在
- `test_chinese_semantic_search`：DB schema 是旧版（`branch_name` TEXT），代码期望新版（`branch_id` INTEGER）

## Goals / Non-Goals

Goals:
- 修复 2 个失败测试
- 集成测试分为 fast（默认不跑）和 slow（`#[ignore]`）
- 快速集成测试用 debug binary
- Makefile 添加 `test` 和 `test-full` target

Non-Goals:
- 不改 CI/CD
- 不新增测试
- 不改覆盖率

## System Architecture

```
                    ┌──────────────────────────────┐
                    │       cargo test (0.05s)      │
                    │  65 单元测试，无外部依赖       │
                    └──────────┬───────────────────┘
                               │
                    ┌──────────▼───────────────────┐
                    │   make test (0.5s + debug)   │
                    │  单元 + 快速集成（默认不跑）  │
                    └──────────┬───────────────────┘
                               │
                    ┌──────────▼───────────────────┐
                    │ make test-full (55s + rel)   │
                    │ 全部 75+ 测试，含编译         │
                    └──────────────────────────────┘
```

## Decisions

### Decision: 快速集成测试用 debug binary
除 `test_clang_parser` 和 `test_multi_format_index` 两个已用 debug binary 的测试外，其余快速集成测试也改用 `target/debug/codeloom`。

理由：省掉 54s release 编译。debug binary 对索引和查询功能无性能差异。

### Decision: `#[ignore]` 标记慢测试
全部集成测试标记 `#[ignore]`，默认不运行。快速集成测试作为普通集成测试不标记 `#[ignore]`，但只包含不依赖 release binary 的轻量测试。

理由：Rust 生态中 `#[ignore]` 是标准的慢测试标记方式，`cargo test -- --ignored` 运行所有慢测试。

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|---------|
| `#[ignore]` 导致回归测试遗漏 | Makefile `test-full` 显式运行 ignored 测试 |
| debug binary 性能差异 | 索引和查询主要瓶颈在 SQLite 和 I/O，非 Rust 优化 |
| 快速集成测试顺序依赖 | 每个 test 独立索引独立 fixture，无共享状态 |

## Migration Plan

1. 修复 2 个失败测试（改 repo 名 + 清旧 DB）
2. 快速集成测试改 debug binary
3. 标记慢测试为 `#[ignore]`
4. 更新 Makefile
