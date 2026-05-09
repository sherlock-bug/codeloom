# Design: 模板解析

## Context

CodeLoom 当前 `walk_children` 的 `template_declaration` match arm 只提取 body 中的 class/function，模板参数（`template_parameter_list`）和模板使用（`template_type`）完全跳过。标准库容器 `std::vector<T>` 出现在字段声明中但不产生任何关系边。

本设计在已有 `template_declaration` 处理基础上，增加参数提取、模板标记、使用检测、容器关系四条链路。

## Goals / Non-Goals

**Goals:**
- `template_declaration` 下的 class/function 标记为 `template_class` / `template_function`
- 模板参数提取为边
- 自定义模板使用检测（`MyVector<User>` → `template_use:MyVector<User>`）
- std 容器生成类间关系边（`vector<B>` → `aggregate:B` 等，双模式）
- 双模式 to 解析

**Non-Goals:**
- 模板特化检测
- 变参模板（`template<typename... Args>`）——参数名同样提取即可
- `std::function`、`std::variant` 等复杂类型——暂不处理
- 非 C++ 语言

## Decisions

### Decision 1: 新 kind 而非标记
选择 `template_class` / `template_function` 而非在 class/function 上加 `is_template` 标记。

原因：
- 搜索 "template" 直接命中 kind 字段，不用扫标记列
- 不影响已有 class/function 查询的语义
- 实现简单：`template_declaration` 下发现的 class → 直接改 kind 为 `template_class`

### Decision 2: 模板参数放边中
`template_param:T[type]` 格式，`T` 是参数名，`[type]` 是参数类别（type/non_type/template）。

原因：
- 参数名是占位符，不是符号，不应建节点
- 边可结构化查询：`WHERE edge_type LIKE 'template_param:%'`

### Decision 3: 容器关系映射表
硬编码 std 容器 → 关系边类型映射：

| 容器 | 边类型 |
|------|--------|
| `std::vector` `std::list` `std::deque` `std::set` `std::unordered_set` `std::multiset` `std::stack` `std::queue` `std::priority_queue` `std::array` | `aggregate` |
| `std::unique_ptr` | `owns` |
| `std::shared_ptr` `std::weak_ptr` | `shares` |
| `std::map` `std::unordered_map` `std::multimap` | `map_key` + `map_value` |

不引入配置文件——这些映射极其稳定，配置文件过度设计。

### Decision 4: 双模式 to 解析
`resolve_or_max(name, symbols)` 函数：先精确匹配 `name`，再试 `*::name` 后缀匹配，都失败返回 `usize::MAX`。

### Decision 5: 边解析修复（smart.rs）
`smart.rs` 的 `resolve_target` 原本无视提取器传递的 `usize::MAX`，对所有边统一走名字解析（LIKE 子串匹配），导致 `field_type:T` 被 LIKE `%T%` 误匹配到 MyVector。

修复方案：
1. 信任提取器的判断 — `tgt_idx == usize::MAX` 直接 to=0，不走名字解析
2. LIKE 后缀匹配 — `%X%` 必命中含 `X` 的任何名字，改为 `%::X` 只匹配限定名后缀
3. map 容器按边类型分别解析 — `map_key:int` 和 `map_value:User` 各自调用 `resolve_or_max`

原因：提取器的 `usize::MAX` 是显式的"无法解析"信号，DB 层不应推翻。`%X%` 在大小写不敏感的 LIKE 中会命中所有含该字符的符号名，对单字符类型名尤其危险。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| `std::vector<bool>` 触发 aggregate 边 | bool 不是类，不会匹配到符号；to=MAX，无害 |
| 模板使用节点名可能含 `::` 前缀 | 双模式的 `*::name` 后缀匹配覆盖 |
| kind 从 `class` 变为 `template_class` 影响已有搜索 | 模板类本来就少；template_class 更精确 |
| `resolve_target` LIKE 子串匹配误命中 | 改为后缀匹配 `%::X` + 信任提取器的 usize::MAX |
| map 容器 key/value 共用 resolve 结果 | 改为逐边分别解析 |
