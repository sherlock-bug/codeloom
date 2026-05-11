# Delta for test-layering

## ADDED Requirements

### Requirement: 测试分层门禁
系统 SHALL 将测试分为快速门禁层和慢集成层，快速门禁支持秒级验证，慢集成层标记 `#[ignore]`。

#### Scenario: 快速门禁验证
- GIVEN 开发者修改了单个解析器函数
- WHEN 运行 `cargo test` 或 `make test`
- THEN 所有单元测试和快速集成测试 SHALL 在 5 秒内完成
- AND 不要求先编译 release binary

#### Scenario: 完整回归验证
- GIVEN 开发者准备提交或上线
- WHEN 运行 `make test-full`
- THEN 所有测试 SHALL 运行（含 `#[ignore]` 标记的慢集成测试）
- AND 先编译 release binary 以确保二进制最新

### Requirement: 测试失败修复
测试套件 SHALL 全通过，无硬编码不存在的仓库名，搜索查询 SHALL 匹配当前 DB schema。

#### Scenario: 缺失分支错误测试
- GIVEN 测试需要检查缺少 branch 参数时的错误提示
- WHEN 调用 MCP 工具的 `codeloom_search` 方法，传入存在的 repo 但不传 branch
- THEN 返回的响应 SHALL 包含 "branch is required" 错误信息
- AND 测试使用的仓库名 SHALL 在测试 DB 中存在

#### Scenario: 中文语义搜索测试
- GIVEN zhsearch 仓库已索引中文语义搜索 fixture
- WHEN 用中文关键词搜索
- THEN 返回结果 SHALL 不为空，且搜索 SQL SHALL 正确匹配当前 DB schema

### Requirement: 快速集成测试用 debug binary
默认不跑的集成测试 SHALL 使用 `target/debug/codeloom` 作为测试 binary，避免每次修改后需要 54 秒的 release 编译。

#### Scenario: 快速测试流程
- GIVEN 开发者完成了代码修改
- WHEN 运行 `cargo test`
- THEN 测试工具 SHALL 使用 `target/debug/codeloom`（已由 `cargo build` 生成）
- AND 无需等待 `cargo build --release`

### Requirement: Makefile 测试目标
Makefile SHALL 提供 `test` 和 `test-full` 两个 target，分别对应快速门禁和完整回归。

#### Scenario: 日常验证
- GIVEN 开发者完成修改
- WHEN 在项目根目录执行 `make test`
- THEN SHALL 执行 `cargo build`（debug 模式） + `cargo test`（含快速集成测试） + 不运行 `#[ignore]` 测试

#### Scenario: 提交前回归
- GIVEN 开发者准备提交代码
- WHEN 执行 `make test-full`
- THEN SHALL 执行 `cargo build --release` + `cargo test -- --ignored` + `cargo test`
