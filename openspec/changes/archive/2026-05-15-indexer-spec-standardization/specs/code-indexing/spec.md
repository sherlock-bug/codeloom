# Delta for code-indexing

## ADDED Requirements

### Requirement: 项目头文件类入库
系统 SHALL 将项目头文件（非系统头文件）中定义的类、结构体、枚举作为项目符号入库，`is_external` 为 `false`。对于声明在项目 `.h` 文件中的 CXXRecordDecl，即使 Clang AST 不输出 `loc.file`，SHALL 通过 `includedFrom` 路径属性和翻译单元文件路径判断是否为项目符号。

#### Scenario: 头文件类入库
- GIVEN 类 `DB` 定义在项目头文件 `include/leveldb/db.h` 中
- WHEN 索引器处理包含该头文件的翻译单元
- THEN `DB` SHALL 以 `kind=class` 入库，`is_external=false`
- AND 其方法（`Open`、`Get`、`Put` 等）SHALL 以 `kind=method` 入库

#### Scenario: 头文件结构体入库
- GIVEN 结构体 `Range` 定义在项目头文件 `include/leveldb/db.h` 中
- WHEN 索引器处理该节点
- THEN `Range` SHALL 以 `kind=struct` 入库，`is_external=false`

#### Scenario: 头文件枚举入库
- GIVEN 枚举 `LogLevel` 定义在项目头文件中
- WHEN 索引器处理该 EnumDecl 节点
- THEN `LogLevel` SHALL 以 `kind=enum` 入库
- AND 其枚举值（`LOG_INFO`、`LOG_DEBUG` 等）SHALL 分别入库

### Requirement: 系统头文件类不入库
系统 SHALL 不将系统头文件中定义的类、结构体入库，除非是知名 STL 容器的模板实例化且参数含项目类型。

#### Scenario: 系统类不入库
- GIVEN 类 `std::ctype`、`std::__shared_ptr` 定义在系统头文件中
- WHEN 索引器处理
- THEN 这些类 SHALL 不入库，不创建符号
