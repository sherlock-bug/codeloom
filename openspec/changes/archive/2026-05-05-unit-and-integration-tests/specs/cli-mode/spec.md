# Delta for cli-mode

## ADDED Requirements

### Requirement: 自动化测试覆盖
系统 SHALL 提供单元测试和集成测试，验证 `codeloom index` 和 `codeloom status` CLI 命令的正确性。`cargo test` SHALL 在 30 秒内完成全部测试。

#### Scenario: index + status 集成测试
- GIVEN leveldb 源码在 D:/code/leveldb
- WHEN 执行 `codeloom index /path/to/leveldb --repo testdb --branch main`
- THEN symbols > 1000, edges > 5000, docs > 50
- WHEN 执行 `codeloom status --repo testdb`
- THEN 输出包含 Repo: testdb, Symbols:, Edges:, Docs:

#### Scenario: cargo test 全部通过
- GIVEN 测试代码已编写
- WHEN 执行 `cargo test`
- THEN 所有测试通过，不依赖外部网络
