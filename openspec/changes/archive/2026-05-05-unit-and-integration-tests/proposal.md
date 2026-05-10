# Proposal: 单元测试 + 集成测试

## Why

当前 CodeLoom 仅有 `ignore.rs` 中 4 个 `.codeloomignore` 测试和 `main.rs` 中 1 个调试用 AST dump 测试。storage、query、indexer、mcp、embedding、config 六大核心模块没有任何自动化测试。用户要求"后面每次 SDD Change 都需要用例通过才行"，必须先补齐测试基础设施。

## What Changes

- 新增 33 个单元测试（`#[cfg(test)]`），覆盖 5 个核心模块
- 新增 10 个集成测试（`tests/` 目录），用 leveldb/flatbuffers 做端到端验证
- 集成测试不依赖外部网络（模型已在 models/ 目录）

## Capabilities

### Modified Capabilities

- `cli-mode`: 新增测试验证 CLI index/status 命令
- `mcp-server`: 新增测试验证工具列表、branch 必传、错误响应
- `code-indexing`: 新增测试验证 tree-sitter 解析和 git 集成
- `doc-indexing`: 新增测试验证文档索引

## Impact

- 二进制不变（测试代码不编译进 release）
- `cargo test` 单元测试耗时 < 5 秒
- 集成测试耗时 < 30 秒（不含模型加载）
- 从本变更开始，所有后续 SDD Change 必须 `cargo test` 通过
