# Delta for auto-cross-function

## ADDED Requirements

### Requirement: auto 变量通过 returns 边推断类型
当 `try_resolve_auto()` 遇到 `auto x = func()` 形式（右侧为 call_expression）时，系统 SHALL 在当前文件符号表和已索引全局符号中查找 `func` 的 `returns` 边，将其返回类型作为 `x` 的类型。

#### Scenario: 同文件函数返回类型
- GIVEN 同文件内有 `Iterator* NewIterator() { return new DBIter(...); }` 且有 `returns:Iterator` 边
- WHEN 解析 `auto iter = NewIterator();`
- THEN `iter` 的类型 SHALL 解析为 `Iterator`

#### Scenario: 跨文件函数返回类型
- GIVEN 已索引的 `GetEnv()` 函数有 `returns:Env` 边
- WHEN 解析 `auto env = GetEnv();`
- THEN `env` 的类型 SHALL 解析为 `Env`

#### Scenario: 未查到返回类型时降级
- GIVEN `auto x = UnknownFunc()` 无法在任何位置找到 `returns` 边
- WHEN 解析该声明
- THEN `x` 的类型 SHALL 保持为 `"auto"`

### Requirement: 只追溯一层
`try_resolve_auto()` SHALL 不跨多级 `auto` 追溯类型。`auto y = x->bar()` 中 `x` 为已解析的 `auto` 变量时，不做二次推断。

#### Scenario: 单层 auto 解析成功
- GIVEN `auto iter = NewIterator();`
- WHEN `iter->Valid()` 被解析
- THEN `iter` 的已解析类型为 `Iterator`，调用 SHALL 生成 `calls:Iterator::Valid`

## REMOVED Requirements
- 无
