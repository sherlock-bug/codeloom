# Delta for clang-subprocess-parser

## ADDED Requirements

### Requirement: 子进程调用 Clang 解析 C/C++ 源文件
系统 SHALL 通过子进程调用系统 PATH 上的 `clang` 命令，使用 `-fsyntax-only -ast-dump=json` 参数解析 C/C++ 源文件，从 JSON AST 输出中提取符号和关系边。

#### Scenario: 成功解析单个翻译单元
- GIVEN 系统 PATH 上存在 clang 19.1.7，且有一个包含函数定义的 C++ 源文件 `main.cpp`
- WHEN 系统对该文件执行 `clang -fsyntax-only -ast-dump=json -- main.cpp`
- THEN 从 stdout 获取完整 JSON AST，解析出源文件中所有 FunctionDecl、CXXRecordDecl 等声明节点

#### Scenario: clang 命令不可用
- GIVEN 系统 PATH 上不存在 `clang` 命令
- WHEN 系统尝试调用 clang 子进程
- THEN 返回明确的错误提示："未找到 clang 命令，请安装 Clang 15+ 并确保在 PATH 中"

#### Scenario: 源文件编译失败
- GIVEN 源文件存在语法错误，clang 子进程返回非零 exit code
- WHEN 系统尝试解析该文件
- THEN 记录该文件解析失败的错误信息（文件名 + stderr），跳过该文件继续处理其他文件

### Requirement: 流式解析大型 AST JSON
系统 SHALL 使用流式 JSON 解析器处理 clang 输出的 AST JSON，避免将整个 JSON 树加载到内存中，以支持大型项目（单文件 AST JSON 可达 50MB+）。

#### Scenario: 大型 JSON 不导致 OOM
- GIVEN 一个包含 5000+ 行的大型 C++ 源文件，clang 输出的 AST JSON 超过 50MB
- WHEN 系统流式解析该 JSON
- THEN 内存峰值不超过解析小文件时的 3 倍，不会因 OOM 崩溃

### Requirement: 从 AST 提取函数声明和定义
系统 SHALL 从 Clang AST 中识别 FunctionDecl 和 CXXMethodDecl 节点，提取函数名、签名、返回类型、参数列表、所在命名空间、是否为定义（有函数体），并据此创建 function/method 符号。

#### Scenario: 提取普通函数定义
- GIVEN 源文件包含 `int add(int a, int b) { return a + b; }`
- WHEN 解析对应的 FunctionDecl 节点
- THEN 创建 kind=function、name=add、signature="int (int, int)"、is_definition=TRUE 的符号

#### Scenario: 区分声明与定义并合并
- GIVEN 头文件包含 `int add(int a, int b);`（仅有声明，附带注释 "/// 加法函数"），源文件包含 `int add(int a, int b) { return a + b; }`（定义，附带注释 "/// 实现加法"）
- WHEN 先解析声明创建 is_definition=FALSE 的符号，后解析定义时发现 name+namespace+signature 匹配
- THEN 更新同一符号：is_definition=TRUE，file_path/line 更新为定义位置，doc_comment="/// 加法函数\n/// 实现加法"（声明注释在前，定义注释在后）

### Requirement: 从 AST 提取类/结构体声明
系统 SHALL 从 Clang AST 中识别 CXXRecordDecl 节点，提取类名、继承关系、成员函数列表、成员变量列表、访问控制（public/private/protected），并据此创建 class/struct 符号。

#### Scenario: 提取带继承的类
- GIVEN 源文件包含 `class Dog : public Animal { void bark(); };`
- WHEN 解析对应的 CXXRecordDecl 节点
- THEN 创建 kind=class、name=Dog 的符号，并创建 Dog → Animal 的 inherits: 边

### Requirement: 从 AST 提取调用关系
系统 SHALL 从 Clang AST 的 CallExpr 和 DeclRefExpr 节点中提取函数调用关系，为每个调用创建 edges 表中的边。

#### Scenario: 提取直接调用
- GIVEN 函数 `foo()` 内部调用了 `bar()`
- WHEN 解析 foo 对应的 FunctionDecl 子节点中的 CallExpr→DeclRefExpr
- THEN 创建 source=foo、target=bar、edge_type="calls:bar" 的边

#### Scenario: 调用外部符号
- GIVEN 函数 `foo()` 内部调用了 `std::vector::push_back`
- WHEN 解析该 CallExpr，发现 DeclRefExpr 指向的声明不在本项目源文件中
- THEN 创建 external_symbol stub 节点（name="std::vector::push_back"），创建 calls_external:std::vector::push_back 边

### Requirement: 区分 C/C++ 语言路由
系统 SHALL 仅对 C/C++ 源文件使用 Clang 子进程解析，其他语言（Python、Java 等）保持现有 tree-sitter 解析不变。

#### Scenario: C++ 文件走 Clang
- GIVEN 待索引文件列表包含 `main.cpp` 和 `helper.h`
- WHEN `indexer/mod.rs` 进行语言路由
- THEN `main.cpp` 和 `helper.h` 路由到 `indexer/clang/` 模块

#### Scenario: Python 文件不走 Clang
- GIVEN 待索引文件列表包含 `script.py`
- WHEN `indexer/mod.rs` 进行语言路由
- THEN `script.py` 路由到现有 tree-sitter Python 解析器
