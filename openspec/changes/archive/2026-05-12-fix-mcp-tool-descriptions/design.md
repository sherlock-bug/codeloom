# Design: MCP 工具描述修复

## Context

当前 `src/mcp/mod.rs::tools_list()` 中注册了 13 个 MCP 工具，其中有效 11 个（`codeloom_status` 和 `codeloom_get_doc`、`codeloom_query_excel` 已 DISABLED）。审查发现 description 中存在以下问题：

1. `codeloom_index` 的描述末尾引用了已 DISABLE 的 `codeloom_status`，LLM 调用该工具会失败
2. `codeloom_schema` 的描述包含"调用图类工具前须先调用此工具"的强制指令，实际 LLM 只在拿不准参数值时才需要查看 schema
3. `codeloom_get_call_graph` 描述中的"唯一方式"措辞不够精确——LLM 也可通过 neighbor_graph 拼出调用链，只是效率更低
4. `codeloom_list_repos` 描述末尾的输出示例属于输出格式噪音

## Goals

- 去除所有已失效的工具引用
- 使 schema 工具的描述从"强制前置"改为"按需参考"
- 提升 call_graph 的竞争区分度
- 精简噪音

## Non-Goals

- 不改 inputSchema / required 字段
- 不改实现逻辑
- 不新增/删除工具

## System Architecture

无架构变更。仅修改 `tools_list()` 函数中的工具描述字符串，该函数在 MCP 客户端请求 `tools/list` 时返回静态 JSON。

## Decisions

### Decision 1: `codeloom_index` 死链 → 引用 `codeloom_list_repos`

原描述末尾："用codeloom_status确认状态"
修改为："用codeloom_list_repos检查是否已索引"

`codeloom_list_repos` 是有效的活跃工具，能返回已索引仓库列表。

### Decision 2: `codeloom_schema` 强制指令 → 按需引导

原描述末尾："调用图类工具前须先调用此工具"
修改为："拿不准参数值时调用此工具查看可用节点类型和边类型枚举"

这样 LLM 在三种场景下会主动调用 schema：
- 不确定 edge_filter 有哪些可用的边类型时
- 不确定 kind 过滤参数有哪些可用值时
- 想了解工具的元数据输出结构时

### Decision 3: `codeloom_get_call_graph` 加竞争对比

原描述开头："**唯一方式**：分析函数/方法的调用者和被调用者"
修改为："**首选方式**：分析函数/方法的调用者和被调用者（callers/callees）。优于多次调neighbor_graph拼凑——一次到位且带递归深度控制"

保留与 grep 的竞品对比（grep无法获取调用关系），加上与 neighbor_graph 的对比，因为这是 LLM 真正会用来替代 call_graph 的工具。

### Decision 4: `codeloom_list_repos` 精简输出格式

原描述末尾："返回如\"codeloom\nleveldb\nspdlog\"。"
删除该示例句。仅保留前段，以"无需任何参数。"结尾。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| LLM 不再主动调 schema 导致传错参数值 | schema 描述已写明"拿不准时调用"，需要时 LLM 自会调。且所有图工具的参数都有 enum/default，LLM 通常不需要 schema |
| CI 缓存旧 binary 导致新描述未生效 | 改后必须 `cargo build --release` 重新编译部署 |
