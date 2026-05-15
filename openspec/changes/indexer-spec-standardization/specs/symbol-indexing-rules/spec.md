# Delta for symbol-indexing-rules

## ADDED Requirements

### Requirement: 项目自定义符号入库规则
系统 SHALL 将项目自定义的类、结构体、枚举、函数、方法、变量作为符号入库，前提是 `completeDefinition=true` 且声明在项目文件范围内。

#### Scenario: 项目类入库
- GIVEN 项目头文件中定义了 `class DB { ... }`
- WHEN 索引器处理包含该头文件的翻译单元
- THEN 符号 `DB` SHALL 以 `kind=class` 入库
- AND 其 `file_path` SHALL 指向头文件路径（非 `.cc` 文件路径）
- AND 其 `line_start`/`line_end` SHALL 指向类定义在头文件中的起止行

### Requirement: 前向声明不创建符号
系统 SHALL 不将前向声明（`completeDefinition=false` 或缺失）创建为独立符号，但 SHALL 继续递归处理其子节点以维持命名空间上下文和嵌套关系。

#### Scenario: 前向声明跳过
- GIVEN 头文件中有 `class DB;` 前向声明
- WHEN 索引器处理到该 CXXRecordDecl 节点
- THEN 系统 SHALL 不创建 `DB` 符号
- AND SHALL 继续递归处理其子节点

### Requirement: 隐式节点全跳过
系统 SHALL 跳过所有 `isImplicit=true` 的节点，不为其创建符号，不处理其子节点。

#### Scenario: 编译器生成节点跳过
- GIVEN Clang AST 输出中有 `isImplicit=true` 的拷贝构造函数、析构函数等
- WHEN 索引器处理该节点
- THEN 系统 SHALL 跳过该节点及其所有子节点，不创建任何符号

### Requirement: 项目头文件类保库
系统 SHALL 正确处理声明在项目头文件中的类/结构体/枚举。对于 Clang AST 中 `loc.file` 为空的头文件声明，系统 SHALL 通过 `loc.includedFrom` 判断其是否属于项目头文件：如果 `includedFrom.file` 属于项目路径，则视为项目符号。此规则仅适用于 CXXRecordDecl、RecordDecl、EnumDecl 等声明类节点，不适用于 FunctionDecl。

#### Scenario: 头文件类入库
- GIVEN 类 `DB` 声明在 `include/leveldb/db.h` 中（项目头文件）
- AND Clang AST 中该 CXXRecordDecl 的 `loc.file` 为空
- AND `loc.includedFrom.file` 为项目内的 `.cc` 文件路径
- WHEN 索引器处理该节点
- THEN 符号 `DB` SHALL 以 `kind=class` 入库
- AND 其 `is_external` SHALL 为 `false`

#### Scenario: 系统头文件类仍丢弃
- GIVEN 类 `std::vector` 声明在系统头文件中
- AND Clang AST 中该 CXXRecordDecl 的 `loc.file` 为空
- AND `loc.includedFrom.file` 为项目内的 `.cc` 文件路径
- WHEN 索引器处理该节点
- THEN 系统 SHALL 检查该节点是否属于知名 STL 容器或模板实例化（由模板实例规则决定保留）
- AND 非知名 STL 容器的系统类 SHALL 不创建符号

### Requirement: 外部符号存根
系统 SHALL 为被项目符号引用的外部类型（非项目内的类型）创建唯一的外部存根符号，仅建唯一定位点，不展开其内部细节。

#### Scenario: 外部基类存根
- GIVEN 项目类 `MyClass` 继承自外部类 `ExternalBase`
- WHEN 索引器创建 `inherits:` 边
- THEN 系统 SHALL 在 `ExternalBase` 不存在于已索引符号中时创建外部存根
- AND 存根的 `is_external` SHALL 为 `true`

### Requirement: 函数调用不创建外部存根
系统 SHALL 不为 `calls:` 边指向的外部函数创建外部存根或边。函数体中调用外部函数（如 `memcpy`、`printf`）时，SHALL 跳过该调用边。

#### Scenario: C 库函数调用跳过
- GIVEN 项目函数体中调用了 `memcpy`
- WHEN 索引器处理函数体的 DeclRefExpr
- THEN 系统 SHALL 检查目标 `memcpy` 是否在已索引符号中
- AND 若不在，SHALL 跳过该调用边，不创建存根
