# Delta for clang-subprocess-parser

## MODIFIED Requirements

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
