# Tasks

## 1. 字符串字面量提取

- [ ] 1.1 在 `walk_children` 中增加 `string_literal`、`raw_string_literal`、`concatenated_string`、`system_lib_string` 四个 match arm
- [ ] 1.2 实现 `extract_string_literal()` — 提取字符串内容（去引号）为名称，收集前置注释，创建 `kind: "string_literal"` 符号

## 2. 测试与验证

- [ ] 2.1 `cargo test` 全量通过
- [ ] 2.2 在测试仓库上验证字符串搜索：`codeloom search "/api"` 返回预期结果
- [ ] 2.3 验证注释收集：带 `///` 注释的字符串符号有非空 `doc_comment`

## 3. 文档

- [ ] 3.1 更新 README.md — 在符号类型表中增加 `string_literal`
