# template-parsing

## Purpose
CodeLoom C++ 索引器的模板解析功能域。支持模板类/函数提取、模板参数边、std 容器关系边（aggregate/owns/shares/map_key/map_value）以及自定义模板使用检测，所有边使用双模式符号解析（真实 ID 或 usize::MAX）。

## Requirements

### Requirement: 模板类提取
系统 SHALL 将 `template_declaration` 下的 `class_specifier` / `struct_specifier` 提取为 `kind: "template_class"` 的符号节点。

#### Scenario: 模板类被搜索
- GIVEN 源文件包含 `template<typename T> class MyVector { ... }`
- WHEN 执行索引
- THEN `codeloom search "MyVector"` 返回类型为 `template_class` 的符号

### Requirement: 模板函数提取
系统 SHALL 将 `template_declaration` 下的 `function_definition` 提取为 `kind: "template_function"` 的符号节点。

#### Scenario: 模板函数被搜索
- GIVEN 源文件包含 `template<typename Func> void for_each(Func f) { ... }`
- WHEN 执行索引
- THEN `codeloom search "for_each"` 返回类型为 `template_function` 的符号

### Requirement: 模板参数提取
系统 SHALL 提取模板参数列表中的每个参数为边，格式为 `template_param:NAME[KIND]`，其中 `KIND` 为 `type`（类型参数）、`non_type`（非类型参数）或 `template`（模板模板参数）。

#### Scenario: 类型参数边
- GIVEN `template<typename T>` 定义
- WHEN 索引
- THEN MyVector 有一条 `template_param:T[type]` 边

### Requirement: 自定义模板使用检测
系统 SHALL 检测 `template_type` 节点对自定义模板的实例化（如 `MyVector<int>`），若模板名匹配已有的 `template_class` 符号，则建立 `template_use:NAME<ARGS>` 边。边 to 字段 SHALL 使用双模式解析——能匹配到项目内符号则用真实 ID，否则为 `usize::MAX`。

#### Scenario: 自定义模板使用
- GIVEN 已索引 `template_class MyVector`，且有代码 `MyVector<int> data;`
- WHEN 索引
- THEN `data` 字段有一条 `template_use:MyVector<int>` 边

### Requirement: std 容器类间关系
系统 SHALL 检测字段声明中的 std 容器类型并建立类间关系边：
- `std::vector<T>` 等序列/集合容器 SHALL 建 `aggregate:T` 边
- `std::unique_ptr<T>` SHALL 建 `owns:T` 边
- `std::shared_ptr<T>` / `std::weak_ptr<T>` SHALL 建 `shares:T` 边
- `std::map<K,V>` 等映射容器 SHALL 建 `map_key:K` + `map_value:V` 边
所有边 SHALL 使用双模式 to 解析。

#### Scenario: vector 聚合
- GIVEN `class A { std::vector<B> items; }`，B 是项目内已索引的类
- WHEN 索引
- THEN A 有一条 `aggregate:B` 边，to 为 B 的真实符号 ID

#
