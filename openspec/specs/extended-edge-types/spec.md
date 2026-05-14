# extended-edge-types

## Purpose
CodeLoom SHALL 从 Clang AST 中提取边类型，包括调用、继承、参数类型、返回类型、包含、覆写、别名等关系，构建完整的代码关系图。

> ⚠️ **说明**：以下边类型中有部分尚未由索引器实现（标记为 🔴）。对应规格但未实现的边在 `known-limitations/spec.md` 中记录。开发者提交变更前 SHALL 按约定更新相关场景。

### Requirement: 外部调用通过 is_external 区分 ✅ 已实现
系统 SHALL 对调用外部符号使用普通 `calls:` 边，通过目标符号的 `is_external` 列区分内外部，不创建独立的 calls_external: 边类型。
- GIVEN 本项目函数 `process()` 调用了 `std::sort`
- WHEN 解析到该调用
- THEN 创建 source=process、target=std::sort、edge_type="calls:std::sort" 的边，其中 std::sort 的 is_external=1

### Requirement: 提取参数类型边
系统 SHALL 从函数声明的参数列表中提取每个参数的类型，创建 param_type: 边，从函数节点指向参数类型对应的 class/struct/enum 节点。

#### Scenario: 参数类型关联
- GIVEN 函数 `void handle(MyClass obj)`，MyClass 是本项目中定义的类
- WHEN 解析 ParmVarDecl
- THEN 创建 source=handle、target=MyClass、edge_type="param_type:MyClass" 的边

#### Scenario: 参数类型为基本类型
- GIVEN 函数 `void calc(int x)`
- WHEN 解析 ParmVarDecl，参数类型为 int（内置类型）
- THEN 不创建 param_type 边（内置类型不索引）

### Requirement: 提取返回值类型边
系统 SHALL 从函数声明的返回类型提取类型信息，创建 return_type: 边，从函数节点指向返回类型对应的节点。

#### Scenario: 返回自定义类型
- GIVEN 函数 `MyClass create()`
- WHEN 解析 FunctionDecl，返回类型为 MyClass
- THEN 创建 source=create、target=MyClass、edge_type="return_type:MyClass" 的边

### Requirement: 提取 include 边
系统 SHALL 从 Clang AST 的预处理信息中提取 #include 关系，创建 includes: 边，从源文件指向被 include 的头文件（以文件级边形式存储）。

#### Scenario: 源文件 include 头文件
- GIVEN `main.cpp` 包含 `#include "helper.h"`
- WHEN 解析 main.cpp 的 AST，提取预处理指令
- THEN 创建 source=main.cpp、target=helper.h、edge_type="includes:helper.h" 的边

### Requirement: 提取 typedef 别名边
系统 SHALL 从 TypedefDecl/TypeAliasDecl 中提取 aliases: 边，从 typedef 节点指向底层类型。

#### Scenario: typedef 指向类型
- GIVEN `typedef MyClass* MyClassPtr;`
- WHEN 解析 TypedefDecl
- THEN 创建 source=MyClassPtr、target=MyClass、edge_type="aliases:MyClass" 的边

### Requirement: 提取函数对变量的依赖边
系统 SHALL 从函数体内部对全局变量、静态变量、枚举值、字符串字面量的引用中提取 uses: 边，从 function/method 指向被引用的变量节点。

#### Scenario: 函数使用全局变量
- GIVEN 函数 `process()` 内部引用了全局变量 `g_config`
- WHEN 解析 process 函数体内的 DeclRefExpr→g_config
- THEN 创建 source=process、target=g_config、edge_type="uses:g_config" 的边

#### Scenario: 函数使用字符串字面量
- GIVEN 函数内 `printf("error: %s", msg);`
- WHEN 解析 CallExpr 参数中的 StringLiteral
- THEN 创建 source=当前函数、target="error: %s"（string_literal节点）、edge_type="uses:error: %s" 的边

### Requirement: 提取 contains 边
系统 SHALL 从 Clang AST 的类定义中提取 contains: 边，从 class/struct 指向其成员方法/字段，替代 parent_class 列。

#### Scenario: 类包含方法
- GIVEN 类 `class Foo { void bar(); int x; };`
- WHEN 解析 CXXRecordDecl 及其子节点
- THEN 创建 source=Foo、target=bar、edge_type="contains:bar" 和 source=Foo、target=x、edge_type="contains:x" 的边

### Requirement: 合并 overrides 边
系统 SHALL 将原来的 calls_override: 逻辑合并为 overrides: 边，从覆写方法指向被覆写的基类虚方法。

#### Scenario: 虚函数覆写
- GIVEN Base::foo 是虚函数，Derived::foo 加了 override 关键字
- WHEN 解析 Derived 的 CXXMethodDecl，检测到 override 标记
- THEN 创建 source=Derived::foo、target=Base::foo、edge_type="overrides:Base::foo" 的边
