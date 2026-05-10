# fix-call-graph-like-fallback

## Purpose
CodeLoom call_graph LIKE fallback 修复：排除非函数类型符号（string_literal/enum_value），确保 auto-match 只匹配函数和方法。

## Requirements

### Requirement: call_graph 符号匹配排除非函数类型
系统 SHALL 在 call_graph 的 LIKE 降级查询中排除 `kind = 'string_literal'` 和 `kind = 'enum_value'` 类型符号，确保 auto-match 只匹配函数/方法。

#### Scenario: LIKE 匹配到正确函数
- GIVEN 查询 `DBImpl::Get`，符号表中有 `DBImpl::Get`（method）和 `FLAGS_benchmarks = "..."`（string_literal）
- WHEN 调用 `codeloom_get_call_graph(name="DBImpl::Get")`
- THEN auto-match 返回 `DBImpl::Get` 而非 `FLAGS_benchmarks`

#### Scenario: 无匹配时正常降级
- GIVEN 查询不存在的符号 `NonexistentFunc`
- WHEN 调用 `codeloom_get_call_graph`
- THEN 返回 "Symbol 'NonexistentFunc' not found"
