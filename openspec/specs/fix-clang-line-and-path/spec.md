# fix-clang-line-and-path Specification

## Purpose
TBD - created by archiving change fix-clang-line-and-path. Update Purpose after archive.
## Requirements
### Requirement: 行号从 range.offset 换算
系统 SHALL 使用 Clang AST 中 `range.begin.offset` 文件字节偏移，通过扫描源文件换行符将 offset 换算为行号，而非从始终不存在的 `range.begin.line` 字段读取。

#### Scenario: 字段声明行号
- GIVEN 源文件第 20 行包含字段声明 `int counter;`
- WHEN 解析对应的 FieldDecl 节点
- THEN symbol.line_start SHALL 为 20

#### Scenario: 多行函数体结束行号
- GIVEN 函数定义从第 5 行到第 25 行
- WHEN 解析对应的 FunctionDecl 节点
- THEN symbol.line_end SHALL 为 25

#### Scenario: 行号缓存不重复扫描
- GIVEN 同一个源文件被多次引用（如通过 #include 包含）
- WHEN 每次引用时都需换算 offset 成行号
- THEN 换行符偏移表 SHALL 只扫描一次，后续查询用缓存

### Requirement: 头文件符号路径正确处理
系统 SHALL 正确识别头文件中符号的文件路径，确保 `file_path` 为绝对路径且能被 `is_project_file()` 正确匹配。

#### Scenario: 头文件符号路径绝对化
- GIVEN Clang AST 输出中头文件路径为相对路径（如 `db/version_set.h`）
- WHEN Python filter 传播到子节点
- THEN 子节点的 `loc.file` SHALL 为绝对路径（如 `/mnt/d/code/leveldb/db/version_set.h`）

#### Scenario: 防御性路径归一化
- GIVEN Rust `is_project_file()` 接收到相对路径
- WHEN 进行 `startswith(project_root)` 检查前
- THEN 系统 SHALL 将相对路径拼上 `project_root` 转绝对后再比较

### Requirement: 端到端集成测试
系统 SHALL 提供手写 C++ fixture 的端到端集成测试，验证行号和路径修复的正确性，确保核心质量门禁。

#### Scenario: fixture 验证行号和路径
- GIVEN tests/fixtures/line_and_path/ 目录下包含手写 C++ 文件
- AND 该文件含至少一个类、一个方法、以及 `#include` 引用的头文件
- WHEN 运行集成测试（`cargo test --test integration test_clang_line_and_path`）
- THEN 验证所有符号的 `line_start` 不为零
- AND 验证头文件类符号的 `is_external` 为 false
- AND 验证头文件类符号的 `file_path` 为绝对路径且指向头文件
- AND 验证边（param_type、calls 等）已正确提取

