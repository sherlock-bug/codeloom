# Design: MCP 分析工具层

## Context
CodeLoom 已有数据：symbols 表 + edges 表（9 种 edge_type）。目标：保留调用分析降低噪声，新增 4 个图遍历工具，删除 overview。不动索引层，纯查询层改造。

## Goals / Non-Goals
**Goals**：
- 保留 `codeloom_get_call_graph`，增强终端节点信息
- 4 个新工具：路径分析、影响分析、邻里图、继承树
- 所有工具支持 direction 参数（forward/reverse/both）
- 路径分析支持全路径模式 + 环检测
- 删除 `codeloom_overview`

**Non-Goals**：
- 不改 schema、不改索引逻辑、不新增边类型
- 不做图数据库（纯 SQL 遍历）

## Decisions

### Decision 1: 保留调用分析，纯函数调用降低噪声
邻里图包含所有边类型，噪音大。LLM 问"谁调了 validate"时只需要 calls 边。保留 `codeloom_get_call_graph` 作为轻量专用工具。

**增强**：终端节点（遍历到的每个函数/方法）附带该节点使用的枚举值、引用的全局/静态变量、包含的字符串字面量，让 LLM 拿到完整依赖卡片。

### Decision 2: 全 SQL 实现，不用图库
用递归 CTE 实现 BFS 路径搜索和传递闭包。理由：edges 表规模可控（leveldb ~10K 条），SQL CTE 足够覆盖。

### Decision 3: 三向方向控制
所有工具统一支持 `direction` 参数：
- `forward`：只沿边正向（A→B）
- `reverse`：只沿边反向（B→A，即"谁指向了 A"）
- `both`：双向
- 默认值：影响分析 `reverse`（"改了 X 影响谁"），邻里图 `both`（"X 是谁"），路径分析 `both`

### Decision 4: 路径分析双模式 + edge_filter + 环检测
- `mode=shortest`（默认）：BFS 找到第一条有效路径即返回
- `mode=all`：BFS 穷举所有有效路径，受 `max_paths` 限制（默认 20）
- `edge_filter`：限定走哪些边类型，LLM 通过 `codeloom_schema` 获取可用边类型后自行选择。不指定时走所有边类型
- 环检测：路径上出现重复节点 ID 则丢弃该路径

### Decision 5: Schema 元数据独立工具
新增 `codeloom_schema` 工具，导出所有节点类型（symbol kind）、边类型（edge_type 前缀）、属性说明。LLM 一次调用即知可用能力，不再在提示词中硬编码，节省上下文 token。

### Decision 6: 继承树独立于邻里图
四种边关系（inherits/overrides/contains:method/field_type）构成继承树的专用数据源。独立工具 `codeloom_inheritance_tree` 用递归 CTE 构建树形结构，输出带 `children` 数组的 JSON。

### Decision 7: 邻里图按边类型分组输出
`codeloom_neighbor_graph` 返回按边类型分组的邻居列表，层级为 1-2 跳。LLM 拿到一个干净的邻接分类。

## System Architecture

```
MCP Router (src/mcp/mod.rs)
  ├─ codeloom_schema             → src/mcp/mod.rs (inline)      (元数据)
  ├─ codeloom_get_call_graph     → src/query/call_graph.rs      (增强)
  ├─ codeloom_path_analysis     → src/query/path.rs             (BFS)
  ├─ codeloom_impact_analysis   → src/query/impact.rs           (闭包)
  ├─ codeloom_neighbor_graph    → src/query/neighbor.rs         (邻里)
  ├─ codeloom_inheritance_tree  → src/query/inheritance.rs      (继承)
  └─ [REMOVED]                  → codeloom_overview
```

共享引擎：`src/query/graph.rs` — 通用 BFS、邻接查询、传递闭包。

## API Design

### codeloom_get_call_graph (增强后)
```
输入: name=DBImpl::Get, direction=callers, max_depth=2
输出:
  DBImpl::Get
    ← called_by: validate
      uses: [Status::ERROR]               ← 新增
      references: [g_config]              ← 新增
      string_literals: ["invalid id"]     ← 新增
```

### codeloom_path_analysis
```
输入: source=main, target=handle_error, mode=all, max_paths=5
输出: {
  "paths": [
    {"edges": [{"from":"main","edge":"calls","to":"dispatch"}, ...]},
    {"edges": [...]}
  ],
  "total_found": 2
}
```

### codeloom_impact_analysis
```
输入: symbol=g_config, direction=reverse, radius=3
输出: {
  "affected": [
    {"symbol":"process_request","distance":1,"via":"references"},
    {"symbol":"main","distance":2,"via":"calls"}
  ]
}
```

### codeloom_neighbor_graph
```
输入: symbol=DBImpl, direction=both, depth=1
输出: {
  "neighbors": {
    "forward": {
      "calls": ["Env::NewSequentialFile"],
      "contains": ["DBImpl::Get"],
      "uses": ["Status::OK"]
    },
    "backward": {
      "inherits": ["DB"],
      "called_by": ["main"],
      "field_type_of": ["Database"]
    }
  }
}
```

### codeloom_inheritance_tree
```
输入: symbol=DB, direction=down, max_depth=5
输出: {
  "root": "DB",
  "children": [
    {"symbol":"DBImpl","overrides":["Write","Get"],
     "children": [{"symbol":"LevelDBImpl","overrides":["Write"],"children":[]}]}
  ]
}
```

## Risks / Trade-offs
| 风险 | 缓解 |
|------|------|
| 大仓库递归 CTE 可能慢 | leveldb ~10K edges，实测 <10ms |
| 全路径模式可能炸 | max_paths=20 上限 + 环检测 |
| 删除 overview 后 LLM 需适应 | 新工具 description 引导 LLM 从 search 开始 |
