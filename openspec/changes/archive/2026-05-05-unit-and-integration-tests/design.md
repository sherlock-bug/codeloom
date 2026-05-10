# Design: 单元测试 + 集成测试

## Context

CodeLoom 六大核心模块无自动化测试。所有改动依赖手工 MCP 调用验证。需要建立 `cargo test` 一键验证能力。

## Goals / Non-Goals

**Goals:**
- 33 单元测试覆盖 5 个模块的公开 API
- 10 集成测试覆盖全流程（index→query→status）
- `cargo test` 零失败

**Non-Goals:**
- 不追求 100% 覆盖率
- 不测试第三方 crate 内部逻辑

## Decisions

### Decision 1: 单元测试放模块内，集成测试放 tests/

Rust 惯例：`#[cfg(test)] mod tests` 测私有 API，`tests/` 测公开 CLI/MCP 行为。

### Decision 2: 集成测试用 std::process::Command 调 codeloom 二进制

不链接 codeloom 库——直接 `cargo build` 产出二进制后，集成测试调 CLI 和 MCP stdio。

### Decision 3: 测试素材用 D:\code\leveldb 和 flatbuffers

已存在于本地，无需网络下载。leveldb (132 C++ files) 适合代码索引测试，flatbuffers (63 .md) 适合文档索引测试。

### Decision 4: 使用 mockall 框架进行有限 mock

`mockall` 用于 mock `Embedder` trait，但仅限 TextEmbedder 降级场景测试。CandleEmbedder SHALL 使用真实模型（models/bge-small-zh/ 已由 build.rs 确保存在），加载和推理必须通过集成测试验证。

### Decision 5: 覆盖率 60% + 模型路径必须覆盖

使用 `cargo tarpaulin` 测量行覆盖率。目标：行覆盖 ≥ 60%。`CandleEmbedder::get_or_load`、`embed()`、`forward()` 等模型相关路径必须被测试覆盖，不能跳过。

## Test Structure

```
codeloom/
├── src/
│   ├── embedding/mod.rs    ← +8 单元测试
│   ├── storage/mod.rs      ← +8 单元测试
│   ├── mcp/mod.rs          ← +6 单元测试
│   ├── indexer/mod.rs      ← +7 单元测试
│   └── config/mod.rs       ← +4 单元测试
└── tests/
    ├── integration.rs      ← 索引+查询 全流程
    ├── branch_filter.rs    ← 分支过滤
    ├── semantic.rs         ← 语义搜索
    └── mcp_tools.rs        ← MCP 工具验证
```

## Risks

| 风险 | 缓解 |
|------|------|
| 集成测试依赖 models/ 目录 | build.rs 已确保 models/ 存在，否则跳过 candle 测试 |
| D:\code 路径在 CI 不可用 | 集成测试检测路径存在性，不存在时跳过 |
| candle 模型加载慢 | 语义搜索测试标记 `#[ignore]`，需要时 `cargo test -- --ignored` |
