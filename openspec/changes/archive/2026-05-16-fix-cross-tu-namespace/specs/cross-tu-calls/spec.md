# Delta for cross-tu-calls

## ADDED Requirements

### Requirement: 节点名包含 namespace 前缀
方法/函数的节点名 SHALL 包含完整的 namespace 前缀。如跨 TU 命名空间 `cross_tu` 中的类 `Handler` 的方法 `handle`，其节点名 SHALL 为 `cross_tu::Handler::handle`（而非 `Handler::handle`）。

#### Scenario: namespace 内方法的 FQN 节点名
- GIVEN 一个定义在 `cross_tu` 命名空间下的类 `Handler`，其方法 `handle` 在另一 TU 中定义
- WHEN 索引该代码
- THEN DB 中的节点名 SHOULD 为 `cross_tu::Handler::handle`

#### Scenario: 跨 TU 调用边通过 FQN 匹配
- GIVEN 函数 `call_handler` 通过指针调用 `cross_tu::Handler::handle`
- WHEN 索引后查询 `call_handler` 的调用边
- THEN 调用图中 SHOULD 包含 `call_handler` → `cross_tu::Handler::handle` 边

### Requirement: 跨 TU 调用边 DB fallback（精确 FQN 匹配）
当跨 TU 场景下 `name_to_id` 中找不到目标函数时，系统 SHALL 回退到 SQLite DB 查询，使用精确 FQN 匹配目标节点。不需要 namespace 剥离或 LIKE 模糊匹配——FQN 命名统一后两边一致。

#### Scenario: 直接调用跨 TU 边
- GIVEN 函数 `compute` 调用 `cross_tu::Calculator::add`，后者在另一 TU 中定义
- WHEN 索引后查询 `compute` 的调用边
- THEN 调用图中 SHOULD 包含 `compute` → `Calculator::add` 边

#### Scenario: 指针调用跨 TU 边（global namespace）
- GIVEN 函数 `call_via_pointer` 通过指针调用全局类 `Client::process`（无 namespace），后者在另一 TU 中定义
- WHEN 索引后查询 `call_via_pointer` 的调用边
- THEN 调用图中 SHOULD 包含 `call_via_pointer` → `Client::process` 边

#### Scenario: 指针调用跨 TU 边（带 namespace）
- GIVEN 函数 `call_handler` 通过指针调用 `cross_tu::Handler::handle`（带 namespace），后者在另一 TU 中定义
- WHEN 索引后查询 `call_handler` 的调用边
- THEN 调用图中 SHOULD 包含 `call_handler` → `cross_tu::Handler::handle` 边
