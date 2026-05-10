# Proposal: GB2312/GBK 编码支持

## Why

部分工程源码使用 GB2312 或 GBK 编码（非 UTF-8），当前 `std::fs::read_to_string` 默认假定 UTF-8，遇到 GB2312 文件直接报错，导致索引失败、符号提取为 0。

## What Changes

- 新增 `read_file_smart()` 工具函数：先试 UTF-8，失败则用 GB18030（兼容 GBK/GB2312）解码
- `parse_file`、`index_includes`、`index_docs` 三处改用 `read_file_smart`
- 新增 `encoding_rs` crate 依赖（Firefox 同款 GBK 解码器，~200KB 编译增量）
- 新增 GB2312 测试 fixture 和单元测试

## Capabilities

### New Capabilities
- `gb2312-encoding-support`: 源码文件编码自动检测与转码 — UTF-8 优先，GB18030 兜底

## Impact

- `Cargo.toml` — 新增 `encoding_rs` 依赖
- `src/indexer/tree_sitter.rs` — `parse_file` 改用 `read_file_smart`
- `src/cli/mod.rs` — `index_docs`、`index_includes` 改用 `read_file_smart`
- `src/util.rs`（新建）— `read_file_smart()` 函数
- `tests/fixtures/gb2312/`（新建）— GB2312 编码的 C++ 测试文件
