# CodeLoom MCP 工具规格

> 共 12 active + 3 disabled，源码 `src/mcp/mod.rs`

---

## 1. codeloom_list_symbols

- **类型**: mcp | **状态**: active
- **源码位置**: `src/mcp/mod.rs:300`
- **用途**: 按名称模糊搜索已索引的符号（FTS5 BM25），返回纯文本列表
- **输入**:

| 参数 | 类型 | 必填 | 默认 | 说明 |
|------|------|------|------|------|
| pattern | string | Y | — | 符号名模糊匹配 |
| repo | string | Y | — | 仓库名 |
| branch | string | Y | — | 分支名 |
| limit | integer | N | 20 | 最大返回数 |

- **输出**: 纯文本 `id=[{kind:10}] {name:40} @ {file}:{line}`，空时 `(none)`
- **设计vs实现**: ✅ match

---

## 2. codeloom_get_call_graph

- **类型**: mcp | **状态**: active
- **源码位置**: `src/mcp/mod.rs:148`
- **用途**: 分析函数/方法的调用者或被调用者，支持递归深度
- **输入**:

| 参数 | 类型 | 必填 | 默认 | 说明 |
|------|------|------|------|------|
| id | integer | N | — | 符号节点 ID（优先使用） |
| name | string | N | — | 符号名 |
| repo | string | Y | — | — |
| branch | string | Y | — | — |
| direction | string | Y | — | `callers` / `callees` |
| max_depth | integer | N | 3 | 递归深度 |

- **输出**: JSON 字符串
- **设计vs实现**: ✅ match（注：描述说 id 优先，但 name 非空时也传 name 到下层函数）

---

## 3. codeloom_search (BM25)

- **类型**: mcp | **状态**: active
- **源码位置**: `src/mcp/mod.rs:562`
- **用途**: BM25 关键词搜索，结果含类型专属增强字段
- **输入**:

| 参数 | 类型 | 必填 | 默认 | 说明 |
|------|------|------|------|------|
| query | string | Y | — | 关键词 |
| repo | string | Y | — | — |
| branch | string | Y | — | — |
| kind | string | N | — | 符号类型过滤 |
| limit | integer | N | 10 | 最大返回数 |

- **输出**:
```json
{
  "results": [
    {
      "id": 123, "score": 0.85, "name": "AuthService",
      "type": "code", "file": "src/auth.rs",
      "kind": "class", "signature": "...",
      "line": 42, "snippet": "...",
      // 类型专属增强字段:
      "members": "username,password",   // class/struct
      "methods": "login,logout",        // class/struct
      "values": "ADMIN,USER",           // enum
      "parent_class": "AuthService",    // function/method
      "prev_section": "安装说明",       // section
      "next_section": "配置详解",       // section
      "parent_section": "快速开始",     // chunk
      "prev_chunk": "...",
      "next_chunk": "...",
      "sections": "安装,配置,API"       // file
    }
  ],
  "query": "...", "count": 5
}
```
- **边界**: 空结果返回空数组；无效 kind 返回错误；enrichment 空字段省略
- **SDD Change**: `search-enrichment`（已归档）
- **设计vs实现**: ⚠️ **偏差** — 工具描述的 kind 可选值列表（10 种）少于实际实现（15 种）

---

## 4. codeloom_semantic_search

- **类型**: mcp | **状态**: active
- **源码位置**: `src/mcp/mod.rs:602`
- **用途**: 自然语言语义向量搜索（仅符号，无文档）
- **输入**:

| 参数 | 类型 | 必填 | 默认 | 说明 |
|------|------|------|------|------|
| query | string | Y | — | 自然语言 |
| repo | string | Y | — | — |
| branch | string | Y | — | — |
| limit | integer | N | 10 | — |

- **输出**: 同 `search`，但**缺少 `id` 字段**，`type` 硬编码为 `"code"`
- **设计vs实现**: ⚠️ **偏差** — 与 BM25 搜索输出不一致：缺 `id`，`type` 写死 `"code"`（忽略实际 hit_type），违反搜索增强 spec 的跨工具一致性要求

---

## 5. codeloom_inspect

- **类型**: mcp | **状态**: active
- **源码位置**: `src/mcp/mod.rs:320`
- **用途**: 查看节点的结构化信息，按类型智能返回
- **输入**:

| 参数 | 类型 | 必填 | 默认 | 说明 |
|------|------|------|------|------|
| id | integer | N | — | 节点 ID |
| name | string | N | — | 符号名 |
| repo | string | Y | — | — |
| branch | string | Y | — | — |

- **输出**（按类型）:
  - class/struct → `bases+members+methods+terminals`
  - enum → `values+terminals`
  - section → `parent_section+children_sections+children_chunks+prev_section+next_section`
  - chunk → `parent_section+prev_chunk+next_chunk`
  - file → `sections`
  - 其他 sym → 通用 `edges` 分组 + terminals
- **SDD Change**: `inspect-enrichment`（已归档）
- **设计vs实现**: ⚠️ **偏差** — inspect-enrichment spec 要求 class/struct 包含 `template_args`，但实现中未查询

---

## 6. codeloom_list_repos

- **类型**: mcp | **状态**: active
- **源码位置**: `src/mcp/mod.rs:199`
- **用途**: 列出所有已索引仓库名
- **输入**: 无
- **输出**: 纯文本，每行一个仓库
- **设计vs实现**: ✅ match

---

## 7. codeloom_list_branches

- **类型**: mcp | **状态**: active
- **源码位置**: `src/mcp/mod.rs:207`
- **用途**: 列出指定仓库的所有已索引分支
- **输入**:

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| repo | string | Y | 仓库名 |

- **输出**: 纯文本分支列表
- **设计vs实现**: ⚠️ **偏差** — 描述说"返回分支名和符号数"，但实现只返回分支名，未查询符号数量

---

## 8. codeloom_schema

- **类型**: mcp | **状态**: active
- **源码位置**: `src/mcp/mod.rs:643`
- **用途**: 导出元数据枚举（节点类型 + 边类型）
- **输入**: 无
- **输出**: JSON，含 `node_kinds` 和 `edge_types` 数组
- **设计vs实现**: ⚠️ **偏差** — 描述说"15 种节点/11 种边"，实际返回 16 种节点/10 种边，边类型名称也不完全对应

---

## 9. codeloom_path_analysis

- **类型**: mcp | **状态**: active | **源码位置**: `679`
- **用途**: 两符号间路径搜索（调用/继承/数据流等）
- **输入**: `source/source_id` · `target/target_id` · `mode(shortest\|all)` · `max_paths` · `max_depth` · `direction` · `edge_filter`
- **输出**: JSON `{paths: [{edges: [...]}], total_found: N}`
- **设计vs实现**: ✅ match

---

## 10. codeloom_impact_analysis

- **类型**: mcp | **状态**: active | **源码位置**: `697`
- **用途**: 传递闭包 N 跳影响范围分析
- **输入**: `symbol/id` · `direction(forward\|reverse\|both)` · `radius(3)` · `edge_filter`
- **输出**: JSON `{symbol, radius, direction, affected: [...]}`
- **设计vs实现**: ✅ match

---

## 11. codeloom_neighbor_graph

- **类型**: mcp | **状态**: active | **源码位置**: `712`
- **用途**: 1 跳邻居，按边类型分组
- **输入**: `symbol/id` · `direction(forward\|reverse\|both)`
- **输出**: JSON `{symbol, depth:1, forward: {...}, backward: {...}}`
- **特性**: class/struct 跳过成员，暴露成员引用的外部符号
- **设计vs实现**: ✅ match

---

## 12. codeloom_inheritance_tree

- **类型**: mcp | **状态**: active | **源码位置**: `730`
- **用途**: 类的完整继承树（含虚方法覆写）
- **输入**: `symbol/id` · `direction(up\|down\|both)` · `max_depth(5)`
- **输出**: JSON 嵌套树
- **设计vs实现**: ✅ match

---

## 13. codeloom_status (DISABLED)

- **类型**: mcp | **状态**: disabled | **源码位置**: `268`
- **用途**: 查看索引状态（符号数/边数/文档数/DB 大小）
- **状态说明**: `status()` 函数已完整实现，但 MCP dispatch 被注释；handle_tool_call 中整段不可达

---

## 14. codeloom_get_doc (DISABLED)

- **类型**: mcp | **状态**: disabled | **源码位置**: `845`
- **用途**: 获取文档节点完整内容及嵌入图片
- **输入**: `doc_id` · `repo` · `branch`
- **状态说明**: `get_doc()` 函数已完整实现（含 base64 图片），但 MCP dispatch 被注释
- **注意**: 只查 `node_type='section'`，不返回 sym/chunk

---

## 15. codeloom_query_excel (DISABLED)

- **类型**: mcp | **状态**: disabled | **源码位置**: `920`
- **用途**: Excel 表格结构化查询
- **输入**: `doc_id` · `mode(row\|column\|filter\|auto)` · `filter/search` · `limit(20)`
- **状态说明**: `query_excel()` 函数已完整实现（四模式+比较运算符+文本搜索），但 MCP dispatch 被注释
