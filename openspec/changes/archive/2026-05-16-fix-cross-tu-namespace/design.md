# Design: fix-cross-tu-namespace

## Context
当前 indexer 在生成方法/函数节点名时 strip 了 namespace 前缀。例如 `cross_tu::Handler::handle` 只存为 `Handler::handle`，namespace 存在 `sym.namespace` 字段中。但调用边 extractor 从 `qualType` 提取 target_name 时却包含了 namespace（如 `"cross_tu::Handler *"` → `"cross_tu::Handler::handle"`），导致节点名与 target_name 不匹配，跨 TU 调用边丢失。

## Goals / Non-Goals
Goals:
- 节点名统一包含 namespace 前缀
- 跨 TU 调用边全部补齐（G17 通过）
- leveldb 场景验证（`Env::GetChildren` 等）

Non-Goals:
- 更改 namespace 字段存储格式
- 非 C++ 语言支持

## Decisions

### Decision: FQN 节点命名
选择将 namespace 前缀加入 `sym.name`，而非从 extractor 的 qualType 中剥离 namespace。理由：
- namespace 是身份标识的一部分，FQN 语义更正确
- extractor 本身也生成 FQN target_name，命名统一后无需额外剥离逻辑
- 查询工具（inspect、search）可用 FQN 精确定位

### Decision: 精确 FQN 匹配（内部），模糊前缀匹配（外部）
跨 TU 边匹配：node name 和 extractor target_name 都用 FQN，`name_to_id` 精确查找即可。
外部接口（inspect/search/call_graph）：用户可能用不完整的名称查询，走 FTS5/LIKE 前缀匹配。

不引入 namespace 剥离逻辑——内部精确匹配，外部自动 FTS 模糊。

### Decision: filter.py 保留，不碰
跨 TU 场景中 filter.py 的 `loc: null` 问题（DeclStmt 导致子树被 strip）是独立 bug，本次不做修改。本 change 专注边缘匹配逻辑。

### Decision: line_end 从 attrs JSON 迁移到列
`line_end` 在原始 schema 中同时存在于 `nodes` 表列和 `attrs` JSON 中，但不同 merge 路径（same-file vs cross-file）只更新其中一处，导致双写不一致（G5d 断言失败：.cc 定义行号被 .h 声明的 attrs 旧值覆盖）。
选择迁移到列唯一存储，理由：
- 消除双写根源——列是唯一可靠来源，后续不会再有同步问题
- INSERT/SELECT/merge 路径统一操作列而非 JSON
- 所有阅读端（MCP inspect、CLI inspect）从 `n.line_end` 读取，不走 `json_extract`
- 涉及 5 个文件改动：`symbols.rs`（建表+INSERT）、`nodes.rs`（struct+SELECT）、`clang/mod.rs`（merge 路径）、`mcp/mod.rs`、`cli/mod.rs`

## Risks / Trade-offs
| 风险 | 缓解措施 |
|------|---------|
| FQN 命名改变影响所有已有 fixtures | 重新索引，更新已知断言 |
| 剥离 namespace 的 LIKE 可能误匹配 | 从最完整到最精简有序尝试，短路第一个匹配 |
