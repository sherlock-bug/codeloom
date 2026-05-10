# Tasks

## 1. 枚举值使用边
- [x] 1.1 qualified_identifier 节点检测（EnumName::Value 及 ClassName::EnumName::Value）
- [x] 1.2 查符号表确认 target 为 enum_value 种类
- [x] 1.3 创建 `uses:EnumName::Value` 边

## 2. 全局/静态变量引用边
- [x] 2.1 identifier 节点检测，排除本地变量和 C++ 关键字
- [x] 2.2 修复 extract_decl 中 declarator 名字含初始化值（`x = 0`）的问题
- [x] 2.3 查符号表确认 target 为 global/static_var 种类
- [x] 2.4 创建 `references:var_name` 边

## 3. 字符串字面量归属边
- [x] 3.1 调换 collect_all_string_literals 到 walk_children 之前执行
- [x] 3.2 在函数体 walk 完成时用文本子串匹配 string_literal 符号
- [x] 3.3 创建 `uses:string_content` 边

## 4. 测试
- [x] 4.1 tests/fixtures/symbol_usage_edges/usage_test.h — 三种场景全覆盖
- [x] 4.2 集成测试验证枚举值/全局变量/字符串字面量三种边
- [x] 4.3 cargo test: 76 passed (66 unit + 10 integration)

## 5. 验证
- [x] 5.1 SQL 验证: 13 uses + 2 references + 其他边全部正确
- [ ] 5.2 对 leveldb 重建索引验证（可选）
