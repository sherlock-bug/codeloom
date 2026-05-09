# Proposal: 模板解析

## Why

CodeLoom 当前对 C++ 模板的处理是"透过模板看身体"——`template<typename T> class MyVector { ... }` 被提取为普通的 `class MyVector`，模板参数 `T` 丢弃，模板使用 `MyVector<int>` 不检测，标准库容器 `std::vector<B>` 不产生任何类间关系边。

这导致三个问题：
1. **头文件 only 仓库能力缺失**：大量模板库（header-only）只有模板声明和定义，当前提取出的符号缺乏模板语义，搜索不到模板特有的信息
2. **使用分析空白**：`MyVector<User>` 实例化时，无法知道 MyVector 和 User 之间的模板使用关系
3. **类间关系建模缺失**：`class A { std::vector<B> data; }` → 无法知道 A 聚合 B。这对后续调用链和架构理解至关重要

## What Changes

- **NEW**: `template_declaration` 下的 class/function 提取为 `template_class` / `template_function` kind
- **NEW**: 提取模板参数（`template_param:T[type]`、`template_param:N[non_type]` 边）
- **NEW**: 检测自定义模板使用（`MyVector<int>` → `template_use:MyVector<int>` 边，双模式 to）
- **NEW**: 检测 std 容器使用并建类间关系边（`vector<T>` → `aggregate:T`，`unique_ptr<T>` → `owns:T`，`shared_ptr<T>` → `shares:T`，`map<K,V>` → `map_key:K` + `map_value:V`，全部双模式 to）
- **NEW**: 双模式解析函数——能匹配到项目内符号则用真实 ID，否则 MAX

## Capabilities

### New Capabilities
- `template-parsing`: C++ 模板声明、模板参数、模板使用、std 容器类间关系

### Modified Capabilities
无。

## Impact

- 受影响文件：`src/indexer/queries/cpp.rs`（~100 行新增）
- kind 新增：`template_class`、`template_function`
- 边新增：`template_param:`、`template_use:`、`aggregate:`、`owns:`、`shares:`、`map_key:`、`map_value:`
- 测试：需验证新 kind/边在索引后正确出现
