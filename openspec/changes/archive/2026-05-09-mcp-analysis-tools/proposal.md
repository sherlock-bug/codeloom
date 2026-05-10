# Proposal: MCP 分析工具层

## Why
CodeLoom 的 edges 表已覆盖 9 种语义关系（calls, inherits, contains, uses, references 等），但 MCP 工具只暴露了单跳查询（`codeloom_get_call_graph`、`codeloom_inspect`），LLM 无法做多跳图遍历。`codeloom_overview` 输出统计数字对 LLM 无效。

核心缺口：LLM 拿到 edges 数据后需要自己拼图——"谁调用了A"→手动追踪→"调用了A的B又调用了什么"——每次手动追踪都要多轮 MCP 调用，浪费 token 和时间。

## What Changes
1. **删除** `codeloom_overview`（对 LLM 无意义）
2. **保留 + 增强** `codeloom_get_call_graph`：维持纯调用分析降低噪声，终端节点附带其使用的枚举值/全局变量/字符串字面量
3. **新增** 4 个 MCP 分析工具，全部基于现有 edges 表做图遍历

## Capabilities

### New Capabilities
- `path-analysis`: 跨边类型的 BFS 路径分析，支持 shortest/all 两种模式 + edge_filter 指定边类型 + 环检测 + max_paths 上限
- `impact-analysis`: 传递闭包影响分析，支持方向控制（forward/reverse/both）
- `neighbor-graph`: 单符号邻里图，支持方向控制，按边类型分组
- `inheritance-tree`: 继承体系树形分析，含虚拟方法分发和 override 列表
- `schema-metadata`: 导出当前 CodeLoom 的所有节点类型、边类型、属性及其说明，供 LLM 查询可用能力

### Modified Capabilities
- `enhanced-call-graph`: `codeloom_get_call_graph` 终端节点附带 enum/global/string_literal 信息

### Removed Capabilities
- `remove-overview`: 删除 `codeloom_overview`

## Impact
- 改动范围：`src/mcp/mod.rs`（工具注册/删除/增强）+ `src/query/`（新增 graph 引擎 + 4 个工具模块）+ `src/query/call_graph.rs`（增强）
- 不动索引层、不动 schema、不动 data model
- 测试：新增集成测试覆盖 5 个工具（含增强后的 call_graph）
- 删除 overview 后需更新 MCP tool 列表验证
