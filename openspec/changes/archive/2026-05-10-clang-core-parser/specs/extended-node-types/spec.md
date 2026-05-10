# Delta for extended-node-types

## ADDED Requirements

### Requirement: 识别并创建命名空间节点
系统 SHALL 从 Clang AST 的 NamespaceDecl 节点中提取具名命名空间，创建 kind=namespace 的符号节点，并建立符号与命名空间之间的包含关系。

#### Scenario: 提取顶级命名空间
- GIVEN 源文件包含 `namespace myapp { class Foo {}; }`
- WHEN 解析对应的 NamespaceDecl 和其中的 CXXRecordDecl
- THEN 创建 kind=namespace、name=myapp 的符号，创建 kind=class、name=Foo、namespace=myapp 的符号

#### Scenario: 嵌套命名空间
- GIVEN 源文件包含 `namespace outer { namespace inner { int x; } }`
- WHEN 解析 NamespaceDecl 和 VarDecl
- THEN 创建namespace=outer 和 namespace=outer::inner 节点，变量 x 的 namespace 为 outer::inner

#### Scenario: 匿名命名空间
- GIVEN 源文件包含 `namespace { int hidden; }`
- WHEN 解析匿名 NamespaceDecl
- THEN 不创建独立 namespace 节点，内部符号的 namespace 标记为 "(anonymous)"

### Requirement: 识别并创建模板实例化节点
系统 SHALL 识别本项目定义的类模板的实例化（ClassTemplateSpecializationDecl），创建 kind=template_instance 的符号节点，并通过 instantiates: 边关联到主模板。

#### Scenario: 本项目模板实例化
- GIVEN 本项目定义了 `template<typename T> class MyVec { ... }`（kind=class），且源文件中有 `MyVec<int> v;`
- WHEN 解析 ClassTemplateSpecializationDecl 节点，且主模板 MyVec 在本项目源文件中
- THEN 创建 kind=template_instance、name="MyVec<int>"、template_args='["int"]' 的符号，创建 MyVec<int> → MyVec 的 instantiates:MyVec 边

#### Scenario: STL 模板实例化不创建
- GIVEN 源文件中有 `std::vector<int> v;`
- WHEN 解析对应的 ClassTemplateSpecializationDecl，发现主模板 `std::vector` 不在本项目源文件中
- THEN 不创建 template_instance 节点，外部类型通过 is_external=1 处理

### Requirement: 识别并创建 typedef/using 节点
系统 SHALL 从 Clang AST 的 TypedefDecl 和 TypeAliasDecl 节点中提取类型别名，创建 kind=typedef 的符号节点，并通过 aliases: 边指向底层类型。

#### Scenario: typedef 别名
- GIVEN 源文件包含 `typedef unsigned int uint32_t;`
- WHEN 解析 TypedefDecl 节点
- THEN 创建 kind=typedef、name="uint32_t" 的符号，创建 aliases:unsigned int 边

#### Scenario: using 别名
- GIVEN 源文件包含 `using StringMap = std::map<std::string, std::string>;`
- WHEN 解析 TypeAliasDecl 节点
- THEN 创建 kind=typedef、name="StringMap" 的符号，创建 aliases: 边指向底层类型

### Requirement: 识别并创建字符串字面量节点
系统 SHALL 从 Clang AST 的 StringLiteral 表达式中提取字符串字面量，创建 kind=string_literal 的符号节点，按字符串值去重。

#### Scenario: 提取字符串字面量
- GIVEN 函数内包含 `const char* url = "https://api.example.com/v1";`
- WHEN 解析 StringLiteral 表达式
- THEN 创建 kind=string_literal、name="https://api.example.com/v1" 的符号（同值字符串复用已存在节点）

#### Scenario: 字符串去重
- GIVEN 三个文件中都出现了字符串 `"timeout"`
- WHEN 依次解析三个 StringLiteral
- THEN 只创建一个 kind=string_literal、name="timeout" 的节点（按 name 去重）

### Requirement: 外部符号标识
系统 SHALL 用 `is_external` 布尔列标识外部符号，复用 `kind` 列存储 Clang 原始类型（function/class/struct 等），不创建独立节点类型。

#### Scenario: 外部函数标记
- GIVEN 本项目中调用了 `std::sort()`，其声明来自系统头文件
- WHEN 解析 CallExpr→DeclRefExpr 判定目标不在项目索引范围内
- THEN 创建 kind=function、name="std::sort"、namespace="std"、is_external=1 的符号

#### Scenario: 外部符号去重
- GIVEN 多个文件都引用了 `std::string`
- WHEN 第一个文件创建了 kind=class、is_external=1 的符号，后续文件再次遇到
- THEN 按 (name, namespace, kind) 去重，不重复插入

### Requirement: 符号表扩展列
系统 SHALL 为 symbols 表新增 sid、access、is_virtual、is_definition、is_external、template_args 六列。access 仅 method 和 field 有值（其余 NULL），is_virtual 仅 method 有意义（其余 0），is_external 和 is_definition 全局有效，template_args 仅 template_function 和 template_instance 有值。

#### Scenario: 虚函数标记
- GIVEN 源文件包含 `virtual void foo() override;`
- WHEN 解析对应的 CXXMethodDecl
- THEN 该符号的 is_virtual=1，且创建 overrides: 边指向被覆写的基类方法

#### Scenario: access 区分公有/私有
- GIVEN 类中包含 `public: void api();` 和 `private: int m_data;`
- WHEN 解析对应的 CXXMethodDecl 和 FieldDecl
- THEN api 的 access="public"，m_data 的 access="private"

#### Scenario: 声明合并后 is_definition
- GIVEN 头文件中 `int add(int,int);` 声明和源文件中 `int add(int,int){...}` 定义
- WHEN 先解析声明创建 is_definition=0，后解析定义匹配到同一符号
- THEN 更新 is_definition=1，位置更新到定义处，注释拼接
