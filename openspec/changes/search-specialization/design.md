# Design: search-specialization

## Context

当前搜索工具对所有节点类型返回统一的字段集。LLM 拿到结果后还需多次调 `inspect` 才能判断节点是否匹配需求。

tool-id-param (⑤) 已为搜索工具输出添加了 `id` 字段。

## Data Sources

| 类型 | 字段 | 来源 |
|------|------|------|
| class/struct | 成员字段名列表 | edges `contains:` + nodes.kind='field' → nodes.name, LIMIT 15 |
| class/struct | 方法名列表 | edges `contains:` + nodes.kind='method' → nodes.name, LIMIT 15 |
| enum | 枚举值名列表 | edges `contains:` + nodes.kind='enum_value' → nodes.name, LIMIT 15 |
| function/method | 所属类名 | nodes.attrs `parent_class` |
| section | 前一章节标题 | 同 file_path 按 id 降序取上一个 section 的 name |
| section | 后一章节标题 | 同 file_path 按 id 升序取下一个 section 的 name |
| chunk | 所属 section 名 + 前后 chunk | reverse contains: + 同父节点按 idx 排序的相邻 chunk |
| file | 顶层 section 列表 | nodes.type='section' AND file_path=当前文件 → nodes.name, LIMIT 15 |

## Approach

### 1. FusedResult 结构调整

```rust
pub struct FusedResult {
    // 现有字段不变
    pub id: i64, pub score: f64, pub name: String, ...

    // 新 enrichment 字段（逗号分隔的字符串列表）
    pub members: String,      // 字段名列表，如 "name,age,email"
    pub methods: String,      // 方法名列表，如 "getUser,setName"
    pub values: String,       // 枚举值列表，如 "RED,GREEN,BLUE"
    pub parent_class: String, // 所属类名
    pub prev_section: String, // 前一章节标题
    pub next_section: String, // 后一章节标题
    pub prev_chunk: String,   // 前一 chunk 标题
    pub next_chunk: String,   // 后一 chunk 标题
    pub sections: String,     // 文件顶层 section 列表
    pub parent_section: String, // 所属 section 名
}
```

### 2. 填充逻辑

在 `enrich_search_results()` 中，对每个结果按类型查询：

class/struct:
```sql
-- members（字段名）
SELECT n.name FROM edges e JOIN nodes n ON e.target_id = n.id
WHERE e.source_id = ?1 AND e.branch_id = ?2 AND e.edge_type = 'contains:' AND n.kind = 'field'
ORDER BY n.id LIMIT 15

-- methods（方法名）
... AND n.kind = 'method' ...
```

enum:
```sql
SELECT n.name FROM edges e JOIN nodes n ON e.target_id = n.id
WHERE e.source_id = ?1 AND e.branch_id = ?2 AND e.edge_type = 'contains:' AND n.kind = 'enum_value'
ORDER BY n.id LIMIT 15
```

function/method:
```sql
SELECT json_extract(attrs, '$.parent_class') FROM nodes WHERE id = ?1
```

section:
```sql
-- prev_section
SELECT name FROM nodes WHERE file_path = ?1 AND node_type = 'section' AND id < ?2
ORDER BY id DESC LIMIT 1

-- next_section
SELECT name FROM nodes WHERE file_path = ?1 AND node_type = 'section' AND id > ?2
ORDER BY id ASC LIMIT 1
```

chunk:
```sql
-- parent_section name
SELECT n.name FROM edges e JOIN nodes n ON e.source_id = n.id
WHERE e.target_id = ?1 AND e.edge_type = 'contains:' AND e.branch_id = ?2 LIMIT 1

-- prev_chunk / next_chunk：同父节点（parent section）下按 idx 排序
```

file:
```sql
SELECT name FROM nodes WHERE file_path = ?1 AND node_type = 'section'
ORDER BY id LIMIT 15
```

### 3. MCP 输出变化

在 JSON 构造中附加 enrichment 字段，仅非空时输出。

## Decisions

### Decision: 逗号分隔字符串 vs JSON 数组
选择逗号分隔字符串。原因是 LLM 对搜索结果的消费方式通常是字符串模式匹配和上下文理解，逗号分隔足以判断"是否有想要的字段/方法"。JSON 数组增加解析复杂度，无实际收益。

### Decision: LIMIT 15
选择 LIMIT 15 避免输出膨胀。15 个字段/方法名足够 LLM 判断类型是否相关。超过 15 的类通常不是目标。

## Risks

| 风险 | 缓解 |
|------|------|
| 查询量增大（每个结果 2-3 条额外 SQL） | 结果数 ≤ 20，每条 SQL 有索引，总开销 < 1ms |
| 字段/方法名列表过长 | LIMIT 15 + 逗号分隔，合理压缩 |
| parent_class 在 attrs 中可能缺失 | 默认空字符串，JSON 输出时跳过 |
