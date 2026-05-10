# Proposal: 符号使用边扩展

## Why
当前边体系覆盖了调用、继承、包含等结构性关系，但缺少三个关键的「使用」维度：函数对枚举值的引用、函数对全局/静态变量的读写、字符串字面量与函数的归属。搜索「哪些函数用了 `Status::ERROR`」或「`g_debug_mode` 被谁读取」无法得到答案。

## What Changes
在 C++ 索引器的函数体扫描环节，新增三类使用边：
1. **枚举值使用边**：函数体中出现 `EnumName::Value` 时创建 `uses:EnumName::Value` 边
2. **全局/静态变量引用边**：函数体中引用非本地声明的变量名，且该变量为 `global` 或 `static_var` 符号时，创建 `references:var_name` 边
3. **字符串字面量归属边**：函数体中出现的字符串字面量，创建 `uses:string_literal_content` 边关联到函数

## Capabilities

### New Capabilities
- `enum-value-usage`: 追踪函数对枚举值的使用，创建 uses 边
- `global-variable-references`: 追踪函数对全局/静态变量的引用，创建 references 边
- `string-literal-references`: 追踪函数体中字符串字面量，关联到函数

## Impact
- 影响范围：`src/indexer/queries/cpp.rs` 的函数体扫描逻辑
- 新增边不可逆：索引后旧数据库无这些边，需重建索引
- 存量 test 和 spec 不受影响（仅新增边类型）
