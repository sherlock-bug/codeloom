# Delta for edge-type-rules

## ADDED Requirements

### Requirement: 所有边类型定义
系统 SHALL 支持以下边类型，每条边有明确的源类型、目标类型和方向。

#### Scenario: 边类型完整清单
- GIVEN 系统处于正常运行状态
- WHEN 查询所有支持的边类型
- THEN 以下边类型 SHALL 全部存在且定义正确：
  - `inherits:` 派生类 → 基类
  - `contains:` 类 → 成员方法/字段
  - `calls:` 调用方函数 → 被调用方函数
  - `overrides:` 派生类方法 → 基类虚方法
  - `instantiates:` 模板实例 → 主模板
  - `uses_type:` 字段/变量/模板实例 → 类型
  - `uses:` 函数体 → 所引用的枚举值/变量
  - `return_type:` 函数 → 返回类型
  - `param_type:` 函数 → 参数类型
  - `includes:` 文件节点 → 被包含的文件

### Requirement: 边方向与源目标类型

#### Scenario: inherits 边规格
- GIVEN 类 `FileLogger` 继承自 `Logger`
- WHEN 索引器处理 `FileLogger` 的 CXXRecordDecl
- THEN SHALL 建立 `inherits:` 边，源为 `FileLogger`，目标为 `Logger`

#### Scenario: contains 边规格
- GIVEN 类 `Logger` 有方法 `log` 和字段 `name`
- WHEN 索引器处理 `Logger` 的 CXXRecordDecl 的 inner 子节点
- THEN SHALL 建立 `contains:` 边，源为 `Logger`，目标为 `Logger::log` 和 `Logger::name`
- AND 边的源为类名前缀的限定名（`Logger`），目标为成员限定名（`Logger::log`）

#### Scenario: calls 边规格
- GIVEN 函数 `initialize_logging` 体内调用了 `report` 和 `get_status_message`
- WHEN 索引器处理函数体中的 DeclRefExpr
- THEN SHALL 为每个直接调用建立 `calls:` 边
- AND 仅当目标函数在已索引符号 `name_to_id` 中时建立边
- AND 目标不在已索引符号中时不建边、不建 stub

#### Scenario: overrides 边规格
- GIVEN 派生类 `ConsoleLogger` 的方法 `log` 使用 `override` 关键字覆写基类 `Logger::log`
- WHEN 索引器处理 `ConsoleLogger::log`
- THEN SHALL 通过继承链检测建立 `overrides:` 边
- AND 源为 `ConsoleLogger::log`，目标为 `Logger::log`

#### Scenario: instantiates 边规格
- GIVEN 模板实例 `foo<int>` 由主模板 `foo` 实例化而来
- WHEN 索引器处理 ClassTemplateSpecializationDecl
- THEN SHALL 建立 `instantiates:` 边，**源为实例 `foo<int>`，目标为主模板 `foo`**

#### Scenario: uses_type 边规格
- GIVEN 有一个 FieldDecl `std::unique_ptr<Iterator> iter_` 属于类 `Table`
- WHEN 索引器处理该 FieldDecl
- THEN SHALL 建立 `uses_type:` 边，源为 `Table::iter_`，目标为主模板简单名 `unique_ptr`（strip `std::` 前缀和模板参数）
- GIVEN 模板实例 `vector<FileMetaData*>` 被创建
- WHEN 索引器处理该 ClassTemplateSpecializationDecl
- THEN SHALL 建立 `uses_type:` 边，源为 `vector<FileMetaData*>`，目标为 `FileMetaData`（项目类型参数）
- AND 对 `std::` 前缀的内置类型参数（如 `int`、`char`）不建立 `uses_type:` 边

#### Scenario: uses 边规格
- GIVEN 函数 `initialize_logging` 体内引用了枚举值 `LOG_INFO` 和全局变量 `g_default_logger`
- WHEN 索引器处理函数体中的 DeclRefExpr
- THEN SHALL 建立 `uses:` 边，源为 `initialize_logging`
- AND 枚举值目标使用限定名（如 `LogLevel::LOG_INFO`），全局变量使用非限定名
- AND 字符串字面量目标使用 `string_literals:` 前缀

#### Scenario: return_type 边规格
- GIVEN 函数 `Open` 的返回类型为 `Status`
- WHEN 索引器处理 FunctionDecl
- THEN SHALL 建立 `return_type:` 边，源为 `Open`，目标为 `Status`

#### Scenario: param_type 边规格
- GIVEN 函数 `Get` 的参数类型为 `const Slice&` 和 `std::string*`
- WHEN 索引器处理 FunctionDecl 的 ParmVarDecl
- THEN SHALL 建立 `param_type:` 边，源为 `Get`，目标分别为 `Slice` 和 `std::string`

#### Scenario: includes 边规格
- GIVEN 翻译单元 `db_impl.cc` 中包含 `#include "db_impl.h"`
- WHEN 索引器处理文件节点
- THEN SHALL 建立 `includes:` 边，源为 `db_impl.cc` 文件节点，目标为 `db_impl.h` 文件节点

### Requirement: 模板实例命名规则
系统 SHALL 使用不同的名称区分主模板和模板实例，防止 `name_to_id` 冲突。主模板使用简单名（如 `foo`），模板实例使用带类型参数的全名（如 `foo<int, double>`）。

#### Scenario: 模板命名区分
- GIVEN 主模板 `foo` 和其实例化 `foo<int>`
- WHEN 索引器创建符号
- THEN 主模板符号名 SHALL 为 `foo`
- AND 实例符号名 SHALL 为 `foo<int, double>`（从函数签名中提取类型参数拼装）

### Requirement: uses_type 边的模板名规范化
当字段或变量的类型为 STL 模板实例化（如 `std::unique_ptr<Iterator>`）时，系统 SHALL strip `std::` 前缀和模板参数，使用模板简单名作为 `uses_type:` 的目标，使 `name_to_id` 能解析到模板实例节点。

#### Scenario: STL 容器 uses_type 规范化
- GIVEN VarDecl 类型为 `std::unique_ptr<Iterator>`
- WHEN 索引器建立 `uses_type:` 边
- THEN 目标名 SHALL 为 `unique_ptr`（去除 `std::` 前缀和 `<Iterator>` 参数）
- AND 使 `name_to_id` 能解析到 `unique_ptr` 的 template_instance 节点
