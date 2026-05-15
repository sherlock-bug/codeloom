# edge-type-rules

## Purpose
定义 CodeLoom 索引器支持的所有边类型——包括方向、源目标类型、创建条件和外部存根规则，确保边数据的语义一致性。

## Requirements

### Requirement: 所有边类型清单
系统 SHALL 支持以下边类型：
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

#### Scenario: 边类型完整清单
- GIVEN 系统处于正常运行状态
- WHEN 查询所有支持的边类型
- THEN 以上 10 种边类型 SHALL 全部存在且定义正确

### Requirement: inherits 边
系统 SHALL 从派生类的 CXXRecordDecl 的 `bases` 字段中提取基类信息，建立 `inherits:` 边。

#### Scenario: 单继承
- GIVEN 类 `FileLogger` 继承自 `Logger`
- WHEN 索引器处理 `FileLogger` 的 CXXRecordDecl
- THEN SHALL 建立 `inherits:` 边，源为 `FileLogger`，目标为 `Logger`

### Requirement: contains 边
系统 SHALL 从类的成员声明中提取 `contains:` 边，源为类名，目标为成员的限定名。

#### Scenario: 类包含成员
- GIVEN 类 `Logger` 有方法 `log` 和字段 `name`
- WHEN 索引器处理 `Logger` 的 CXXRecordDecl 的 inner 子节点
- THEN SHALL 建立 `contains:` 边，源为 `Logger`，目标为 `Logger::log` 和 `Logger::name`

### Requirement: calls 边
系统 SHALL 从函数体的 DeclRefExpr 中提取 `calls:` 边。仅当目标函数在已索引符号中时建立边，外部函数不建边、不建 stub。

#### Scenario: 内部函数调用
- GIVEN 函数 `initialize_logging` 体内调用了 `report` 和 `get_status_message`
- WHEN 索引器处理函数体中的 DeclRefExpr
- THEN SHALL 为每个直接调用建立 `calls:` 边
- AND 仅当目标函数在已索引符号 `name_to_id` 中时建立边

#### Scenario: 外部函数调用跳过
- GIVEN 函数体内调用了 `memcpy`
- WHEN `memcpy` 不在 `name_to_id` 中
- THEN 不建 `calls:` 边、不建 stub

### Requirement: overrides 边
系统 SHALL 通过继承链检测 `overrides` 边，从派生类方法出发沿 `inherits:` 链向上检查基类是否有同名虚函数。

#### Scenario: 继承链 override
- GIVEN 派生类 `ConsoleLogger` 的方法 `log` 使用 `override` 关键字覆写基类 `Logger::log`
- WHEN 索引器处理 `ConsoleLogger::log` 的 CXXMethodDecl
- THEN 从 `ConsoleLogger` 的基类列表向上查找同名虚函数
- AND 若找到，建立 `overrides:` 边，源为 `ConsoleLogger::log`，目标为 `Logger::log`

### Requirement: instantiates 边
系统 SHALL 从模板实例指向主模板建立 `instantiates:` 边，方向为实例 → 主模板。

#### Scenario: 模板实例化
- GIVEN 模板实例 `foo<int>` 由主模板 `foo` 实例化而来
- WHEN 索引器处理 ClassTemplateSpecializationDecl
- THEN SHALL 建立 `instantiates:` 边，源为实例 `foo<int>`，目标为主模板 `foo`

### Requirement: uses_type 边
系统 SHALL 从字段类型、变量类型、模板实例的参数类型建立 `uses_type:` 边。对 STL 模板类型做 strip 规范化（去 `std::` 前缀和模板参数），使用模板简单名作为目标。

#### Scenario: 字段 uses_type
- GIVEN FieldDecl `std::unique_ptr<Iterator> iter_` 属于类 `Table`
- WHEN 索引器处理该 FieldDecl
- THEN SHALL 建立 `uses_type:` 边，源为 `Table::iter_`，目标为主模板简单名 `unique_ptr`

#### Scenario: 模板实例 uses_type
- GIVEN 模板实例 `vector<FileMetaData*>` 被创建
- WHEN 索引器处理该 ClassTemplateSpecializationDecl
- THEN SHALL 建立 `uses_type:` 边，源为 `vector<FileMetaData*>`，目标为 `FileMetaData`
- AND 对 `std::` 前缀的内置类型参数不建立 `uses_type:` 边

### Requirement: uses 边
系统 SHALL 从函数体的 DeclRefExpr 提取 `uses:` 边，指向所引用的枚举值、全局变量等。

#### Scenario: 枚举值和变量引用
- GIVEN 函数 `initialize_logging` 体内引用了枚举值 `LOG_INFO` 和全局变量 `g_default_logger`
- WHEN 索引器处理函数体中的 DeclRefExpr
- THEN SHALL 建立 `uses:` 边
- AND 枚举值目标使用限定名（如 `LogLevel::LOG_INFO`）
- AND 全局变量使用非限定名

### Requirement: return_type 和 param_type 边
系统 SHALL 从函数声明中提取返回类型和参数类型边。

#### Scenario: 返回类型和参数类型
- GIVEN 函数 `Open` 的返回类型为 `Status`，参数类型为 `const Slice&`
- WHEN 索引器处理 FunctionDecl
- THEN SHALL 建立 `return_type:` 边指向 `Status`
- AND SHALL 建立 `param_type:` 边指向 `Slice`

### Requirement: includes 边
系统 SHALL 从 #include 预处理指令建立 `includes:` 边，从翻译单元文件节点指向被包含文件节点。

#### Scenario: #include 关系
- GIVEN 翻译单元 `db_impl.cc` 中包含 `#include "db_impl.h"`
- WHEN 索引器处理文件节点
- THEN SHALL 建立 `includes:` 边，源为 `db_impl.cc` 文件节点，目标为 `db_impl.h` 文件节点

### Requirement: 模板实例命名规则
系统 SHALL 使用不同的名称区分主模板和模板实例。主模板使用简单名，模板实例使用带类型参数的全名。

#### Scenario: 命名区分
- GIVEN 主模板 `foo` 和其实例化 `foo<int>`
- WHEN 索引器创建符号
- THEN 主模板符号名 SHALL 为 `foo`
- AND 实例符号名 SHALL 为 `foo<int>`（从函数签名提取类型参数拼装）

### Requirement: uses_type 边的模板名规范化
当字段或变量的类型为 STL 模板实例化时，系统 SHALL strip `std::` 前缀和模板参数，使用模板简单名作为 `uses_type:` 的目标。

#### Scenario: STL 容器规范化
- GIVEN VarDecl 类型为 `std::unique_ptr<Iterator>`
- WHEN 索引器建立 `uses_type:` 边
- THEN 目标名 SHALL 为 `unique_ptr`（去除 `std::` 前缀和 `<Iterator>` 参数）

### Requirement: contains 边用于外部模板实例
系统 SHALL 为保留的外部模板实例（含项目类型参数）与其被引用的成员之间建立 `contains:` 边，使桥接符号与其实例关联。

#### Scenario: 模板实例包含成员方法
- GIVEN 模板实例 `vector<Record*>` 的成员 `push_back` 已被保留为桥接符号
- WHEN 索引器建立 contains 边
- THEN SHALL 建立 `contains:` 边，源为 `vector<Record*>`，目标为 `vector<Record*>::push_back`

### Requirement: calls 边指向外部模板实例成员
系统 SHALL 在项目函数体内检测到 CXXMemberCallExpr 调用外部模板实例成员时，建立 `calls:` 边指向该桥接符号。此规则是"calls 边只指向已索引符号"的配套规则——因为桥接符号已被保留，calls 边可以建立。

#### Scenario: 通过外部模板实例的方法调用
- GIVEN 项目函数 `demo_stl_with_project_types` 体内有 `records.push_back(r)`
- AND `vector<Record*>::push_back` 已作为桥接符号保留
- WHEN 索引器处理函数体中的 CXXMemberCallExpr
- THEN SHALL 建立 `calls:` 边，源为 `demo_stl_with_project_types`，目标为 `vector<Record*>::push_back`

### Requirement: impact analysis 穿越外部模板实例
系统 SHALL 支持通过以下路径链完成影响分析穿越：`project_function → calls → template_instance::member → contains → template_instance → uses_type → project_type`。上述链 SHALL 在 radius=5 内可遍历。

#### Scenario: 穿越链验证
- GIVEN 项目函数 `demo_stl_with_project_types` 使用 `vector<Record*>::push_back`
- WHEN 对 `Record` 做反向影响分析（radius=5）
- THEN 结果 SHALL 包含 `demo_stl_with_project_types`（通过 `uses_type → vector<Record*> → contains → push_back → calls → demo_stl_with_project_types`）
- AND 路径中的 `distance` SHALL 正确反映跳数
