# Tasks

## 1. 修复失败测试

- [x] 1.1 `test_mcp_missing_branch_error`：将 `repo:"leveldb"` 改为一个存活的测试 repo（如 `"ix"` 或 `"bf"`），验证错误消息含 "branch is required"
- [x] 1.2 `test_chinese_semantic_search`：清掉 zhsearch 的旧 DB，重新索引，确认搜索返回正确结果

## 2. 快速集成测试改 debug binary

- [x] 2.1 将 `const CODELOOM` 常量从 `"target/release/codeloom"` 改为 `"target/debug/codeloom"`
- [x] 2.2 更新 error message 文案（`cargo build --release` → `cargo build`）

## 3. 标记慢集成测试为 `#[ignore]`

- [x] 3.1 在文件头部添加注释说明 `#[ignore]` 分层规则
- [x] 3.2 标记 9 个集成测试为 `#[ignore]`，仅 `test_mcp_tools_list` 不标记（纯 JSON-RPC 无文件/DB 依赖）

## 4. 更新 Makefile

- [x] 4.1 添加 `test` target：`cargo build && cargo test`（仅快速门禁，`#[ignore]` 不跑）
- [x] 4.2 添加 `test-full` target：`cargo build --release && cargo test -- --ignored && cargo test`（完整回归）
- [x] 4.3 验证 `make test` 和 `make test-full` 均通过
