# Delta for enum-value-usage

## ADDED Requirements

### Requirement: 枚举值使用边提取
系统 SHALL 在索引函数体时检测 `EnumName::Value` 形式的枚举值引用，并创建 `uses:EnumName::Value` 边关联函数与枚举值符号。

#### Scenario: 函数返回枚举值
- GIVEN C++ 函数体包含 `return Status::OK;`
- WHEN 索引该文件
- THEN 创建一条 `uses:Status::OK` 边，source 为该函数符号，target 为 `Status::OK` 的 enum_value 符号

#### Scenario: 函数参数使用枚举值
- GIVEN C++ 函数调用 `set_mode(Mode::Debug)`
- WHEN 索引该文件
- THEN 创建 `uses:Mode::Debug` 边关联到调用函数

#### Scenario: switch 语句 case 枚举值
- GIVEN C++ 函数包含 `switch(x) { case Color::Red: ... }`
- WHEN 索引该文件
- THEN 创建 `uses:Color::Red` 边关联到该函数

#### Scenario: 非枚举值不创建边
- GIVEN C++ 代码 `Namespace::Function()` 但 `Namespace::Function` 不是 enum_value
- WHEN 索引该文件
- THEN 不创建 uses 边（正确的函数调用走已有的 calls 边）

### Requirement: 枚举值使用边可搜索
系统 SHALL 支持通过边类型 `uses:EnumName::Value` 查询哪些函数使用了指定枚举值。

#### Scenario: 搜索使用特定枚举值的函数
- GIVEN 已索引的 repo 中有函数 `handle_error` 使用了 `ErrorCode::TIMEOUT`
- WHEN 查询边的 target 为 `ErrorCode::TIMEOUT` 且 edge_type 前缀为 `uses:ErrorCode`
- THEN 返回 `handle_error` 函数符号
