# CodeLoom MCP 端到端手工测试用例文档

> **测试目标**：通过 JSON-RPC stdio 验证全部 12 个 MCP 工具的正确性
> **测试二进制**：`/mnt/d/RagMcpHermes/codeloom/target/release/codeloom`
> **测试仓库**：leveldb (C++) @ branch=main1 (3359 symbols, 26 docs)
> **调用方式**：`echo 'JSON-RPC-REQUEST' | ./codeloom mcp`
> **版本**：v1.0 — 2026-05-12

---

## 目录

1. [基本信息查询工具](#1-基本信息查询工具)
   - MCP-01: codeloom_list_repos — 正向
   - MCP-02: codeloom_list_branches — 正向
   - MCP-03: codeloom_schema — 正向
2. [符号搜索工具](#2-符号搜索工具)
   - MCP-04: codeloom_list_symbols — 正向（pattern 搜索）
   - MCP-05: codeloom_list_symbols — 边界（空 pattern）
   - MCP-06: codeloom_search — 正向（class 搜索 + enrichment 验证）
   - MCP-07: codeloom_search — 正向（enum 搜索 + enrichment 验证）
   - MCP-08: codeloom_search — 正向（kind 过滤 + limit 截断）
   - MCP-09: codeloom_search — 边界（空结果）
   - MCP-10: codeloom_search — 异常（无效 kind）
   - MCP-11: codeloom_semantic_search — 正向
3. [符号详情工具](#3-符号详情工具)
   - MCP-12: codeloom_inspect — 类（class）检查
   - MCP-13: codeloom_inspect — 方法（method）检查
   - MCP-14: codeloom_inspect — 结构体（struct）检查
4. [调用关系工具](#4-调用关系工具)
   - MCP-15: codeloom_get_call_graph — callers
   - MCP-16: codeloom_get_call_graph — callees + max_depth
5. [图分析工具](#5-图分析工具)
   - MCP-17: codeloom_path_analysis — 最短路径
   - MCP-18: codeloom_path_analysis — 全路径 + edge_filter
   - MCP-19: codeloom_impact_analysis — forward
   - MCP-20: codeloom_impact_analysis — reverse + edge_filter
   - MCP-21: codeloom_neighbor_graph — both
   - MCP-22: codeloom_inheritance_tree — down
   - MCP-23: codeloom_inheritance_tree — up（空结果）
6. [异常场景](#6-异常场景)
   - MCP-24: 缺参错误（缺 branch）
   - MCP-25: 缺参错误（缺 repo）
   - MCP-26: 不存在仓库
   - MCP-27: 不存在分支（未索引）
7. [端到端流程](#7-端到端流程)
   - MCP-28: E2E — search → inspect → get_call_graph → neighbor_graph

---

## 1. 基本信息查询工具

### MCP-01: codeloom_list_repos — 正向

| 字段 | 值 |
|------|-----|
| **编号** | MCP-01 |
| **名称** | `codeloom_list_repos` 列出所有已索引仓库 |
| **前置条件** | 已至少索引 leveldb 仓库 |
| **JSON-RPC 请求** | `echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_list_repos","arguments":{}}}' \| /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null` |
| **预期结果** | `result.content[0].text` 包含字符串 `"leveldb"` |
| **验证方法** | 检查 JSON 响应：`result.content[0].text` 为纯文本，以换行符分隔各仓库名；`id` 回显为 1；`jsonrpc` 为 `"2.0"`；无 `error` 字段 |

---

### MCP-02: codeloom_list_branches — 正向

| 字段 | 值 |
|------|-----|
| **编号** | MCP-02 |
| **名称** | `codeloom_list_branches` 列出指定仓库分支 |
| **前置条件** | leveldb 仓库已索引，分支包含 `__builtin__` 和 `main1` |
| **JSON-RPC 请求** | `echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"codeloom_list_branches","arguments":{"repo":"leveldb"}}}' \| /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null` |
| **预期结果** | `result.content[0].text` 包含 `"__builtin__"` 和 `"main1"` |
| **验证方法** | 检查输出包含仓库名前缀和两个分支名；JSON-RPC `id` 回显为 2 |

---

### MCP-03: codeloom_schema — 正向

| 字段 | 值 |
|------|-----|
| **编号** | MCP-03 |
| **名称** | `codeloom_schema` 导出节点类型和边类型枚举 |
| **前置条件** | 无（无参数工具） |
| **JSON-RPC 请求** | `echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"codeloom_schema","arguments":{}}}' \| /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null` |
| **预期结果** | JSON 响应，包含 `node_kinds`（≥15 种，含 function/method/class/struct/enum/enum_value/field/global/static_var/macro/template_function/template_class/template_struct/variable/string_literal）和 `edge_types`（≥10 种，含 calls/calls_override/inherits/contains/uses/references/returns/param_type/field_type/template_use） |
| **验证方法** | 解析 JSON 确认 `node_kinds` 数组长度 ≥15；`edge_types` 数组长度 ≥10；每种节点/边类型有 `name`/`description`/`example` 字段 |

---

## 2. 符号搜索工具

### MCP-04: codeloom_list_symbols — 正向（pattern 搜索）

| 字段 | 值 |
|------|-----|
| **编号** | MCP-04 |
| **名称** | `codeloom_list_symbols` 按 pattern 模糊搜索符号 |
| **前置条件** | leveldb@main1 已索引 |
| **JSON-RPC 请求** | `echo '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"codeloom_list_symbols","arguments":{"pattern":"compaction","repo":"leveldb","branch":"main1"}}}' \| /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null` |
| **预期结果** | 纯文本列表，包含多个匹配 `"compaction"` 的符号；结果包含 `Compaction` 类、`Compaction::level` 方法等；每行格式：`id=N [type] name @ file:line`；结果数量 > 5 |
| **验证方法** | 检查响应包含 `[class] Compaction`、`[method] Compaction::level`、`[field] Compaction::level_` 等；id 为正整数 |

---

### MCP-05: codeloom_list_symbols — 边界（空 pattern / limit）

| 字段 | 值 |
|------|-----|
| **编号** | MCP-05 |
| **名称** | `codeloom_list_symbols` 空 pattern 和 limit 参数 |
| **前置条件** | leveldb@main1 已索引 |
| **JSON-RPC 请求** | `echo '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"codeloom_list_symbols","arguments":{"pattern":"","repo":"leveldb","branch":"main1","limit":5}}}' \| /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null` |
| **预期结果** | 返回 `"(none)"` 或空列表（空 pattern 不匹配任何符号） |
| **验证方法** | 检查输出含 `"(none)"` 或 `Symbols matching ''` 提示 |

---

### MCP-06: codeloom_search — 正向（class 搜索 + enrichment 验证）

| 字段 | 值 |
|------|-----|
| **编号** | MCP-06 |
| **名称** | `codeloom_search` 搜索 class 符号，验证 enrichment 字段 |
| **前置条件** | leveldb@main1 已索引 |
| **JSON-RPC 请求** | `echo '{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"codeloom_search","arguments":{"query":"DBImpl","repo":"leveldb","branch":"main1"}}}' \| /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null` |
| **预期结果** | JSON 响应，`count` > 0；`results` 数组中的 method 类型结果包含 `parent_class` 字段（如 `"parent_class":"DBImpl"`）、`signature` 字段；结果包含 `kind`/`file`/`line`/`name`/`score`/`snippet`/`type` 等字段；class 类型结果有 `parent_class`（空字符串）或 `snippet` |
| **验证方法** | 解析 JSON：验证 `count` = 10；验证 `DBImpl::Put` 的 `parent_class` = `"DBImpl"` 且 `signature` = `"Status (const WriteOptions &, const Slice &)"`；验证 `DBImpl::BackgroundCompaction` 的 `signature` = `"void ()"` |

---

### MCP-07: codeloom_search — 正向（enum 搜索 + enrichment 验证）

| 字段 | 值 |
|------|-----|
| **编号** | MCP-07 |
| **名称** | `codeloom_search` 搜索 enum 类型，验证 enum_value enrichment |
| **前置条件** | leveldb@main1 已索引 |
| **JSON-RPC 请求** | `echo '{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"codeloom_search","arguments":{"query":"Code","repo":"leveldb","branch":"main1"}}}' \| /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null` |
| **预期结果** | JSON 响应，`count` > 0；结果包含 `Code`（enum）、`Code::kOk`、`Code::kNotFound` 等 enum_value |
| **验证方法** | 验证 results 包含 `Code` 项且其 `kind` = `"enum"`；验证有 `Code::kOk` 等 enum_value 结果 |

---

### MCP-08: codeloom_search — 正向（kind 过滤 + limit 截断）

| 字段 | 值 |
|------|-----|
| **编号** | MCP-08 |
| **名称** | `codeloom_search` 使用 kind 过滤 + limit 限制 |
| **前置条件** | leveldb@main1 已索引 |
| **JSON-RPC 请求** | `echo '{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"codeloom_search","arguments":{"query":"Options","repo":"leveldb","branch":"main1","kind":"struct","limit":3}}}' \| /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null` |
| **预期结果** | JSON 响应，`count` = 3（受 limit 限制，实际匹配 ≥5），所有结果 `kind` = `"struct"` |
| **验证方法** | 解析 JSON：验证 `results` 数组长度 ≤ 3；验证每条结果的 `kind` 均为 `"struct"`；验证 `file` 路径包含 `leveldb` |

---

### MCP-09: codeloom_search — 边界（空结果）

| 字段 | 值 |
|------|-----|
| **编号** | MCP-09 |
| **名称** | `codeloom_search` 搜索不存在的符号返回空结果 |
| **前置条件** | leveldb@main1 已索引 |
| **JSON-RPC 请求** | `echo '{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"codeloom_search","arguments":{"query":"NonExistentSymbolXYZ123","repo":"leveldb","branch":"main1"}}}' \| /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null` |
| **预期结果** | JSON 响应，`count` = 0，`results` = `[]`；无 `error` 字段 |
| **验证方法** | 验证 `result.content[0].text` 中 JSON 的 `count` = 0，`results` 为空数组 |

---

### MCP-10: codeloom_search — 异常（无效 kind）

| 字段 | 值 |
|------|-----|
| **编号** | MCP-10 |
| **名称** | `codeloom_search` 传入无效 kind 返回错误 |
| **前置条件** | leveldb@main1 已索引 |
| **JSON-RPC 请求** | `echo '{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"codeloom_search","arguments":{"query":"Version","repo":"leveldb","branch":"main1","kind":"invalid_kind"}}}' \| /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null` |
| **预期结果** | 返回 `error` 字段，`code` = -32602（Invalid params），`message` 包含 `"Invalid kind 'invalid_kind'"` 并列出合法取值 |
| **验证方法** | 验证响应包含 `"error"` 而非 `"result"`；验证 `error.code` = -32602；验证 `error.message` 包含 `invalid_kind` 和所有合法 kind 值 |

---

### MCP-11: codeloom_semantic_search — 正向

| 字段 | 值 |
|------|-----|
| **编号** | MCP-11 |
| **名称** | `codeloom_semantic_search` 语义搜索 |
| **前置条件** | leveldb@main1 已索引；向量模型已加载 |
| **JSON-RPC 请求** | `echo '{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"codeloom_semantic_search","arguments":{"query":"compaction and version management","repo":"leveldb","branch":"main1","limit":5}}}' \| /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null` |
| **预期结果** | JSON 响应，`count` > 0，结果包含 `name`/`kind`/`file`/`line`/`score`/`type`/`parent_class`（若方法）/`signature` 等字段；已知 bug：结果中缺失 `id` 字段，`type` 写死为 `"code"` |
| **验证方法** | 解析 JSON：验证 `count` > 0；验证结果包含 `VersionSet::PickCompaction`、`Version::compaction_level_`、`ManualCompaction` 等语义相关符号；确认 `type` 字段为 `"code"` |

---

## 3. 符号详情工具

### MCP-12: codeloom_inspect — 类（class）检查

| 字段 | 值 |
|------|-----|
| **编号** | MCP-12 |
| **名称** | `codeloom_inspect` 检查类（验证 bases/members/methods 等） |
| **前置条件** | leveldb@main1 已索引 |
| **JSON-RPC 请求** | `echo '{"jsonrpc":"2.0","id":12,"method":"tools/call","params":{"name":"codeloom_inspect","arguments":{"name":"Compaction","repo":"leveldb","branch":"main1"}}}' \| /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null` |
| **预期结果** | JSON 响应，包含 `name`=`"Compaction"`、`kind`=`"class"`、`namespace`=`"leveldb"`、`file`、`line_start`、`line_end`、`documentation` 字段 |
| **验证方法** | 验证 JSON 的 `name` = `"Compaction"`，`kind` = `"class"`，`namespace` = `"leveldb"`，`file` 包含 `"version_set.h"` |

---

### MCP-13: codeloom_inspect — 方法（method）检查

| 字段 | 值 |
|------|-----|
| **编号** | MCP-13 |
| **名称** | `codeloom_inspect` 检查方法 |
| **前置条件** | leveldb@main1 已索引 |
| **JSON-RPC 请求** | `echo '{"jsonrpc":"2.0","id":13,"method":"tools/call","params":{"name":"codeloom_inspect","arguments":{"name":"DBImpl::Put","repo":"leveldb","branch":"main1"}}}' \| /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null` |
| **预期结果** | JSON 响应，包含 `name`=`"DBImpl::Put"`、`kind`=`"method"`、`namespace`=`"leveldb"`、`file` 等字段；可能含 `edges` 对象描述调用/参数/返回边 |
| **验证方法** | 验证 `name` = `"DBImpl::Put"`，`kind` = `"method"`，`file` 包含 `"db_impl.h"` |

---

### MCP-14: codeloom_inspect — 结构体（struct）检查

| 字段 | 值 |
|------|-----|
| **编号** | MCP-14 |
| **名称** | `codeloom_inspect` 检查结构体 |
| **前置条件** | leveldb@main1 已索引 |
| **JSON-RPC 请求** | `echo '{"jsonrpc":"2.0","id":14,"method":"tools/call","params":{"name":"codeloom_inspect","arguments":{"name":"Options","repo":"leveldb","branch":"main1"}}}' \| /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null` |
| **预期结果** | JSON 响应（注意：当有多个同名符号时返回数组），数组第一个元素包含 `name`=`"Options"`、`kind`=`"struct"`、`namespace`=`"leveldb"`、`file`、`line_start`、`line_end`、`documentation` 等 |
| **验证方法** | 验证返回为数组；验证数组长度 ≥1；验证第一个元素 `name` = `"Options"`，`kind` = `"struct"`，`file` 包含 `"options.h"` |

---

## 4. 调用关系工具

### MCP-15: codeloom_get_call_graph — callers

| 字段 | 值 |
|------|-----|
| **编号** | MCP-15 |
| **名称** | `codeloom_get_call_graph` 查询 callers |
| **前置条件** | leveldb@main1 已索引 |
| **JSON-RPC 请求** | `echo '{"jsonrpc":"2.0","id":15,"method":"tools/call","params":{"name":"codeloom_get_call_graph","arguments":{"name":"DBImpl::Put","repo":"leveldb","branch":"main1","direction":"callers"}}}' \| /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null` |
| **预期结果** | 纯文本输出，包含 `"Call graph for 'DBImpl::Put' (callers):"` 头部；列出 `DBImpl::Put (id=953)` 及其调用者 |
| **验证方法** | 检查输出包含 `"callers"` 和 `"DBImpl::Put"`；验证格式为树状缩进文本 |

---

### MCP-16: codeloom_get_call_graph — callees + max_depth

| 字段 | 值 |
|------|-----|
| **编号** | MCP-16 |
| **名称** | `codeloom_get_call_graph` 查询 callees 带 max_depth |
| **前置条件** | leveldb@main1 已索引 |
| **JSON-RPC 请求** | `echo '{"jsonrpc":"2.0","id":16,"method":"tools/call","params":{"name":"codeloom_get_call_graph","arguments":{"name":"DBImpl::BackgroundCompaction","repo":"leveldb","branch":"main1","direction":"callees","max_depth":2}}}' \| /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null` |
| **预期结果** | 纯文本输出，包含 `"Call graph for 'DBImpl::BackgroundCompaction' (callees):"`；列出被调用函数 |
| **验证方法** | 检查输出包含 `"callees"` 和 `"DBImpl::BackgroundCompaction"`；确认树深度不超过 2 层 |

---

## 5. 图分析工具

### MCP-17: codeloom_path_analysis — 最短路径

| 字段 | 值 |
|------|-----|
| **编号** | MCP-17 |
| **名称** | `codeloom_path_analysis` 找两点间最短路径 |
| **前置条件** | leveldb@main1 已索引 |
| **JSON-RPC 请求** | `echo '{"jsonrpc":"2.0","id":17,"method":"tools/call","params":{"name":"codeloom_path_analysis","arguments":{"source":"DBImpl","target":"Options","repo":"leveldb","branch":"main1","mode":"shortest"}}}' \| /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null` |
| **预期结果** | JSON 响应，`paths` 数组长度 ≥1，每条路径由 edges 数组组成；第一跳从 `DBImpl` 到 `DBImpl::options_` 通过 `contains` 边，第二跳从 `DBImpl::options_` 到 `Options` 通过 `uses_type` 边；`total_found` ≥ 1 |
| **验证方法** | 解析 JSON：验证 `paths[0].edges[0]` 包含 `"DBImpl"` 和 `"contains"` 和 `"DBImpl::options_"`；验证 `paths[0].edges[1]` 包含 `"uses_type"` 和 `"Options"` |

---

### MCP-18: codeloom_path_analysis — 全路径 + edge_filter

| 字段 | 值 |
|------|-----|
| **编号** | MCP-18 |
| **名称** | `codeloom_path_analysis` 全路径模式 + edge_filter 过滤 |
| **前置条件** | leveldb@main1 已索引 |
| **JSON-RPC 请求** | `echo '{"jsonrpc":"2.0","id":18,"method":"tools/call","params":{"name":"codeloom_path_analysis","arguments":{"source":"DBImpl","target":"Options","repo":"leveldb","branch":"main1","mode":"all","edge_filter":["uses_type","contains"]}}}' \| /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null` |
| **预期结果** | JSON 响应，`total_found` > 0（可能达 20+ 条路径因 `max_paths` 默认上限）；所有路径的边类型仅包含 `uses_type` 和 `contains`；路径模式为 `DBImpl → contains → DBImpl::xxx → uses_type → Options` |
| **验证方法** | 验证 `total_found` > 0；验证路径中不包含 `calls`/`inherits` 等未指定的边类型 |

---

### MCP-19: codeloom_impact_analysis — forward

| 字段 | 值 |
|------|-----|
| **编号** | MCP-19 |
| **名称** | `codeloom_impact_analysis` forward 方向分析影响范围 |
| **前置条件** | leveldb@main1 已索引 |
| **JSON-RPC 请求** | `echo '{"jsonrpc":"2.0","id":19,"method":"tools/call","params":{"name":"codeloom_impact_analysis","arguments":{"symbol":"Version","repo":"leveldb","branch":"main1","direction":"forward","radius":1}}}' \| /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null` |
| **预期结果** | JSON 响应，`symbol`=`"Version"`，`direction`=`"forward"`，`radius`=1；`affected` 数组包含 Version 类的所有方法和字段；每项有 `distance`=1、`symbol`（如 `"Version::AddIterators"`）、`via`（如 `"contains:Version::AddIterators"`） |
| **验证方法** | 解析 JSON：验证 `affected` 数组长度 > 20（Version 类含大量成员）；验证至少有 `"Version::Get"`、`"Version::Ref"`、`"Version::files_"`、`"Version::compaction_score_"` |

---

### MCP-20: codeloom_impact_analysis — reverse + edge_filter

| 字段 | 值 |
|------|-----|
| **编号** | MCP-20 |
| **名称** | `codeloom_impact_analysis` reverse 方向 + edge_filter |
| **前置条件** | leveldb@main1 已索引 |
| **JSON-RPC 请求** | `echo '{"jsonrpc":"2.0","id":20,"method":"tools/call","params":{"name":"codeloom_impact_analysis","arguments":{"symbol":"Version","repo":"leveldb","branch":"main1","direction":"reverse","radius":2,"edge_filter":["param_type","return_type","uses_type"]}}}' \| /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null` |
| **预期结果** | JSON 响应，`direction`=`"reverse"`；影响结果仅包含通过 `param_type`/`return_type`/`uses_type` 到达的符号（不含 contains/inherits 等） |
| **验证方法** | 验证 `affected` 数组中所有项的 `via` 字段的前缀仅为 `param_type`/`return_type`/`uses_type` 之一 |

---

### MCP-21: codeloom_neighbor_graph — both

| 字段 | 值 |
|------|-----|
| **编号** | MCP-21 |
| **名称** | `codeloom_neighbor_graph` 查看符号的直接邻居 |
| **前置条件** | leveldb@main1 已索引 |
| **JSON-RPC 请求** | `echo '{"jsonrpc":"2.0","id":21,"method":"tools/call","params":{"name":"codeloom_neighbor_graph","arguments":{"symbol":"Version","repo":"leveldb","branch":"main1","direction":"both"}}}' \| /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null` |
| **预期结果** | JSON 响应，`depth`=1（默认），`direction`=`"both"`，`symbol`=`"Version"`；`backward` 部分包含 `param_type`/`return_type`/`uses_type` 等反向引用；`forward` 部分包含 `contains`（列出所有方法和字段） |
| **验证方法** | 解析 JSON：验证 `forward.contains` 包含 `"Version::AddIterators"`、`"Version::Get"`、`"Version::Ref"`、`"Version::files_"` 等；验证 `backward.param_type` 存在且非空 |

---

### MCP-22: codeloom_inheritance_tree — down

| 字段 | 值 |
|------|-----|
| **编号** | MCP-22 |
| **名称** | `codeloom_inheritance_tree` 查看类的子类继承树 |
| **前置条件** | leveldb@main1 已索引 |
| **JSON-RPC 请求** | `echo '{"jsonrpc":"2.0","id":22,"method":"tools/call","params":{"name":"codeloom_inheritance_tree","arguments":{"symbol":"DBImpl","repo":"leveldb","branch":"main1","direction":"down","max_depth":3}}}' \| /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null` |
| **预期结果** | JSON 响应，`root`=`"DBImpl"`；`children` 数组可能为空（若无子类）或包含子类名 |
| **验证方法** | 验证 `root` = `"DBImpl"`；检查 JSON 结构完整性 |

---

### MCP-23: codeloom_inheritance_tree — up（空结果验证）

| 字段 | 值 |
|------|-----|
| **编号** | MCP-23 |
| **名称** | `codeloom_inheritance_tree` 查看类的父类（空结果） |
| **前置条件** | leveldb@main1 已索引 |
| **JSON-RPC 请求** | `echo '{"jsonrpc":"2.0","id":23,"method":"tools/call","params":{"name":"codeloom_inheritance_tree","arguments":{"symbol":"Slice","repo":"leveldb","branch":"main1","direction":"up","max_depth":3}}}' \| /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null` |
| **预期结果** | JSON 响应，`root`=`"Slice"`；`children` 为空数组 `[]`（Slice 无父类/子类） |
| **验证方法** | 验证 `root` = `"Slice"`；验证 `children` 为 `[]`；无 error 字段 |

---

## 6. 异常场景

### MCP-24: 缺参错误（缺 branch）

| 字段 | 值 |
|------|-----|
| **编号** | MCP-24 |
| **名称** | `codeloom_search` 缺少必填参数 `branch` |
| **前置条件** | leveldb@main1 已索引 |
| **JSON-RPC 请求** | `echo '{"jsonrpc":"2.0","id":24,"method":"tools/call","params":{"name":"codeloom_search","arguments":{"query":"Version","repo":"leveldb"}}}' \| /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null` |
| **预期结果** | 返回 `error` 字段，`code` = -32602，`message` 包含 `"branch is required"` |
| **验证方法** | 验证响应包含 `"error"`；验证 `error.message` 含 `"branch"` |

---

### MCP-25: 缺参错误（缺 repo）

| 字段 | 值 |
|------|-----|
| **编号** | MCP-25 |
| **名称** | `codeloom_get_call_graph` 缺少必填参数 `repo` |
| **前置条件** | leveldb@main1 已索引 |
| **JSON-RPC 请求** | `echo '{"jsonrpc":"2.0","id":25,"method":"tools/call","params":{"name":"codeloom_get_call_graph","arguments":{"name":"DBImpl::Put"}}}' \| /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null` |
| **预期结果** | 返回 `error` 字段，`code` = -32602，`message` 包含 `"未找到仓库"` 或 `"repo is required"` |
| **验证方法** | 验证响应包含 `"error"`；验证 `error.message` 含 `"仓库"` 或 `"repo"` |

---

### MCP-26: 不存在仓库

| 字段 | 值 |
|------|-----|
| **编号** | MCP-26 |
| **名称** | `codeloom_list_symbols` 指定不存在的仓库 |
| **前置条件** | leveldb@main1 已索引 |
| **JSON-RPC 请求** | `echo '{"jsonrpc":"2.0","id":26,"method":"tools/call","params":{"name":"codeloom_list_symbols","arguments":{"pattern":"main","repo":"nonexistent_repo","branch":"main1"}}}' \| /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null` |
| **预期结果** | 返回 `error`，`code` = -32602，`message` 包含 `"未找到仓库 'nonexistent_repo'"` 并列出可用仓库 `leveldb` |
| **验证方法** | 验证 `error.message` 含 `"nonexistent_repo"` 和 `"leveldb"` |

---

### MCP-27: 不存在分支

| 字段 | 值 |
|------|-----|
| **编号** | MCP-27 |
| **名称** | `codeloom_search` 指定不存在的分支 |
| **前置条件** | leveldb@main1 已索引 |
| **JSON-RPC 请求** | `echo '{"jsonrpc":"2.0","id":27,"method":"tools/call","params":{"name":"codeloom_search","arguments":{"query":"Version","repo":"leveldb","branch":"nonexistent_branch"}}}' \| /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null` |
| **预期结果** | 返回 `error` 或空结果（取决于服务端实现）；若报错则 `code` = -32602 |
| **验证方法** | 检查响应结构，确保无异常崩溃（如非 JSON 输出） |

---

## 7. 端到端流程

### MCP-28: E2E 端到端流程 — search → inspect → get_call_graph → neighbor_graph

| 字段 | 值 |
|------|-----|
| **编号** | MCP-28 |
| **名称** | 端到端流程：搜索符号 → 检查详情 → 调用图 → 邻居图 |
| **前置条件** | leveldb@main1 已索引 |
| **步骤** | 见下方子步骤 |

#### 步骤 1: codeloom_search — 搜索 `DBImpl` class

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_search","arguments":{"query":"DBImpl","repo":"leveldb","branch":"main1","kind":"class"}}}' | /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null
```

**验证**：`count` ≥ 1；`results[0].name` = `"DBImpl"`，`kind` = `"class"`；记录 `id` 供后续使用

#### 步骤 2: codeloom_inspect — 检查 `DBImpl` 类详情

```bash
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"codeloom_inspect","arguments":{"name":"DBImpl","repo":"leveldb","branch":"main1"}}}' | /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null
```

**验证**：`name` = `"DBImpl"`，`kind` = `"class"`，`namespace` = `"leveldb"`，`file` 包含 `"db_impl.h"`

#### 步骤 3: codeloom_get_call_graph — 查 `DBImpl::Put` 的调用者

```bash
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"codeloom_get_call_graph","arguments":{"name":"DBImpl::Put","repo":"leveldb","branch":"main1","direction":"callers"}}}' | /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null
```

**验证**：输出包含 `"Call graph for 'DBImpl::Put' (callers)"`；包含 `"DBImpl::Put (id=953)"`

#### 步骤 4: codeloom_neighbor_graph — 查 `DBImpl::Put` 的邻居

```bash
echo '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"codeloom_neighbor_graph","arguments":{"symbol":"DBImpl::Put","repo":"leveldb","branch":"main1","direction":"both"}}}' | /mnt/d/RagMcpHermes/codeloom/target/release/codeloom mcp 2>/dev/null
```

**验证**：`symbol` = `"DBImpl::Put"`；`forward` 部分应包含 `param_type`（Slice, WriteOptions）、`return_type`（Status）

#### 整体验证标准

| 检查项 | 期望 |
|--------|------|
| 所有 4 个子步骤均返回成功（无 error 字段） | ✓ |
| JSON-RPC id 按 1-4 递增回显 | ✓ |
| 数据一致性：步骤 1 中的 class 名 `DBImpl` 可在步骤 2 中 inspect | ✓ |
| 步骤 3 的方法名 `DBImpl::Put` 可由步骤 2 的 inspect 结果推断 | ✓ |
| 步骤 4 的 forward.param_type 包含 `Slice` — 与 leveldb 代码一致 | ✓ |
| 所有响应均为有效 JSON | ✓ |
| 所有响应含 `jsonrpc: "2.0"` | ✓ |

---

## 附录

### A. 测试数据速查

| 符号名 | kind | 文件 | 说明 |
|--------|------|------|------|
| `DBImpl` | class | db/db_impl.h:29 | leveldb 核心数据库实现类 |
| `Compaction` | class | db/version_set.h:33 | 压缩操作类 |
| `Version` | class | db/version_set.h:38 | 版本管理类 |
| `Options` | struct | include/leveldb/options.h:34 | 数据库选项结构体 |
| `Slice` | class | include/leveldb/slice.h:27 | 字符串视图类 |
| `CompressionType` | enum | include/leveldb/options.h:25 | 压缩类型枚举 |
| `CompressionType::kNoCompression` | enum_value | options.h:28 | 枚举值 |
| `CompressionType::kSnappyCompression` | enum_value | options.h:29 | 枚举值 |
| `DBImpl::Put` | method | db_impl.h:39 | 写操作 |
| `DBImpl::BackgroundCompaction` | method | db_impl.h:143 | 后台压缩 |
| `DBImpl::options_` | field | db_impl.h | Options 成员字段 |
| `VersionSet::PickCompaction` | method | version_set.h:234 | 选择压缩 |
| `VersionSet::NeedsCompaction` | method | version_set.h:252 | 是否需要压缩 |

### B. 预期 JSON-RPC 错误码

| 错误码 | 含义 | 典型场景 |
|--------|------|----------|
| -32602 | Invalid params | 缺必填参数、无效 kind 值、不存在的仓库 |
| -32603 | Internal error | 服务端内部错误（需关注日志） |

### C. 已知问题

1. **codeloom_semantic_search**: 返回结果缺乏 `id` 字段；`type` 写死为 `"code"`（代码类型）而非区分代码/文档
2. **codeloom_inspect 同名符号**: 当仓库中存在多个同名符号（如 `Options` 出现在 4 个文件中），inspect 返回数组而非单个对象
3. **codeloom_get_call_graph**: 当前工具返回的调用者/被调用者信息较为有限（仅显示单层自身）；需结合 `codeloom_neighbor_graph` 或 `codeloom_impact_analysis` 获取更丰富的边信息

### D. 测试通过准则

- **正向用例**：响应包含 `"result"` 而非 `"error"`；结果内容符合预期结构及数据
- **异常用例**：响应包含 `"error"`，`error.code` 符合预期，`error.message` 包含关键提示信息
- **边界用例**：响应不崩溃；空结果返回 `count=0`/`results=[]` 而非错误
- **端到端用例**：所有 4 个子步骤均通过；步骤间数据可衔接一致
