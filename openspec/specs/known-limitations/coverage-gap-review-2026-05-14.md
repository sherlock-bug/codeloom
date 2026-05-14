# CodeLoom MCP 工具测试覆盖缺口审查报告

> 审查日期：2026-05-14
> 审查方法：Phase 0 平行专家审查（spec-only + test-only，不读 src/ 实现代码）
> 参与专家：3 路（MCP 工具规格专家 / 索引器+搜索规格专家 / 测试覆盖审计专家）

---

## 审查总览

| 维度 | 数值 |
|------|------|
| 审查的 spec 文件 | 21 个（specs/ 下 14 个 + archive 2 个 + overview/edge/node 5 个） |
| 审查的 MCP 工具 | 12 个 |
| 审查的测试断言 | 75 个（run-spec-assertions.py） |
| 已记录的已知缺口 | 14 项（known-limitations/spec.md） |
| **本次新发现缺口** | 🔴 高 9 项 / 🟡 中 12 项 / 🟢 低 5 项 |

---

## 逐工具审查

### 1. codeloom_schema

**规格契约**：无参数调用，返回 15 种节点类型 + 11 种边类型。每节点含 name/description/example，每边含 prefix/direction/source_kinds/target_kinds。

**测试覆盖**（12 断言）：
| 断言内容 | 备注 |
|---------|------|
| 无参调用成功，node_kinds 存在 | ✅ |
| node_kinds 每个条目含 name, description | ✅ 浅层 |
| node_kinds count >= 13 | ✅ |
| edge_types 存在，每个条目含 prefix, direction, source_kinds, target_kinds | ✅ 浅层 |
| edge_types count >= 10 | ✅ |

**缺口**：

| 严重度 | 缺口 | 状态 |
|--------|------|------|
| 🔴 | 15 种节点未逐类型断言（仅 >=13） | 📋 已知 #8 |
| 🔴 | 11 种边未逐类型断言（仅 >=10） | 📋 已知 #8 |
| 🟢 | node_kinds 各条目的 `example` 字段内容未验证 | 🆕 |
| 🟢 | edge_types 各条目的 `direction`/`source_kinds`/`target_kinds` 内容未验证 | 🆕 |

---

### 2. codeloom_list_repos / codeloom_list_branches

**规格契约**：list_repos 无参数返回所有已索引仓库名；list_branches 需 repo 参数返回分支列表。

**测试覆盖**（各 1 断言）：仅验证输出包含 "expert-test" / "main"。

**缺口**：

| 严重度 | 缺口 | 状态 |
|--------|------|------|
| 🟡 | 多 repo 场景未测试 | 🆕 |
| 🟡 | list_repos 空环境（无索引仓库）未测试 | 🆕 |
| 🟡 | list_branches 多分支场景未测试 | 🆕 |

---

### 3. codeloom_list_symbols

**规格契约**：LIKE 模式模糊匹配符号名，覆盖 #include 头文件。返回 name/kind/file_path/line。C++ 方法用 ClassName::methodName 格式。无 enrichment。

**测试覆盖**（6 断言）：Logger 相关类名匹配 4 项，AdvancedLogger（typedef）1 项，DataStore（模板）1 项。

**缺口**：

| 严重度 | 缺口 | 状态 |
|--------|------|------|
| 🟡 | 无匹配 pattern 的负向测试缺失 | 🆕 |
| 🟡 | limit 参数效果未明确验证 | 🆕 |
| 🟡 | branch 参数未变化测试 | 🆕 |
| 🟡 | ClassName::methodName 格式输出未验证 | 🆕 |

---

### 4. codeloom_search (BM25)

**规格契约**：双通道搜索（名称权重 0.7，注释权重 0.3），噪音过滤（z-score < 1.0 过滤），支持 kind 类型过滤。返回结果按节点类型附 enrichment：class→members+methods，enum→values，function→parent_class，section→prev/next，chunk→parent+prev/next，file→sections。按 mcp-full-attribute-json 规则按 hit_type 裁剪字段。

**测试覆盖**（4 断言）：HybridLogger 精确匹配排名、LogLevel 枚举 values 增强、空结果。

**缺口**：

| 严重度 | 缺口 | 状态 |
|--------|------|------|
| 🔴 | enrichment 仅测了 enum→values，未测 class→members+methods、function→parent_class、section、chunk、file | 📋 SPEC-CHANGE-001 |
| 🔴 | kind 参数从未使用 | 🆕 |
| 🔴 | 名称/注释双通道权重（0.7/0.3）排名未验证 | 🆕 |
| 🔴 | 噪音过滤（z-score < 1.0）未触发验证 | 🆕 |
| 🟢 | mcp-full-attribute-json 按 hit_type 字段裁剪合规性未验证 | 🆕 |

---

### 5. codeloom_semantic_search

**规格契约**：自然语言向量搜索，只搜索符号（不搜文档/注释），无 kind 过滤。输出增强规则与 search 完全一致（search-enrichment spec 要求一致性）。

**测试覆盖**（1 断言）：仅验证不崩溃（lambda 恒 True）。

**缺口**：

| 严重度 | 缺口 | 状态 |
|--------|------|------|
| 🔴 | **几乎零覆盖** — 无结果正确性验证、无 enrichment 验证、无相关性判断 | 🆕 |
| 🔴 | "只搜索符号不搜索文档/注释"的行为未验证 | 🆕 |
| 🔴 | "无 kind 过滤"的行为未验证（传 kind 应报错或被忽略） | 🆕 |
| 🟡 | 空查询 / 无匹配 场景未测试 | 🆕 |

---

### 6. codeloom_inspect

**规格契约**：按节点类型专有输出：class/struct→bases/members/methods/template_args，enum→values，file→sections，section→parent_section/child_sections/child_chunks/prev_section/next_section，chunk→parent_section/prev_chunk/next_chunk。其余类型回退到通用 edges 列表。支持 name 或 id（id 优先）。

**测试覆盖**（13 断言）：class/struct/enum/function 四种类型的基本字段验证（kind、file、line、values、bases、methods），跨文件合并（BUG-009）。

**缺口**：

| 严重度 | 缺口 | 状态 |
|--------|------|------|
| 🔴 | id 参数从未使用 | 📋 已知 #9 |
| 🔴 | template_args 字段未测试 | 📋 已知 #2 |
| 🔴 | file/section/chunk 节点类型完全无覆盖 | 🆕 |
| 🔴 | 通用回退（function/method 的 edges 分组列表输出）未验证 | 🆕 |
| 🔴 | bases/members/methods 仅验证键存在，未验证内容正确性 | 🆕 |
| 🟡 | enum values 内容验证仅 2/4 个值（LOG_WARN/LOG_ERROR 未断言） | 🆕 |

---

### 7. codeloom_inheritance_tree

**规格契约**：嵌套树格式，up 方向用 `inherits` 链，down 方向用 `children` 数组。顶层键为 `symbol`（非 `root`）。每个子类含 `overrides` 列表。direction up/down/both，max_depth=5。

**测试覆盖**（14 断言）：覆盖 3 方向 + 3 种节点角色（根类/中间类/叶子类）。

**缺口**：

| 严重度 | 缺口 | 状态 |
|--------|------|------|
| 🔴 | 顶层键名应为 `symbol`，当前使用 `root` | 📋 BUG-008 |
| 🔴 | `overrides` 字段未验证 | 📋 已知 #3 |
| 🟡 | max_depth 参数未测试 | 🆕 |
| 🟡 | 非 class 符号输入未测试（错误路径） | 🆕 |

---

### 8. codeloom_neighbor_graph

**规格契约**：按方向+边类型分组返回邻居。direction forward/reverse/both（默认 both），depth=1（固定）。class/struct 跳过成员，暴露成员引用的外部符号。覆盖所有 11 种边类型。

**测试覆盖**（12 断言）：覆盖 inherits（正/反向）、calls、uses、field_type 四种边类型，forward/reverse 两种方向。

**缺口**：

| 严重度 | 缺口 | 状态 |
|--------|------|------|
| 🔴 | **6 种边类型未覆盖**：contains、returns/return_type、param_type、aliases、includes、instantiates（现有 fixture 已具备产生这些边的代码结构） | 📋 已知 #5 部分 |
| 🔴 | class/struct 跳过成员、暴露外部符号的行为未验证 | 🆕 |
| 🟡 | depth 参数（固定 1）未验证 | 🆕 |
| 🟡 | id 参数未使用 | 📋 已知 #9 |
| 🟡 | 孤立符号（无邻居）的返回未测试 | 🆕 |

---

### 9. codeloom_call_graph

**规格契约**：文本格式层级化调用树。direction callers/callees。终端节点标注区分 uses（枚举值）、references（全局/静态变量）、string_literals。max_depth 控制递归深度。

**测试覆盖**（6 断言）：仅 CLI 调用，验证输出含 uses:/references:/string_literals: 标注。

**缺口**：

| 严重度 | 缺口 | 状态 |
|--------|------|------|
| 🔴 | 仅 CLI 测试，无 MCP JSON-RPC 协议层测试 | 📋 已知 #4 |
| 🔴 | 终端节点标注均不达标（统一标为 calls:X） | 📋 已知（call_graph 标注不达标） |
| 🔴 | max_depth 参数未测试 | 🆕 |
| 🟡 | calls_override/overrides 边类型未验证 | 🆕 |
| 🟡 | 深度递归场景（> 3 层调用链）未测试 | 🆕 |

---

### 10. codeloom_impact_analysis

**规格契约**：传递闭包 N 跳影响分析。返回 affected 数组，每项含 symbol/distance/via（边类型）。direction forward/reverse/both（默认 reverse），radius 默认 3。支持 edge_filter。

**测试覆盖**（1 断言）：仅验证 affected 键存在。

**缺口**：

| 严重度 | 缺口 | 状态 |
|--------|------|------|
| 🔴 | distance/via 字段内容完全未验证 | 🆕 |
| 🔴 | forward 方向未测试 | 📋 已知 #6 |
| 🔴 | both 方向未测试 | 📋 已知 #6 |
| 🔴 | radius 参数未测试（仅用默认 3） | 📋 已知 #6 |
| 🔴 | edge_filter 参数从未使用 | 📋 已知 #6 + 🆕（补充） |
| 🟡 | 仅 1 种边类型（inherits）被验证，10 种未覆盖 | 🆕 |

---

### 11. codeloom_path_analysis

**规格契约**：符号间路径搜索。mode shortest/all，direction forward/reverse/both（默认 both），max_depth=10，max_paths=20，edge_filter 可选。返回 paths 数组 + total_found。

**测试覆盖**（4 断言）：shortest 模式基本路径存在性，路径含 `→` 分隔符，空路径返回空数组。

**缺口**：

| 严重度 | 缺口 | 状态 |
|--------|------|------|
| 🔴 | mode=all 全路径模式未测试 | 📋 已知 #7 |
| 🔴 | 输出格式不匹配规格（当前非字符串链格式） | 📋 已知（path_analysis 输出格式不匹配） |
| 🟡 | direction 参数未变化（仅用默认 both） | 🆕 |
| 🟡 | max_depth / max_paths 参数未测试 | 🆕 |
| 🟡 | edge_filter 参数从未使用 | 🆕 |
| 🟡 | 多跳路径（> 2 跳）的内容格式未验证 | 🆕 |

---

## 函数体解析边类型专项分析

用户特别关注 calls/uses/references 在 neighbor_graph/call_graph/path_analysis 中的覆盖：

| 边类型 | neighbor_graph | call_graph | path_analysis | 评价 |
|--------|:---:|:---:|:---:|------|
| calls | ✅ 有断言（forward calls 非空） | ✅ 文本验证 | ⚠️ 仅验证路径存在 | call_graph 标注格式不达标（标注为 calls:X 而非区分类型） |
| uses | ✅ 有断言（uses 分组非空） | ⚠️ 文本验证 uses: | ⚠️ 路径中边类型未精确验证 | call_graph uses 标注仍不达标 |
| references | ❌ 未单独断言 references 分组 | ⚠️ 文本验证 references: | ❌ 未测试 references 边路径 | neighbor_graph 的 references 边未显式断言 |
| string_literals | ❌ 未测试 | ⚠️ 文本验证 | ❌ 未测试 | fixture 有 `g_app_name` 等字面量但未覆盖 |

---

## 跨工具系统性缺口

这些缺口影响多个工具，按类型归纳：

### 参数覆盖缺口

| 参数 | 受影响工具 | 状态 |
|------|-----------|------|
| `id`（节点 ID） | inspect, neighbor_graph, inheritance_tree, call_graph, impact_analysis, path_analysis | 📋 已知 #9 |
| `kind`（类型过滤） | search | 🆕 |
| `edge_filter`（边类型数组） | path_analysis, impact_analysis | 🆕 |
| `max_depth`/`radius` | call_graph, path_analysis, inheritance_tree | 🆕 |

### 错误路径覆盖

| 错误场景 | 状态 |
|---------|------|
| 不存在的 repo/branch/symbol | 📋 已知 #10 |
| 不支持的 kind 值 | 🆕 |
| 空参数 / 超限参数 | 🆕 |

### 边类型覆盖全景

规格定义 11 种边类型。测试断言覆盖情况：

| 边类型 | 测试覆盖 | 备注 |
|--------|:---:|------|
| calls | ✅ | neighbor_graph / call_graph / path_analysis |
| inherits | ✅ | neighbor_graph / inheritance_tree |
| overrides/calls_override | ⚠️ | inheritance_tree 间接涉及，未直接断言 |
| contains | ❌ | inspect 通过 methods 字段间接依赖，但未断言 contains 边本身 |
| uses | ⚠️ | neighbor_graph validates，call_graph 文本检查，但格式不达标 |
| references | ⚠️ | call_graph 文本检查，但格式不达标 |
| param_type | ❌ | 完全未覆盖 |
| return_type/returns | ❌ | 完全未覆盖 |
| includes | ❌ | 完全未覆盖 |
| aliases | ❌ | 完全未覆盖 |
| instantiates | ❌ | 完全未覆盖 |
| field_type | ✅ | neighbor_graph BaseConfig→LogLevel |
| template_use | ❌ | 完全未覆盖 |

### Fixture 符号利用率

Fixture 定义了 36 个符号，测试引用了 18 个（50%）。以下符号具备测试价值但未被覆盖：

| 未引用符号 | 可测的边类型 |
|-----------|------------|
| `MAX_BUFFER`（macro） | —（macro 基本覆盖） |
| `LOG_WARN`/`LOG_ERROR`（enum_value） | uses 边 |
| `Logger::instance_count`（static_var） | references 边 |
| `g_app_name`/`g_max_msg_len`（global） | references 边 |
| `SimpleLogger`（using alias） | aliases 边 |
| `Pair<T>`（template struct） | instantiates 边 |
| `max_of<T>`（template function） | template_use 边 |
| `cleanup_logging`/`get_status_message`（free func） | calls/returns/param_type |
| `FileLogger::log`/`ConsoleLogger::log`（method override） | overrides 边 |

---

## 本次新发现缺口汇总

对比 known-limitations/spec.md 中已记录的 14 项缺口，本次新增：

### 🔴 高严重度（9 项）

| # | 缺口 | 受影响的工具 |
|---|------|------------|
| N1 | search kind 参数从未测试 | codeloom_search |
| N2 | search 名称/注释双通道权重（0.7/0.3）排名未验证 | codeloom_search |
| N3 | search 噪音过滤（z-score < 1.0）未触发验证 | codeloom_search |
| N4 | semantic_search 几乎零覆盖（仅 1 个不崩溃断言） | codeloom_semantic_search |
| N5 | inspect file/section/chunk 节点类型无测试覆盖 | codeloom_inspect |
| N6 | inspect fallback 通用 edges 输出未验证 | codeloom_inspect |
| N7 | inspect bases/members/methods 字段仅验证键存在、不验证内容 | codeloom_inspect |
| N8 | neighbor_graph class/struct 跳过成员暴露外部符号行为未验证 | codeloom_neighbor_graph |
| N9 | call_graph max_depth 参数未测试 | codeloom_call_graph |

### 🟡 中严重度（12 项）

| # | 缺口 | 受影响的工具 |
|---|------|------------|
| N10 | semantic_search "只搜索符号"+"无 kind 过滤"行为未验证 | codeloom_semantic_search |
| N11 | inspect enum values 仅测 2/4 个值 | codeloom_inspect |
| N12 | neighbor_graph depth 参数未验证 | codeloom_neighbor_graph |
| N13 | call_graph calls_override 边未验证 | codeloom_call_graph |
| N14 | impact_analysis distance/via 字段内容未验证 | codeloom_impact_analysis |
| N15 | impact_analysis 仅 1/11 边类型被验证 | codeloom_impact_analysis |
| N16 | path_analysis direction 参数未变化测试 | codeloom_path_analysis |
| N17 | path_analysis max_depth/max_paths 参数未测试 | codeloom_path_analysis |
| N18 | path_analysis edge_filter 参数从未使用 | codeloom_path_analysis |
| N19 | inheritance_tree max_depth 参数未测试 | codeloom_inheritance_tree |
| N20 | list_symbols 负向测试（无匹配）+ limit 效果未验证 | codeloom_list_symbols |
| N21 | 全局错误路径零覆盖（bad repo/branch/symbol/kind/limit） | 全工具 |

### 🟢 低严重度（5 项）

| # | 缺口 | 受影响的工具 |
|---|------|------------|
| N22 | schema node_kinds example 字段内容未验证 | codeloom_schema |
| N23 | schema edge_types direction/source_kinds/target_kinds 内容未验证 | codeloom_schema |
| N24 | search mcp-full-attribute-json 按 hit_type 字段裁剪合规性未验证 | codeloom_search |
| N25 | list_repos 多 repo 场景未测试 | codeloom_list_repos |
| N26 | list_branches 多 branch 场景未测试 | codeloom_list_branches |

---

## 与上次审查的关联

上次审查报告了 48 个缺口（22 高、20 中、6 低），已知记录到 known-limitations/spec.md 的约 14 项。本次审查：

- **确认**了已知的 14 项缺口仍然存在
- **新增**了 9 个高严重度 + 12 个中严重度 + 5 个低严重度缺口
- **重点关注了**函数体解析边类型（calls/uses/references）覆盖——发现 neighbor_graph 的 references 边未独立断言、call_graph 的标注格式问题已记录但未修复、path_analysis 边类型内容未精确验证
- **发现了** semantic_search 的几乎全面覆盖空白和 inspect file/section/chunk 节点类型的覆盖盲区

---

## 建议优先级

| 优先级 | 缺口编号 | 理由 |
|--------|---------|------|
| P0 | N4（semantic_search 零覆盖） | 工具完全不受测试保护 |
| P1 | N1-N3（search 参数覆盖） | search 是高频使用工具 |
| P1 | N8（neighbor_graph 跳过成员行为） | 规格明确声明的行为差异 |
| P2 | N5-N7（inspect 节点类型覆盖） | inspect 是基础查看工具 |
| P2 | N14-N15（impact_analysis 字段内容） | 1 个断言无法保证正确性 |
| P3 | N10-N21（中严重度） | 参数组合和边界测试 |
| P4 | N22-N26（低严重度） | 元数据和辅助工具 |
