# Delta for extended-edge-types

## MODIFIED Requirements

### Requirement: 添加 instantiates 边类型（新增）
系统 SHALL 新增 `instantiates:` 边类型，方向为模板实例 → 主模板。例如 `foo<int>` `instantiates:` `foo`。

#### Scenario: instantiates 边
- GIVEN 模板实例 `DataStore<int>` 由主模板 `DataStore` 实例化而来
- WHEN 索引器处理 ClassTemplateSpecializationDecl
- THEN 创建 `instantiates:` 边，源为 `DataStore<int>`，目标为 `DataStore`

### Requirement: 添加 uses_type 边覆盖范围（修正）
原规格：无明确要求从模板实例到参数类型的 uses_type 边。
新规格：系统 SHALL 从模板实例建立 `uses_type:` 边到其模板参数中的项目类型。例如 `vector<FileMetaData*>` `uses_type:` `FileMetaData`。

#### Scenario: 模板实例 uses_type
- GIVEN 模板实例 `vector<FileMetaData*>`
- WHEN 索引器处理该 CTS 节点
- THEN 对每个非 `std::`、非内置类型的模板参数，建立 `uses_type:` 边
- AND 边目标为 strip 掉 cv-qualifier 和指针引用的类型名

### Requirement: overrides 边通过继承链检测（修正）
原规格：依赖 Clang AST 的 overridden 字段。
新规格：系统 SHALL 通过继承链检测 `overrides` 边。从派生类方法出发，沿 `inherits:` 链向上检查基类是否有同名虚函数。

#### Scenario: 继承链 override
- GIVEN `ConsoleLogger` 继承自 `Logger`
- AND `ConsoleLogger::log` 使用 `override` 关键字
- WHEN 索引器处理 CXXMethodDecl
- THEN 遍历 `ConsoleLogger` 的基类链查找同名虚方法
- AND 若找到，建立 `overrides:` 边

## REMOVED Requirements

### Requirement: 提取 typedef 别名边
原规格创建 `aliases:` 边从 typedef 指向底层类型。
删除原因：`aliases:` 边类型未在索引器中实现，且对 current use cases 不关键。实现时视需求重新评估。

### Requirement: 外部调用通过 is_external 区分
原规格对外部调用使用普通 `calls:` 边。
删除原因：`calls:` 边不再为外部目标建立（按新规格，外部函数调用不建边、不建 stub）。
