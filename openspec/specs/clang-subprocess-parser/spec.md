# clang-subprocess-parser

## Purpose
CodeLoom 使用 Clang 子进程替代 tree-sitter 解析 C/C++ 代码，通过管道 + Python 过滤器剥离系统头文件和函数体，产出精简 JSON 供 Rust 侧提取符号和边。
## Requirements
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
系统 SHALL 从 Clang AST 中识别 FunctionDecl 和 CXXMethodDecl 节点，提取函数名、签名、返回类型、参数列表、所在命名空间、是否为定义（有函数体），并据此创建 function/method 符号，行号 SHALL 不为零。

#### Scenario: 函数声明行号正确
- GIVEN 源文件第 10 行定义函数 `int add(int a, int b) { return a + b; }`
- WHEN 解析对应的 FunctionDecl 节点
- THEN 创建的 symbol.line_start SHALL 等于 10
- AND line_end SHALL 大于等于 10

### Requirement: 从 AST 提取类/结构体声明
系统 SHALL 从 Clang AST 中识别 CXXRecordDecl 节点，提取类名、继承关系、成员函数列表、成员变量列表、访问控制（public/private/protected），并据此创建 class/struct 符号。头文件中定义的类 SHALL 被正确识别为项目内部符号（is_external=false），并正常提取其调用关系等边。

#### Scenario: 头文件类符号非外部
- GIVEN 头文件 `include/leveldb/status.h` 中定义类 `Status`
- AND 该头文件被翻译单元 `db/version_set.cc` 通过 `#include` 引用
- WHEN Clang 索引器解析 `db/version_set.cc` 后提取符号
- THEN 类 `Status` 的 symbol.is_external SHALL 为 false
- AND symbol.file_path SHALL 指向 `include/leveldb/status.h`

#### Scenario: 头文件类符号行号正确
- GIVEN 头文件第 50 行定义 `class Compaction { ... };`
- WHEN 解析对应的 CXXRecordDecl 节点
- THEN 创建的 symbol.line_start SHALL 等于 50

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

### Requirement: 前向声明类符号过滤
Clang 索引器在处理 CXXRecordDecl 和 ClassTemplateDecl 节点时，SHALL 检查 `completeDefinition` 字段。没有该字段或字段值为 false 的节点 SHALL 被跳过，不生成符号。

#### Scenario: 前向声明不生成 class 节点
- GIVEN 一个 C++ 翻译单元中包含 `class Foo;` 前向声明和 `class Foo { ... };` 定义
- WHEN Clang 解析器提取符号
- THEN 仅定义处的 CXXRecordDecl 生成 class 节点
- AND 前向声明处的 CXXRecordDecl SHALL 被跳过

#### Scenario: 多文件前向声明不重复
- GIVEN 类 `Foo` 定义在 `foo.h`，同时前向声明在 `bar.h` 和 `baz.h`
- WHEN 索引整个项目
- THEN 数据库中 SHALL 只存在一个 `Foo` 的 class 节点
- AND 该节点的 file_path SHALL 指向 `foo.h`

#### Scenario: struct 前向声明同样处理
- GIVEN `struct Bar;` 前向声明
- WHEN Clang 解析器处理该节点
- THEN SHALL 被跳过，与 class 的前向声明行为一致

#
