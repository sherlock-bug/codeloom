# CodeLoom Bug Inventory（基于规格测试发现）

来源：spec 对齐断言测试 `tests/run-spec-assertions.py`（108 个检查，90 通过，18 失败）
测试库：`tests/fixtures/expert-designed/`
日期：2026-05-14（新发现 6 类边类型提取断裂）

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

### BUG-010: Indexer 边类型提取断裂（6 类边未产生）
- **影响范围**: neighbor_graph / call_graph / path_analysis 等依赖边的所有工具
- **通用根因**: 索引器在提取函数体/类结构/文件关系时未创建以下边类型

| # | 边类型 | 测试用例 | 预期产生场景 | fixture 数据 |
|---|--------|---------|-------------|-------------|
| a | **calls** | HybridLogger::log 方法体 | 调用 FileLogger::log 和 ConsoleLogger::log | HybridLogger.cc:27-28 |
| b | **overrides** | ConsoleLogger::log | 覆写基类 Logger::log | .h:34 `override` 关键字 |
| c | **instantiates** | DataStore<int> | 显式模板实例化 | .cc:129 `template class DataStore<int>;` |
| d | **references** | initialize_logging | 引用全局变量 g_default_logger | .cc:62 `g_default_logger = logger;` |
| e | **contains** | LogLevel | 枚举包含枚举值 | .h:8-13 enum { LOG_DEBUG, LOG_INFO, ... } |
| f | **includes** | expert_fixture.cc | #include "expert_fixture.h" | .cc:1 `#include "expert_fixture.h"` |

- **影响工具**:
  - neighbor_graph: HybridLogger::log 无 calls 边、LogLevel 无 contains 边、initialize_logging 无 references 边等
  - inheritance_tree: 无 overrides 边
  - path_analysis: 部分路径因缺边不可达
  - call_graph: 部分调用链缺失
- **根因推测**: Clang AST 解析后，这些边类型的提取分支缺失或 filter 过滤过度

---

## P1 — 缺规格功能

### BUG-004: search 结果缺 class 增强字段 methods
- **工具**: codeloom_search (BM25)
- **输入**: 搜索到 class 符号
- **预期**: 每个 class 结果包含 `methods` 字段
- **规格**: search-enrichment §1.12
- **实际**: 无 `methods` 字段

### BUG-005: search 结果缺 class 增强字段 members
- **工具**: codeloom_search (BM25)
- **输入**: 搜索到 class/struct 符号
- **预期**: 每个 class/struct 结果包含 `members` 字段
- **规格**: search-enrichment §1.12
- **实际**: 无 `members` 字段

### BUG-008: inheritance_tree 顶层用 `root` 而非 `symbol`
- **工具**: codeloom_inheritance_tree
- **输入**: 所有查询
- **预期**: 统一使用 `symbol` 键名
- **实际**: 顶层返回 `root` 键，子节点用 `symbol` 键。不一致

---

## P2 — 索引数据质量

### BUG-011: 跨 TU 类符号重复 + 文件/行号错误（leveldb Compaction）
- **工具**: codeloom_list_symbols / codeloom_inspect
- **测试库**: `leveldb`（外部仓库，保留在索引中）
- **输入**: 搜索 `Compaction`（`class Compaction` 定义于 `db/version_set.h:319`）
- **预期**: 1 个结果，`file_path` = `...version_set.h`, `line_start` = 319
- **实际**: 
  - 2 个结果（id=1229, 2100）
  - id=1229: `file=db_impl.h:93` ❌（实际为 `CompactionStats` 所在行）
  - id=2100: `file=corruption_test.cc:363` ❌（实际为 `ASSERT` 语句）
  - 两套子符号（methods/fields）全部指向各自的错误文件/行号
  - 正确的 `version_set.h:319` 无对应条目
- **根因推测**: Clang AST 中来自 `#include` 的 CXXRecordDecl 节点 `loc.file` 解析异常，`cur_file` 追踪在跨文件上下文中的回退逻辑出错；不同翻译单元对同一类的处理未在 upsert_symbol 阶段去重

### BUG-011: inspect 枚举缺失 values 字段
- **工具**: codeloom_inspect
- **输入**: `{name: "LogLevel"}`
- **预期**: 返回 values 数组含 LOG_INFO, LOG_DEBUG 等
- **规格**: inspect-enrichment 要求 enum 节点返回 values 列表
- **实际**: 无 values 字段

### BUG-012: inspect 类 methods 字段内容不全
- **工具**: codeloom_inspect
- **输入**: `{name: "HybridLogger"}`
- **预期**: methods 含 "log"（类定义的方法名）
- **实际**: methods 字段存在但不含 "log"

### BUG-013: call_graph 终端标注不区分类型
- **工具**: codeloom_call_graph
- **输入**: `initialize_logging callees`
- **预期**: 终端节点标注区分 uses:（枚举值）、references:（全局变量）、string_literals:
- **规格**: enhanced-call-graph/spec.md
- **实际**: 全部标为 (calls:X)

### BUG-014: path_analysis 输出格式不符规格
- **工具**: codeloom_path_analysis
- **输入**: `source: initialize_logging, target: g_default_logger`
- **预期**: 路径为字符串链 "A → calls → B → uses → C"
- **规格**: path-analysis/spec.md
- **实际**: 路径数据非字符串链格式

### BUG-015: path_analysis 空路径处理不符规格
- **工具**: codeloom_path_analysis
- **输入**: 不存在的符号对
- **预期**: 返回 `{paths: [], total_found: 0}`
- **实际**: 当前返回格式不匹配

---

## ⚠️ 规格冲突（非实现 bug）

### SPEC-CONFLICT-001: schema-metadata 说 9 种边，extended-edge-types 说 11 种
- **schema-metadata**: calls, calls_override, inherits, contains, uses, references, returns, param_type, field_type
- **extended-edge-types**: calls, overrides, inherits, contains, uses, param_type, return_type, includes, aliases, field_type
- **差异**: 命名不同（calls_override vs overrides, returns vs return_type）、边集不同
- **现状**: 已确认 calls_override 合并为 overrides，schema-metadata 已同步更新

### SPEC-CONFLICT-002: schema-metadata 声明与实际输出的节点类型名不同
- **description 声明**: namespace, template_instance, typedef
- **实际输出**: variable, template_class, template_struct
- **共计**: 都是 15 种，但类型名不同

---

## 规格变更（不是 bug——用户确认）

### SPEC-CHANGE-001: search 结果不返回 methods/members 增强字段
- **状态**: ✅ 规格更新
- **来源**: BUG-004/005
- **待办**: 更新 search-enrichment/spec.md 移除对 methods/members 的要求

### FIXED-001: neighbor_graph 继承方向反了
- **状态**: ✅ 已修复
- **修复**: src/query/graph.rs — reverse 列序修正

### FIXED-002: .h 声明 + .cc 定义未合并为一个符号
- **状态**: ✅ 已修复
- **修复**: src/indexer/clang/mod.rs — 跨文件回退查询合并

### FIXED-003: 自由函数 inspect 返回 list（原 BUG-003）
- **状态**: ✅ 已修复（归因于 FIXED-002 的跨文件合并）

---

### BUG-016: 系统符号泄漏——CXXMethodDecl 非项目节点路径为空
- **工具**: codeloom_list_symbols / codeloom_inspect
- **测试库**: `leveldb`（外部仓库，保留在索引中）
- **现象**: 索引中混入 1,032 个空 `file_path` 符号（362 method + 321 function + 192 template_instance + 157 template_function），以及 327 个系统头文件符号
- **典型样例**: `atomic<bool>` (template_instance, file_path="")
- **根因**: `clang_filter.py` 的 is_system() 过滤逻辑在 hasBody=True 注入时放行了 CXXMethodDef 系统节点，这些节点在 `ast.rs` 中因 `decl_file` 为空且无 `includedFrom`，产生 file_path="" 的脏符号
- **影响**: 搜索/查询会返回无位置信息的脏数据
- **优先级**: P2

### BUG-017: 方法 line_end 小于 line_start
- **工具**: codeloom_inspect
- **测试库**: `leveldb` 中 `Reader::ReadRecord`（`/mnt/d/code/leveldb/db/log_reader.cc:56`）
- **现象**: inspect 显示 `line_end=55`，但 `line_start=56`（end < start，实际 end 应为函数体结束行）
- **根因推测**: `get_range()` 的行号解析对某些 Clang range 格式返回了错误值。可能跟 range 中 `.begin` 和 `.end` 的 `file` 字段缺失有关（类似 loc.file 为空的问题），导致 fallback 到 `loc.line` 或 offset→line 时取了错误行号
- **影响**: 影响方法定义的范围显示（不阻塞功能，但导航精度下降）
- **优先级**: P3
- **状态**: 2026-05-16 确认，已知但暂不处理

## 发现但未修复

### BUG-009: .h 声明 + .cc 定义 → FIXED-002
- **状态**: ✅ 已修复

### BUG-003: 自由函数 inspect 返回 list → FIXED-003
- **状态**: ✅ 已修复（随跨文件合并修复而解决）

---

## 统计

| 状态 | 数量 | 说明 |
|------|------|------|
|| P0 — 数据错误 | 9 | BUG-002 + BUG-010(a~g) + 旧 BUG-001 已修 |
|| P1 — 功能缺失 | 5 | BUG-008, BUG-011~015 |
|| P2 — 数据质量 | 1 | BUG-016 |
|| P3 — 小问题 | 1 | BUG-017 |
|| 已修复 | 3 | FIXED-001~003 |
| 规格变更 | 1 | SPEC-CHANGE-001 |
| 规格冲突 | 2 | SPEC-CONFLICT-001~002 |
| **待修复合计** | **15** | **9 P0 + 5 P1 + 1 P2** |
