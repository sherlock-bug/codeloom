# Delta for clang-subprocess-parser

## ADDED Requirements

### Requirement: 前向声明类符号过滤
Clang 索引器在处理 CXXRecordDecl 和 ClassTemplateDecl 节点时，SHALL 检查 `completeDefinition` 字段。没有该字段或字段值为 false 的节点 SHALL 被跳过，不生成符号。

#### Scenario: 前向声明不生成 class 节点
- GIVEN 一个 C++ 翻译单元中包含 `class Foo;` 前向声明和 `class Foo { ... };` 定义
- WHEN Clang 解析器提取符号
- THEN 仅定义处的 CXXRecordDecl 生成 class 节点
- AND 前向声明处的 CXXRecordDecl SHALL 被跳过

#### Scenario: 多文件前向声明不重复
- GIVEN 类 `Foo` 定义在 `foo.h`，同时前向声明在 `bar.h` 和 `baz.h`
- WHEN 索引整个项目
- THEN 数据库中 SHALL 只存在一个 `Foo` 的 class 节点
- AND 该节点的 file_path SHALL 指向 `foo.h`

#### Scenario: struct 前向声明同样处理
- GIVEN `struct Bar;` 前向声明
- WHEN Clang 解析器处理该节点
- THEN SHALL 被跳过，与 class 的前向声明行为一致
