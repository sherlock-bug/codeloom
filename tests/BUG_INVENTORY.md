# CodeLoom Bug Inventory（基于规格测试发现）

来源：spec 对齐断言测试 `tests/run-spec-assertions.py`（54 个检查，53 通过，1 失败）
测试库：`tests/fixtures/expert-designed/`
日期：2026-05-14（FIXED-002: 跨文件声明定义合并）

---

## P0 — 数据错误（影响多个工具）

### BUG-001: neighbor_graph 继承方向反了（inherits 反向返回自环）
- **工具**: codeloom_neighbor_graph
- **输入**: `{symbol: "Logger", direction: "reverse"}`
- **预期**: backward.inherits 包含 FileLogger, ConsoleLogger
- **实际**: backward.inherits 返回 [Logger, Logger]（源符号自身重复）
- **根因**: inherits 边的 reverse 查询中 source_id/target_id 搞反了，导致返回自身
- **影响范围**: 所有类/结构的反向邻居中的 inherits 边

### BUG-002: 枚举值无反查询能力（LOG_INFO 反向邻居空）
- **工具**: codeloom_neighbor_graph
- **输入**: `{symbol: "LOG_INFO", direction: "reverse"}`
- **预期**: 返回使用了 LOG_INFO 的 function/method（如 initialize_logging, report 等）
- **实际**: backward 为空
- **根因**: uses 边未正确建立，或 neighbor_graph 对 enum_value 的查询有 bug

### BUG-003: 自由函数 inspect 返回 list 而非 dict
- **工具**: codeloom_inspect
- **输入**: `{name: "initialize_logging", repo: "expert-test", branch: "main"}`
- **预期**: 返回单个节点的 dict（含 kind, name, file 等）
- **实际**: 返回 list（多个搜索结果？）
- **根因**: inspect 对函数名可能返回多个匹配（重载？），但没有正确处理
- **影响范围**: 所有非独一名称的符号查询

---

## P1 — 缺规格功能（search-enrichment 未实现）

### BUG-004: search 结果缺 class 增强字段 methods
- **工具**: codeloom_search (BM25)
- **输入**: 搜索到 class 符号
- **预期**: 每个 class 结果包含 `methods` 字段（逗号分隔的方法名列表）
- **规格**: search-enrichment §1.12
- **实际**: 无 `methods` 字段

### BUG-005: search 结果缺 class 增强字段 members
- **工具**: codeloom_search (BM25)
- **输入**: 搜索到 class/struct 符号
- **预期**: 每个 class/struct 结果包含 `members` 字段（逗号分隔的字段名列表）
- **规格**: search-enrichment §1.12
- **实际**: 无 `members` 字段

---

## P2 — 搜索召回问题

### BUG-006: list_symbols "Logger" 漏掉 FileLogger/ConsoleLogger/HybridLogger
- **工具**: codeloom_list_symbols
- **输入**: `{pattern: "Logger", limit: 20}`
- **预期**: 所有名称含 "Logger" 的符号（Logger, FileLogger, ConsoleLogger, HybridLogger, 及其成员）
- **实际**: 只返回 Logger, Logger::log, Logger::instance_count 等，缺失 FileLogger, ConsoleLogger, HybridLogger 及其成员
- **根因**: list_symbols 使用 SQL LIKE 模式，可能对大小写或子串匹配有限制

### BUG-007: search "Logger" 召回率低于 list_symbols（子问题）
- **工具**: codeloom_search (BM25)
- **输入**: `{query: "Logger"}`
- **预期**: 返回所有含 "Logger" 的符号
- **实际**: 仅返回 3 个（Logger, Logger::log, Logger::operator=），远少于 list_symbols
- **注**: 这个和 BUG-006 可能同根因，也可能是 BM25 索引的问题

---

## P3 — 继承树输出格式

### BUG-008: inheritance_tree 顶层用 `root` 而非 `symbol`
- **工具**: codeloom_inheritance_tree
- **输入**: 所有查询
- **预期**: 统一使用 `symbol` 键名（子节点用 `symbol`，顶层也应一致）
- **实际**: 顶层返回 `root` 键，子节点用 `symbol` 键。不一致
- **严重度**: 低，但不一致的命名容易被 LLM 误解

---

## ⚠️ 规格冲突（非实现 bug）

### SPEC-CONFLICT-001: schema-metadata 说 10 种边，extended-edge-types 说 11 种
- **schema-metadata**: calls, calls_override, inherits, contains, uses, references, returns, param_type, field_type, template_use
- **extended-edge-types**: calls, overrides, inherits, contains, uses, param_type, return_type, includes, aliases, field_type, template_use
- **差异**: 命名不同（calls_override vs overrides, returns vs return_type）、边集不同（有 references 无 includes/aliases, 或有 includes/aliases 无 references）
- **建议**: 统一两份规格，确定标准 11 种边

### SPEC-CONFLICT-002: schema-metadata 声明与实际输出的节点类型名不同
- **description 声明**: namespace, template_instance, typedef
- **实际输出**: variable, template_class, template_struct
- **共计**: 都是 15 种，但类型名不同

---

## 规格变更（不是 bug——用户确认）

### SPEC-CHANGE-001: search 结果不返回 methods/members 增强字段
- **状态**: ✅ 规格更新（search 不应返回过多数据）
- **来源**: BUG-004/005
- **待办**: 更新 search-enrichment/spec.md 移除对 methods/members 的要求

### FIXED-001: neighbor_graph 继承方向反了（inherits 反向返回自环）
- **状态**: ✅ 已修复
- **修复**: src/query/graph.rs line 65 — reverse 方向 SELECT 列序 `s1.name, s2.name` 改为 `s2.name, s1.name`（s2=source, s1=target，列序需对应 source_name→s2, target_name→s1）
- **验证**: run-spec-assertions.py 中 Logger backward 含 FileLogger/ConsoleLogger 3 个测试全部通过

### FIXED-002: .h 声明 + .cc 定义未合并为一个符号
- **状态**: ✅ 已修复
- **修复**: src/indexer/clang/mod.rs upsert_symbol — None 分支增加跨文件回退查询（name+ns+kind+sig 匹配，忽略 file_path），找到则收敛到已有节点，新符号是定义且已有节点是声明时更新 is_definition=1。双向处理（不管谁先到达都合并）。
- **验证**: 
  - `inspect initialize_logging` 返回单个 dict 而非 list
  - file 指向 .h（声明文件）
  - run-spec-assertions.py BUG-009 测试通过（53/54，仅剩 BUG-002）

---

## 发现但未修复

### BUG-009: .h 声明 + .cc 定义未合并为一个符号 → FIXED-002
- **状态**: ✅ 已修复→FIXED-002
- **工具**: 索引器（src/indexer/clang/mod.rs upsert_symbol）
- **问题**: extended-node-types/spec.md §1.12 要求 "声明合并后 is_definition"——头文件声明+实现文件定义应合并为同一符号（更新位置、is_definition=1）
- **实际**: 合并键包含 file_path（line 186: `AND file_path=?5`），.h 和 .cc 路径不同 → 创建两个独立节点
- **影响**: 
  - `inspect initialize_logging` 返回数组而非单个对象（两个同名节点）
  - 可能导出 list_symbols/search 中的重复结果
- **修复思路**: 去掉合并键中的 file_path，或增加一步 (name, namespace, kind, signature) 去重合并

---

## 统计

| 状态 | 数量 |
|------|------|
| P0 — 数据错误 | 2（BUG-002~003） |
| P1 — 功能缺失 | 1（BUG-008） |
| 已修复 | 2（FIXED-001, FIXED-002） |
| 规格变更 | 1（SPEC-CHANGE-001） |
| 正常 | 2（BUG-006, BUG-007） |
| 规格冲突 | 2（SPEC-CONFLICT-001~002） |
| **合计** | **4 个待修复 + 2 个规格冲突** |
