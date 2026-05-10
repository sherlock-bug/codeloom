# call-graph-module

## Purpose
CodeLoom 调用图模块——src/query/call_graph.rs 提供 get_call_graph() API，支持指定符号名、方向（callers/callees）和递归深度，返回文本格式调用树，MCP 工具和 CLI 共用。

## Requirements

### Requirement: call_graph 模块 API
`src/query/call_graph.rs` SHALL 提供 `get_call_graph()` 公有函数，接受连接、符号名、仓库、分支、方向和深度参数，返回文本格式的调用图。

#### Scenario: 查询特定函数的被调用者
- GIVEN 数据库中 `DBImpl::Get` 调用了 `MemTable::Get`
- WHEN 调用 `get_call_graph(conn, "DBImpl::Get", "leveldb", "main", "callees", 2)`
- THEN 返回结果 SHALL 包含 `MemTable::Get` 的层级化调用树

#### Scenario: 查询调用者
- GIVEN 数据库中 `DoCompactionWork` 被 `BackgroundCompaction` 调用
- WHEN 调用 `get_call_graph(conn, "DoCompactionWork", "leveldb", "main", "callers", 1)`
- THEN 返回结果 SHALL 包含 `BackgroundCompaction` 作为调用者

### Requirement: MCP 工具调用 call_graph 模块
`codeloom_get_call_graph` MCP 工具 SHALL 改为调用 `src/query/call_graph::get_call_graph()` 而非内联实现。

#### Scenario: MCP 工具行为不变
- GIVEN MCP 客户端请求 `codeloom_get_call_graph`
- WHEN 参数为 `{"name": "DB::Put", "repo": "leveldb", "branch": "main", "direction": "callees", "max_depth": 1}`
- THEN 返回的 JSON-RPC 格式 SHALL 与重构前完全一致

### Requirement: 模块不引入新依赖
`call_graph.rs` SHALL 仅依赖 `rusqlite::Connection` 和 `std::collections::HashSet`，不引入新 crate。

#### Scenario: 编译检查
- GIVEN 项目 Cargo.toml
- WHEN 执行 `cargo build`
- THEN SHALL 不引入新外部依赖，编译成功

