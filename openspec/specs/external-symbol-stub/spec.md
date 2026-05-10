# external-symbol-stub

## Purpose
CodeLoom SHALL 为项目外部符号创建存根记录，通过白名单机制区分项目符号和外部符号，确保跨文件边引用的一致性。

## Requirements

### Requirement: 外部符号识别
系统 SHALL 在解析 Clang AST 时，通过 Decl 所在文件是否在项目 compile_commands.json 覆盖范围内，判定符号是否外部。

#### Scenario: 判定外部符号
- GIVEN 本项目中 `foo()` 调用了 `printf()`，`printf` 声明来自系统头文件不在项目范围内
- WHEN 解析 CallExpr→DeclRefExpr 检查 Decl 所在文件
- THEN 判定 printf 为外部符号（is_external=TRUE）

#### Scenario: 判定内部符号
- GIVEN 本项目中 `foo()` 调用了同项目 `bar()`，`bar` 声明在 `src/bar.h`
- WHEN 解析 DeclRefExpr 检查 Decl 所在文件
- THEN 判定 bar 为内部符号（is_external=FALSE）

### Requirement: 外部符号列标识
系统 SHALL 用 `is_external` 列标记外部符号，复用 `kind` 存储 Clang 原始类型。

#### Scenario: 外部函数
- GIVEN 检测到 `std::find` 的外部调用
- THEN 创建 kind=function、name="std::find"、namespace="std"、is_external=1、file_path="" 的符号

#### Scenario: 外部类
- GIVEN 参数类型为 `boost::asio::io_context`
- THEN 创建 kind=class、name="boost::asio::io_context"、namespace="boost::asio"、is_external=1 的符号

### Requirement: 外部符号去重
系统 SHALL 按 (name, namespace, kind) 对外部符号去重。

#### Scenario: 跨文件引用去重
- GIVEN `a.cpp` 和 `b.cpp` 都引用了 `std::string`
- WHEN `a.cpp` 创建了 kind=class、is_external=1 的 std::string，`b.cpp` 再次遇到
- THEN 查找已有 (name="std::string", namespace="std", kind="class") 并复用
