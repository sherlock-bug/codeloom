# Delta for overload-disambiguation

## ADDED Requirements

### Requirement: 调用边按参数个数区分重载
`extract_calls_with_types()` SHALL 解析 `call_expression` 的 `arguments` 子节点，按参数个数在边标签中附加 `(N)` 后缀。

#### Scenario: 两参数调用
- GIVEN 调用 `func(1, "hello")` 有 2 个实参
- WHEN 解析该调用
- THEN 生成的调用边 SHALL 为 `calls:func(2)`

#### Scenario: 无参数调用
- GIVEN 调用 `getValue()` 无实参
- WHEN 解析该调用
- THEN 生成的调用边 SHALL 为 `calls:getValue(0)`

#### Scenario: 成员调用也附加参数计数
- GIVEN `obj->method(a, b, c)` 有 3 个实参
- WHEN 解析该调用
- THEN 生成的调用边 SHALL 为 `calls:ClassName::method(3)`

### Requirement: resolve_target 支持参数计数后缀
`resolve_target()` SHALL 在匹配符号名时支持 `func(N)` 格式。首先精确匹配完整名称，然后降级到 LIKE `%::func(N)`，最后降级到 LIKE `%::func`（忽略参数计数）。

#### Scenario: 精确匹配两参数重载
- GIVEN 符号表中有 `MyClass::method(int, const char*)` 和 `MyClass::method(int)`
- WHEN 调用 `obj->method(1, "x")` 生成 `calls:MyClass::method(2)`
- THEN `resolve_target` SHALL 优先匹配签名中参数个数为 2 的版本

## REMOVED Requirements
- 无
