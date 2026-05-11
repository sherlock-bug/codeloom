# Proposal: test-layering-and-fix

## Intent

当前测试体系将所有集成测试混在一起跑，无快慢分层，且 2 个测试因 schema 重构遗留问题失败。目标是建立三层测试门禁，修复失败的测试，让日常开发能秒级验证。

## Scope

In scope:
- 修复 `test_mcp_missing_branch_error` 和 `test_chinese_semantic_search` 两个失败测试
- 将集成测试按编译+运行时间分为快速门禁和慢集成两层
- 快速门禁用 debug binary（省掉 54s release 编译）
- 添加 `make test`（快速门禁）和 `make test-full`（完整回归）target

Out of scope:
- 新增测试用例（只重组和修复现有测试）
- CI/CD 配置变更
- 测试覆盖率提升

## Approach

三层分层：
1. **快速门禁** (`cargo test`) — 现有 65 个单元测试（0.05s），无需任何改动
2. **慢集成测试** (`cargo test -- --ignored`) — 标记为 `#[ignore]`，需显式运行；统一用 release binary
3. **完整回归** (`make test-full`) — 先编译 release，再跑所有测试

快速集成测试移除对 release binary 的依赖，改用 debug binary，降低日常验证门槛。2 个失败测试分别修复：改硬编码 repo 名 + 清旧 DB schema 残骸。

## Capabilities

- `test-gate-layering`: 将集成测试分为 fast（默认不跑）和 slow（标记 `#[ignore]`）两层
- `fast-test-debug-binary`: 快速集成测试用 debug binary，免 release 编译
- `make-test-targets`: 添加 `make test` 和 `make test-full` 命令
