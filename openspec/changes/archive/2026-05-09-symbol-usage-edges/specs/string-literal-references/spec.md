# Delta for string-literal-references

## ADDED Requirements

### Requirement: 字符串字面量归属边提取
系统 SHALL 在索引函数体时检测字符串字面量，并创建 `uses:string_literal_content` 边关联函数与对应的 string_literal 符号。

#### Scenario: 函数体包含字符串字面量
- GIVEN 函数体包含 `LOG("initialization complete")`
- WHEN 索引该文件且 "initialization complete" 已作为 string_literal 符号入库
- THEN 创建 `uses:initialization complete` 边，source 为函数符号，target 为该 string_literal 符号

#### Scenario: 同一个字面量被多个函数使用
- GIVEN 两个不同函数都使用了字符串 `"timeout"`
- WHEN 索引该文件
- THEN 创建两条 `uses:timeout` 边，分别关联到两个函数

#### Scenario: 空字符串不创建边
- GIVEN 函数体包含 `""`
- WHEN 索引该文件
- THEN 不创建 uses 边（空字面量已跳过，无对应 string_literal 符号）

#### Scenario: 字面量未匹配时不创建边
- GIVEN 函数体包含 `"some string"` 但该字面量因去重或其他原因未入库
- WHEN 索引该文件
- THEN 不创建 uses 边（无 target 符号可关联）

### Requirement: 字符串字面量归属边可搜索
系统 SHALL 支持通过边查询哪些函数使用了指定字符串字面量。

#### Scenario: 搜索使用特定字面量的函数
- GIVEN 已索引的 repo 中函数 `init` 使用了字符串 `"hello world"`
- WHEN 查询 target 为 `"hello world"` 的 string_literal 符号且 edge_type 前缀为 `uses:hello world`
- THEN 返回 `init` 函数符号
