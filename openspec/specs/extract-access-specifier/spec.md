# extract-access-specifier Specification

## Purpose
TBD - created by archiving change extract-access-specifier. Update Purpose after archive.
## Requirements
### Requirement: 类成员访问级别提取
Clang 解析器 SHALL 从 Clang AST 中提取 CXXMethodDecl 和 FieldDecl 节点的访问级别（public/private/protected），写入 symbols.access 列。

#### Scenario: 显式 public 方法
- GIVEN 一个 C++ 类含有 `public: void foo();`
- WHEN Clang 解析器索引该类
- THEN symbol `foo` 的 access 列 SHALL 为 "public"

#### Scenario: 显式 private 字段
- GIVEN 一个 C++ 类含有 `private: int m_count;`
- WHEN Clang 解析器索引该类
- THEN symbol `m_count` 的 access 列 SHALL 为 "private"

#### Scenario: struct 隐式 public
- GIVEN 一个 C++ struct 定义，成员在隐式 public 区域
- WHEN Clang 解析器索引该 struct 且无任何 AccessSpecDecl 节点
- THEN 所有成员方法的 access 列 SHALL 为 "public"

#### Scenario: class 隐式 private
- GIVEN 一个 C++ class 定义，成员在隐式 private 区域
- WHEN Clang 解析器索引该 class 且无任何 AccessSpecDecl 节点
- THEN 所有成员方法的 access 列 SHALL 为 "private"

#### Scenario: 混合访问级别
- GIVEN 一个 C++ 类包含 public 方法和 private 方法
- WHEN Clang 解析器索引该类
- THEN 每个方法的 access 列 SHALL 与其所在的访问区域匹配

