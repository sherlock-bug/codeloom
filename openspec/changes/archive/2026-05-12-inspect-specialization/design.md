# Design: inspect-specialization

## Context

当前 `inspect_symbol()` 对所有节点类型输出统一的通用 edges 列表（按 edge_type 分组的关联节点名字列表）。LLM 拿到类结果时仍需要额外调 neighbor_graph 才能看到成员变量、方法。需要根据节点类型智能返回最关心的信息。

## Approach

在 `inspect_symbol()` 中，对每个符号节点获取其 `node_type` + `kind`，按类型分支查询：

### class/struct
- 基类：`edges.edge_type LIKE 'inherits:%'` → 返回 id + name
- 成员变量：`edges.edge_type = 'contains:' AND nodes.kind = 'field'` → 返回 id + name + type
- 方法：`edges.edge_type = 'contains:' AND nodes.kind = 'method'` → 返回 id + name + signature
- 模板参数：`json_extract(nodes.attrs, '$.template_args')` → 字符串

### enum
- 枚举值：`edges.edge_type = 'contains:' AND nodes.kind = 'enum_value'` → 返回 id + name

### file
- 顶层 section：`edges.edge_type = 'contains:' AND nodes.node_type = 'section'` → 返回 id + name
- 找不到 file node 时（搜索只搜 file 名），直接 `SELECT id, name FROM nodes WHERE node_type='section' AND file_path = ?`

### section
- 父 section：反向 `contains:` edge → 返回 id + name
- 子 section：`contains:` + node_type=section → 返回 id + name（限 50 个）
- 子 chunk：`contains:` + node_type=chunk → 返回 id + name（限 50 个）
- 前一节：`file_path = ? AND id < ? ORDER BY id DESC LIMIT 1`
- 后一节：`file_path = ? AND id > ? ORDER BY id ASC LIMIT 1`

### chunk
- 父 section：反向 `contains:` edge → 返回 id + name
- 前一块：同父节点下 `id < ? ORDER BY id DESC LIMIT 1`
- 后一块：同父节点下 `id > ? ORDER BY id ASC LIMIT 1`

### 其余节点类型
保持当前通用 edges 输出（按 edge_type 分组的关联节点列表，LIMIT 200）。

## Output Format

保持 JSON 格式不变（与当前结构兼容，但 edges 部分改为类型专属键名）。

## Decisions

### Decision: 保持 JSON 字符串构建
当前 inspect 使用 `format!()` + `push_str()` 手动构建 JSON。虽然粗糙但性能好，且与当前 MCP 输出格式一致。不改为 serde_json。

## Risks

| 风险 | 缓解 |
|------|------|
| section/file 可能无对应 edges（全量索引才产生） | 回退到直接 SQL `SELECT ... WHERE file_path = ?` |
| chunk 节点不在搜索结果中 | 但通过 inspect 可查。用 doc_id 或直接查 nodes |
