# system-symbol-filter

## Purpose
定义 CodeLoom 系统符号过滤的层级、标准和保留条件——包括 is_system 的判断依据、路径注入范围限制、STL 内部细节过滤以及模板实例保留规则，确保系统头文件噪声不污染索引数据。

## Requirements

### Requirement: is_system 判断依据
系统 SHALL 根据节点的文件路径判断是否属于系统符号。对于有 `loc.file` 的节点，检查文件路径是否在项目根目录下。对于无 `loc.file` 的声明节点（非 body 节点），不得仅凭有 `loc.line` 就判定为非系统符号。此规则在 Python filter 层和 Rust 层双重实施。

#### Scenario: C 库函数被过滤
- GIVEN 系统头文件中的函数 `strstr`，其 AST 节点有 `loc.line` 但无 `loc.file`
- WHEN Python filter 处理该 FunctionDecl 节点
- THEN `is_system()` SHALL 返回 `true`
- AND 该节点 SHALL 被 filter 丢弃
- GIVEN 同样的节点到达 Rust `ast.rs`（双保险）
- WHEN `decl_file` 为空且 `!is_def`（无函数体）
- THEN SHALL 直接 `return` 跳过

### Requirement: 路径注入范围限制
系统 SHALL 仅在处理 `BODY_KINDS`（语句/表达式节点）和 `DeclRefExpr` 时注入项目文件路径到子节点的 `loc.file` 中。对声明节点（FunctionDecl、CXXMethodDecl 等），SHALL 不进行路径注入。

#### Scenario: 声明节点不注入
- GIVEN 系统头文件中的 FunctionDecl 节点，其子节点缺 `loc.file`
- WHEN Python filter 遍历子节点
- THEN SHALL 不向该子节点注入 `.cc` 文件路径
- AND 该子节点保留 `loc` 为空状态，`is_system()` 返回 `true`

### Requirement: STL 内部细节过滤
系统 SHALL 过滤以下 STL 内部细节类，这些纯编译期 machinery 对项目级代码理解无语义价值：
- 所有 `__` 前缀的符号
- `_`+大写字母开头的内部符号
- 分配器相关：`allocator`、`allocator_traits` 等
- 类型萃取：`is_*`、`has_*`、`remove_*`、`add_*`、`enable_if`、`conditional` 等
- 迭代器辅助：`iterator_traits`、`reverse_iterator`、`__normal_iterator` 等
- 编译内建：`initializer_list`、`numeric_limits`、`unary_function`、`binary_function`
- 字符特性/本地化：`char_traits`、`ctype`、`ctype_byname`、`codecvt`、`fpos`

#### Scenario: 分配器细节过滤
- GIVEN 模板实例 `allocator<FileMetaData*>`
- WHEN 处理该节点
- THEN SHALL 被判定为 detail 模板并丢弃

#### Scenario: 类型萃取过滤
- GIVEN 模板实例 `is_same<int, long>`
- WHEN 处理该节点
- THEN SHALL 被丢弃，不创建符号

### Requirement: 模板实例保留规则
系统 SHALL 仅保留以下两类模板实例：①项目自定义模板的实例化；②知名 STL 容器/工具的实例化且参数中含项目类型。纯内置类型参数的 STL 实例、STL 内部 detail 模板的实例不保留。
知名 STL 容器/工具包括：`vector`、`map`、`set`、`deque`、`pair`、`unique_ptr`、`shared_ptr`、`string`、`basic_string`、`function`、`unordered_map`、`unordered_set`。

#### Scenario: 项目类型参数 STL 容器保留
- GIVEN 模板实例 `vector<FileMetaData*>`，参数类型是项目自定义类型
- WHEN 检查模板参数
- THEN SHALL 返回 `true`，该 CTS 节点被保留
- AND 建立 `uses_type:` 边指向 `FileMetaData`

#### Scenario: 纯内置类型参数不保留
- GIVEN 模板实例 `vector<int>`，参数 `int` 是内置类型
- WHEN 检查模板参数
- THEN SHALL 返回 `false`，该 CTS 节点被丢弃

### Requirement: overrides 边通过继承链检测
系统 SHALL 通过继承链检测 `overrides` 边。从派生类方法出发，逐层向上检查基类是否有同名虚函数。

#### Scenario: 继承链检测
- GIVEN `ConsoleLogger` 继承自 `Logger`
- AND `ConsoleLogger::log` 使用 `override` 关键字
- WHEN 索引器处理 CXXMethodDecl
- THEN 从 `ConsoleLogger` 的基类列表向上查找
- AND 若 `Logger::log` 是虚函数，则建立 `overrides:` 边

### Requirement: 系统头文件中的 CXXRecordDecl 特殊处理
对于无 `loc.file` 的 CXXRecordDecl/RecordDecl/EnumDecl 节点，系统 SHALL 使用 `includedFrom` 路径辅助判断。仅当节点为知名 STL 容器且被项目类型参数实例化时保留，其余 SHALL 丢弃。

#### Scenario: 系统类丢弃
- GIVEN 系统头文件中的 `std::__shared_ptr` 类
- WHEN filter 处理该 CXXRecordDecl 节点
- THEN 非知名 STL 容器 → SHALL 被丢弃

#### Scenario: 项目类保留
- GIVEN 项目头文件中的 `class DB` 类
- WHEN filter 处理该 CXXRecordDecl 节点
- THEN `includedFrom.file` 指向项目 `.cc` 文件
- AND 该节点 SHALL 通过 filter 到达 Rust 层

### Requirement: 外部模板实例成员豁免
系统 SHALL 为被保留的外部模板实例（含项目类型参数的 ClassTemplateSpecializationDecl）的直接成员提供过滤豁免。当 Rust 层处理函数体内的 CXXMemberCallExpr 或 MemberExpr 时，若目标成员属于一个已知的被保留模板实例，SHALL 将该成员视为可创建符号的节点（不被系统符号过滤丢弃）。此豁免仅适用于该成员的 Decl/Type 节点，不波及其他不相关的系统符号。

#### Scenario: push_back 豁免
- GIVEN `vector<Record*>` 已被保留为模板实例（因参数含项目类型）
- AND 项目函数体中有 `records.push_back(r)` 的 CXXMemberCallExpr
- WHEN 处理到 `push_back` 的 CXXMethodDecl（在系统头文件中）
- THEN 该 CXXMethodDecl SHALL 被豁免——因属于 `vector<Record*>` 而通过过滤
- AND SHALL 被 Rust 层创建为桥接符号 `vector<Record*>::push_back`
