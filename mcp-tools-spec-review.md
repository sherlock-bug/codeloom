# CodeLoom MCP 工具规格契约审查报告

审查日期：2026-05-14
审查范围：openspec/specs/ 下 14 个 spec 文件
审查原则：仅基于规格文档，不参考 src/ 实现代码

---

## 工具: codeloom_neighbor_graph

来源文件：openspec/specs/neighbor-graph/spec.md
涉及文件：openspec/specs/mcp-tools/spec.md（共享分支隔离规则）

### 参数规格
- `symbol` （必填，string） — 符号完整名称，如 `DBImpl`、`ClassName::methodName`
- `repo` （必填，string） — 仓库名
- `branch` （必填，string） — 分支名，通过 `resolve_branch_id()` 转为整数 ID 用于边过滤
- `direction` （可选，string，默认 `"both"`） — 方向控制
- `depth` （可选，integer，默认 1） — 跳数（spec 中称 depth，但描述为 1-2 跳邻域分析，具体实现可能另设）

### 输出格式规格
- 返回值结构：按方向 + 边类型分组的邻居列表
- 分组格式：
  - 若 direction=both，返回 `forward: {calls: [...], contains: [...], ...}` + `backward: {inherits: [...], called_by: [...], ...}`
  - 若 direction=forward，仅返回正向邻居
  - 若 direction=reverse，仅返回反向邻居（spec 场景中显示为 `backward: {used_by: [...]}`）
- 每个邻居条目包含目标符号名

### 支持的边类型/节点类型
- 规格承诺覆盖所有可用边类型，按边类型分组返回（calls、returns、param_type、inherits、uses、contains 等）
- 具体边类型列表参见 extended-edge-types/spec.md：calls、inherits、overrides、instantiates、param_type、return_type、includes、uses_type、contains、aliases、uses（共 11 种）

### 行为边界
- direction 合法值：`"forward"`、`"reverse"`、`"both"`（默认）
- depth 默认值：1（固定 depth=1 或可在 1-2 间？spec 提到 1-2 跳但默认和固定未明确，参阅 mcp-tool-descriptions 中说"固定 depth=1"）
- 对 class/struct 会跳过其成员，改为暴露成员引用的外部符号（来自 spec 补充描述）
- 分支隔离：查询 edges 表时包含 `AND (branch_id = ? OR branch_id = 0)` 过滤条件
- 使用整数 branch_id 而非字符串 branch_name 进行边过滤

---

## 工具: codeloom_get_call_graph

来源文件：openspec/specs/call-graph-module/spec.md
涉及文件：openspec/specs/mcp-tools/spec.md

### 参数规格
- `name` （必填，string） — 符号完整名称，如 `DBImpl::Get`、`DB::Put`
- `repo` （必填，string） — 仓库名
- `branch` （必填，string） — 分支名
- `direction` （必填，string） — 方向，无默认值
- `max_depth` （可选，integer，默认值未在 spec 中明确指定，但常见为 3） — 递归深度

### 输出格式规格
- 返回值：文本格式的层级化调用树
- JSON-RPC 返回格式与重构前保持一致（spec 要求的兼容性条件）
- 每层缩进表示调用深度

### 支持的边类型
- 调用关系：仅 `calls:` 和 `calls_override`（后改为 `overrides:`）边

### 行为边界
- direction 合法值：`"callers"`（查谁调用了目标）、`"callees"`（目标调用了谁）
- max_depth 默认值：未在 call-graph-module 中明确，后续 enhanced-call-graph 替代
- 注意：call-graph-module/spec.md 仍保留此工具，但 enhanced-call-graph/spec.md 声明"移除旧版 codeloom_get_call_graph 工具的独立声明，其功能被 enhanced-call-graph / neighbor_graph / path_analysis / impact_analysis 全覆盖"
- 分支隔离：同 mcp-tools 共享规则，使用 branch_id 过滤

---

## 工具: codeloom_path_analysis

来源文件：openspec/specs/path-analysis/spec.md
涉及文件：openspec/specs/mcp-tools/spec.md

### 参数规格
- `source` （必填，string） — 起始符号名
- `target` （必填，string） — 目标符号名
- `repo` （必填，string） — 仓库名
- `branch` （必填，string） — 分支名
- `mode` （可选，string，默认 `"shortest"`） — 路径搜索模式
- `direction` （可选，string，默认 `"both"`） — 边方向
- `max_depth` （可选，integer，默认 10） — 最大搜索深度
- `max_paths` （可选，integer，默认 20） — 最大返回路径数（仅全路径模式有效）
- `edge_filter` （可选，array of strings，默认 null） — 限定边类型数组，如 `["calls", "calls_override"]`

### 输出格式规格
- 返回值结构（有路径时）：路径数组，每条路径为符号→边类型→符号→... 的链
- 返回值结构（无路径时）：`{"paths": [], "total_found": 0}`
- 最短路径模式（shortest）：返回单条或多条同长度最短路径
- 全路径模式（all）：返回所有找到的路径，条数不超过 max_paths

### 支持的边类型
- 默认：所有可用边类型（不指定 edge_filter 时走全边）
- 限定模式：仅走 edge_filter 中指定的边类型数组
- 所有 11 种边类型均可用于路径搜索

### 行为边界
- direction 合法值：`"forward"`、`"reverse"`、`"both"`（默认）
- mode 合法值：`"shortest"`（默认）、`"all"`
- max_depth 默认值：10
- max_paths 默认值：20
- edge_filter 默认值：null（所有边类型）
- 环检测：BFS 扩展时检测重复节点路径并丢弃，避免无限循环
- 分支隔离：同 mcp-tools 共享规则

---

## 工具: codeloom_inheritance_tree

来源文件：openspec/specs/inheritance-tree/spec.md
涉及文件：openspec/specs/mcp-tools/spec.md、openspec/specs/extended-edge-types/spec.md

### 参数规格
- `symbol` （必填，string） — 类名
- `repo` （必填，string） — 仓库名
- `branch` （必填，string） — 分支名
- `direction` （可选，string，默认 `"both"`） — 方向：向上查祖先 / 向下查子孙 / 双向
- `max_depth` （可选，integer，默认 5） — 递归深度

### 输出格式规格
- 返回值结构：嵌套树，层层包含 `children` 或 `inherits` 数组
- down 方向返回格式：`{symbol: "DB", children: [{symbol: "DBImpl", children: [{symbol: "LevelDBImpl"}]}]}`
- up 方向返回格式：`{symbol: "LevelDBImpl", inherits: {symbol: "DBImpl", inherits: {symbol: "DB"}}}`
- 叶子节点：`{symbol: "LevelDBImpl", children: []}`
- 每个子类节点包含 `overrides` 字段列出覆写的方法名（含虚函数分发信息）

### 支持的边类型
- `inherits:` — 继承关系边
- `overrides:` — 虚函数覆写边（从 overrides 边获取虚方法分发列表）

### 行为边界
- direction 合法值：`"up"`（查父类/祖先）、`"down"`（查子类/子孙）、`"both"`（默认，同时返回祖先和子孙）
- max_depth 默认值：5
- 含虚拟方法分发：direction=down 时，每个子类包含 `overrides` 字段
- 分支隔离：`build_tree` 和 `get_overrides` 嵌套函数需传入 branch_id，而非硬编码为 0 回退值
- 继承树和虚函数覆写边查询按分支过滤，多分支独立

---

## 工具: codeloom_impact_analysis

来源文件：openspec/specs/impact-analysis/spec.md
涉及文件：openspec/specs/mcp-tools/spec.md

### 参数规格
- `symbol` （必填，string） — 符号名
- `repo` （必填，string） — 仓库名
- `branch` （必填，string） — 分支名
- `direction` （可选，string，默认 `"reverse"`） — 方向
- `radius` （可选，integer，默认 3） — 传递闭包的最大跳数

### 输出格式规格
- 返回值结构：数组或对象，列出每个受影响的符号及其 `distance`（距离）和 `via`（经过的边类型）
- 反向分析示例：`symbol=g_config direction=reverse radius=3`
  → `process_request`（distance=1 via references）、`main`（distance=2 via calls）
- 正向分析示例：`symbol=init direction=forward radius=1`
  → `g_config`（via references）、`db_open`（via calls）

### 支持的边类型
- 覆盖所有可用边类型（11 种：calls、inherits、overrides、instantiates、param_type、return_type、includes、uses_type、contains、aliases、uses）
- 支持 edge_filter 参数（在扩展描述中提及）可选限定边类型

### 行为边界
- direction 合法值：`"forward"`（分析影响哪些对象）、`"reverse"`（默认，分析谁依赖目标）、`"both"`（双向）
- radius 默认值：3，超半径限制时不再扩展
- 传递闭包：N 跳递归，区别于邻居图（仅 1 跳）
- 分支隔离：同 mcp-tools 共享规则

---

## 工具: codeloom_schema

来源文件：openspec/specs/schema-metadata/spec.md

### 参数规格
- 无参数（无需 repo、branch、name 等任何参数即可调用）
- 返回硬编码的稳定元数据

### 输出格式规格
- 返回值结构：`node_kinds` 数组 + `edge_types` 数组
- `node_kinds`：每项包含 `name`、`description`、`example`
- `edge_types`：每项包含 `prefix`、`direction`（from→to 的语义）、`source_kinds`、`target_kinds`

### 支持的节点类型（15 种，来自 extended-node-types/spec.md）
function、method、class、struct、enum、enum_value、global、static_var、variable、field、string_literal、macro、template_function、template_class、template_struct

### 支持的边类型（11 种，来自 extended-edge-types/spec.md）
calls、inherits、overrides、instantiates、param_type、return_type、includes、uses_type、contains、aliases、uses

### 特殊说明
- 此工具替代了已移除的 `codeloom_overview` 工具
- 不需要连接数据库或访问任何索引数据

---

## 工具: codeloom_search（BM25 精确搜索）

来源文件：openspec/specs/bm25-precise-search/spec.md
涉及文件：openspec/specs/search-enrichment/spec.md、openspec/specs/mcp-full-attribute-json/spec.md、openspec/specs/mcp-tool-descriptions/spec.md

### 参数规格
- `query` （必填，string） — 搜索关键词，可以是符号名、注释内容、中文描述
- `repo` （必填，string） — 仓库名
- `branch` （必填，string） — 分支名
- `kind` （可选，string） — 按符号类型过滤（如 function、class 等，用 codeloom_schema 看合法值）
- `limit` （可选，integer，默认 10） — 返回结果数量上限

### 输出格式规格
- 返回值：JSON 数组，每项包含节点完整属性
- 属性根据 hit_type（"code"/"doc"/"file"）静态裁剪：
  - **code 节点**：name、kind、file_path、line_start、line_end、signature、doc_comment、namespace、parent_class、language（不包含 title、section_path、content 等文档字段）
  - **doc 节点**：title、content、file_path、file_format、section_path、level、node_type（不包含 kind、signature、namespace 等符号字段）
  - **file 节点**：file_path、summary、repo（不包含 kind、signature、doc_comment 等符号字段）
- 类型增强字段（search-enrichment/spec.md）：
  - class/struct 结果附加 `members`（逗号分隔字段名）和 `methods`（逗号分隔方法名），不超过 15 项
  - enum 结果附加 `values`（枚举值名，逗号分隔）
  - function/method 结果附加 `parent_class`（所属类名）
  - section 结果附加 `prev_section`、`next_section`
  - chunk 结果附加 `parent_section`、`prev_chunk`、`next_chunk`
  - file 结果附加 `sections`（顶层 section 标题列表）
  - 无对应增强数据的字段在 JSON 中省略
- 搜索命中权重：名称通道（×0.7）优先于注释通道（×0.3）

### 行为边界
- 搜索范围：符号名称、注释、文档内容、文件信息，覆盖 #include 头文件
- 命名权重高于注释权重
- 纯 BM25 关键词匹配，不掺杂向量语义搜索
- 支持噪音过滤：使用预先标定的 BM25 噪音基线，z-score < 1.0 的结果被过滤
- query 可以是精确符号名（如 `DBImpl::CompactMemTable`）或中文关键词（如 `"压缩"`）
- 空结果返回空数组

---

## 工具: codeloom_semantic_search（向量语义搜索）

来源文件：openspec/specs/search-enrichment/spec.md（一致性要求提及）
涉及文件：openspec/specs/mcp-full-attribute-json/spec.md

### 参数规格
- `query` （必填，string） — 自然语言描述，如"处理用户登录的函数"
- `repo` （必填，string） — 仓库名
- `branch` （必填，string） — 分支名
- `limit` （可选，integer，默认 10） — 返回结果数量上限

### 输出格式规格
- 与 codeloom_search 应用**完全相同的类型增强规则**（search-enrichment/spec.md Requirement: 三种搜索工具一致增强）
- 输出 JSON 属性格式与 codeloom_search 一致（mcp-full-attribute-json 规则同样适用）
- class/struct → members + methods
- enum → values
- function/method → parent_class
- section → prev_section + next_section
- chunk → parent_section + prev_chunk + next_chunk
- file → sections
- 空字段省略

### 行为边界
- 只搜索符号节点，不搜索文档/注释
- 不支持 kind 过滤
- 需要向量模型已加载
- 适合自然语言描述查询，不适合精确符号名查找
- 语义搜索无 BM25 噪音基线过滤机制

---

## 工具: codeloom_inspect

来源文件：openspec/changes/archive/2026-05-12-inspect-specialization/specs/inspect-enrichment/spec.md（inspect 增强规格）

### 参数规格
- `name` （可选，string） — 符号完整名称
- `id` （可选，integer） — 符号节点 ID（优先于 name）
- `repo` （必填，string） — 仓库名
- `branch` （必填，string） — 分支名
- name 和 id 至少传一个，传 id 精度最高

### 输出格式规格
按节点类型输出专有信息（而非统一的通用 edges 列表）：

- **class/struct 节点**：
  - `bases`（基类名列表）
  - `members`（成员字段）
  - `methods`（成员方法）
  - `template_args`（模板参数，如有）

- **enum 节点**：
  - `values`（枚举值列表）

- **file 节点**：
  - `sections`（顶层 section 列表）

- **section 节点**：
  - `parent_section`
  - `child_sections`
  - `child_chunks`
  - `prev_section`
  - `next_section`

- **chunk 节点**：
  - `parent_section`
  - `prev_chunk`
  - `next_chunk`

- **通用回退**（function/method/namespace/global 等无专有类型的节点）：
  - 保持通用 edges 分组列表输出（当前未在 inspect-enrichment 中精确定义 edges 列表格式，从"当前"描述推断为方向+边类型分组的邻居边列表）

### 行为边界
- 分支隔离：同 mcp-tools 共享规则（继承自图分析工具的共同行为）
- name 和 id 二选一必传
- 此规格是 inspect 工具的增强版，使其从"通用 edges 列表"进化为"节点类型感知的定制化输出"

---

## 工具: codeloom_list_symbols（模糊搜索）

来源文件：openspec/specs/bm25-precise-search/spec.md（间接提及）、search-enrichment/spec.md（说明不适用类型增强）
涉及文件：openspec/specs/mcp-tool-descriptions/spec.md

### 参数规格
- `pattern` （必填，string） — 模糊名称模式，如 `login` 匹配 `handleLogin`、`loginUser` 等
- `repo` （必填，string） — 仓库名
- `branch` （必填，string） — 分支名
- `limit` （可选，integer，默认 20） — 返回结果数量上限

### 输出格式规格
- 返回值：结构化结果数组，每项包含名称、类型、文件路径、行号
- 纯文本输出，不适用 search-enrichment 的类型增强规则（search-enrichment/spec.md: "list_symbols 是纯文本输出，不适用"）

### 行为边界
- 基于 LIKE 模糊匹配搜索符号名称
- 搜索覆盖索引全部符号，包括 #include 的第三方头文件
- C++ 类方法使用 `ClassName::methodName` 格式
- 不搜索注释/文档内容，仅匹配符号名
- 分支隔离：同 mcp-tools 共享规则

---

## 工具: codeloom_list_repos

来源文件：openspec/specs/mcp-tools/spec.md（间接涉及）
未在用户指定的 14 个文件中独立定义。功能来源于 overview 中 I 层（基础设施与部署）的 list-repos 域。

### 参数规格
- 无参数

### 输出格式
- 返回所有已索引的仓库名列表

### 行为边界
- 在任何搜索/查询操作前必须先调用此工具获取可用的 repo 参数值
- 索引需要通过 CLI 执行：`codeloom index <path> --repo <name> --branch <branch>`

---

## 工具: codeloom_list_branches

来源文件：openspec/specs/mcp-tools/spec.md（间接涉及）
未在用户指定的 14 个文件中独立定义。功能来源于 overview 中 I 层的 list-branches 域。

### 参数规格
- `repo` （必填，string） — 仓库名

### 输出格式
- 返回指定仓库的所有已索引分支名列表

### 行为边界
- 用于确认分支状态，团队协作时使用

---

## 共享行为契约（所有图分析工具通用）

来源文件：openspec/specs/mcp-tools/spec.md

### 参数规格（共享）
- `repo` （必填） — 所有图分析工具都要求 repo + branch 作为必填参数
- `branch` （必填） — 分支名

### 分支隔离规则
- 所有 edges 表查询包含 `AND branch_id = ?` 过滤条件
- 向后兼容旧数据：实际条件为 `AND (branch_id = ? OR branch_id = 0)`
- 使用整数 branch_id（通过 `resolve_branch_id()` 函数将 branch 字符串转为整数 ID）
- 不得使用字符串拼接或 branch_name 进行边过滤

### 涉及的工具有
codeloom_neighbor_graph、codeloom_get_call_graph（或 enhanced 版）、codeloom_path_analysis、codeloom_inheritance_tree、codeloom_impact_analysis、codeloom_inspect

---

## 边类型参考（系统支持的所有边）

来源文件：openspec/specs/extended-edge-types/spec.md，共 11 种：

| 边类型前缀 | 语义 | 来源→目标 |
|-----------|------|----------|
| calls | 函数调用 | caller → callee |
| inherits | 类继承 | derived → base |
| overrides | 虚函数覆写 | override method → base virtual method |
| instantiates | 模板实例化 | template_instance → template |
| param_type | 参数类型 | function → parameter type |
| return_type | 返回类型 | function → return type |
| includes | #include | source file → header file |
| uses_type | 类型使用 | symbol → used type |
| contains | 包含关系 | class/struct → member method/field |
| aliases | 类型别名 | typedef → underlying type |
| uses | 使用关系（枚举值/全局变量/字符串字面量） | function → used symbol |

### 特殊边（通过 uses 前缀区分）

来源文件：openspec/specs/enum-value-usage/spec.md、global-variable-references/spec.md、string-literal-references/spec.md

- `uses:EnumName::Value` — 枚举值使用边：函数体中检测 `EnumName::Value` 形式引用
- `references:var_name` — 全局/静态变量引用边：函数体中检测对 global/static_var 符号的引用
- `uses:string_literal_content` — 字符串字面量使用边：函数体中检测字符串字面量引用

### 边创建规则（索引层行为，非查询层）
- 枚举值使用：仅当 `EnumName::Value` 指向已存在的 enum_value 符号时才创建边
- 全局变量引用：仅当引用的变量 kind=global 或 kind=static_var 时才创建边，本地变量和函数参数不创建
- 字符串字面量：仅当字面量已作为 string_literal 符号入库时才创建边，空字符串 `""` 不创建

---

## 工具描述规范（所有 MCP 工具通用）

来源文件：openspec/specs/mcp-tool-descriptions/spec.md

- 所有 description 应以"**首选工具**"或"**优先使用**"标记开头
- 明确引导 LLM 优先于 grep/rg 搜索代码
- 列出相对于 grep/文件读取的优势
- 包含至少一个具体的 query 示例（搜索类工具）
- query 可以是中文功能描述（如'用户认证'）或符号名（如'AuthService'）
- 不暴露 BM25、FTS5、RRF、vec0、向量嵌入等实现细节
