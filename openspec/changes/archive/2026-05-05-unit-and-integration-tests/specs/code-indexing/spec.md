# Delta for code-indexing

## ADDED Requirements

### Requirement: tree-sitter 解析自动化验证
系统 SHALL 提供自动化测试验证 tree-sitter 语言检测和解析器创建的正确性。

#### Scenario: detect_language 正确识别文件类型
- GIVEN 文件扩展名 .cpp / .py / .java / .ts / .go
- WHEN 调用 detect_language()
- THEN 返回对应的语言标识

#### Scenario: create_parser 创建有效解析器
- GIVEN 语言标识 "cpp"
- WHEN 调用 create_parser("cpp")
- THEN 返回 Some(Parser)

### Requirement: git 集成自动化验证
系统 SHALL 提供自动化测试验证 git 分支检测功能。

#### Scenario: current_branch 返回当前分支
- GIVEN 当前目录是 git 仓库
- WHEN 调用 git::current_branch()
- THEN 返回当前分支名（非空）
