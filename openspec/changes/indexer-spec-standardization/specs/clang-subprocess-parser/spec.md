# Delta for clang-subprocess-parser

## MODIFIED Requirements

### Requirement: is_system 判断标准（替代旧版）
原规格：系统通过 `loc.file` 判断是否系统符号，无 file 时通过 `loc.line` 和 `includedFrom` 辅助判断。
新规格：系统 SHALL 仅依赖 `loc.file` 判断系统符号。无 `loc.file` 的声明节点（非 body 节点）SHALL 视为系统符号，不得仅凭 `loc.line` 或 `includedFrom` 判定为非系统。C 库函数也有行号，不应因此被误判为项目符号。

#### Scenario: C 库函数被正确过滤（替代旧版）
- GIVEN 系统头文件中的函数 `strstr`，其 AST 节点有 `loc.line` 和 `includedFrom` 但无 `loc.file`
- WHEN is_system() 检查该节点
- THEN 无 `loc.file` → SHALL 返回 `true`
- AND 该节点被 filter 丢弃

### Requirement: 路径注入范围（替代旧版）
原规格：系统向子节点传播项目文件路径。
新规格：系统 SHALL 仅对 `BODY_KINDS`（语句/表达式节点）和 `DeclRefExpr` 注入项目文件路径。对 FunctionDecl、CXXMethodDecl、CXXRecordDecl 等声明节点，SHALL 不注入。

#### Scenario: 声明节点不被注入
- GIVEN 系统头文件中的 FunctionDecl 节点
- WHEN filter 遍历其子节点
- THEN 子节点为声明类（FunctionDecl 等），SHALL 不接收路径注入
- AND 子节点保留无文件状态，`is_system()` 为 `true`

### Requirement: CXXRecordDecl 头文件声明保库（新增）
系统 SHALL 在 filter 层对 CXXRecordDecl/RecordDecl/EnumDecl 节点额外处理：即使无 `loc.file`，通过 `loc.includedFrom.file` 检查是否属于项目路径。若属于，保留该节点，但不在 `loc` 中注入文件路径。此规则仅适用于声明类节点，不适用于 FunctionDecl。

#### Scenario: 项目头文件类通过 filter
- GIVEN 项目头文件 `db.h` 中的 CXXRecordDecl `DB`
- AND 其 `loc.includedFrom.file` 指向项目内的 `.cc` 文件
- WHEN filter 处理该节点
- THEN 节点 SHALL 被保留并传递给 Rust 层
- AND filter 不修改节点的 `loc` 内容

### Requirement: 系统头文件中的 CXXRecordDecl 特殊过滤（新增）
系统 SHALL 在 filter 层对通过 includedFrom 幸存下来的 CXXRecordDecl 节点做二次过滤：若节点是知名 STL 容器（vector、map 等），交由模板实例规则判断是否保留；否则丢弃。其子 FunctionDecl 按 FunctionDecl 规则独立判断。

#### Scenario: 系统类丢弃
- GIVEN 系统头文件中的 `std::__shared_ptr` 类
- WHEN filter 处理该 CXXRecordDecl
- THEN 非知名 STL 容器 → SHALL 被丢弃
- AND 其子节点不被处理
