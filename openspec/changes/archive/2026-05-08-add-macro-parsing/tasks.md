# Tasks

## 1. 宏符号提取

- [x] 1.1 在 `walk_children` 中增加 `preproc_def` 和 `preproc_function_def` match arm，调用 `extract_macro()`
- [x] 1.2 实现 `extract_macro()` — 提取宏名称 + 定义，创建 `kind: "macro"` 符号
- [x] 1.3 实现 `is_noise_macro()` — 按名称模式过滤 include guard、平台宏、编译器内置宏

## 2. 条件编译块遍历

- [x] 2.1 在 `walk_children` 中增加 `preproc_if`、`preproc_ifdef`、`preproc_else` match arm，直接递归遍历子节点（不通过 body 字段）

## 3. 测试与验证

- [x] 3.1 `cargo test` 全量 75/75 通过
- [x] 3.2 leveldb 验证：确认噪音宏被正确过滤，有语义宏正常提取
- [x] 3.3 mactest 仓库验证：`MY_MACRO`、`MY_FUNC` 等 macro 符号搜索正常

## 4. 文档

- [x] 4.1 更新 README.md — 无需更新（宏解析为内部索引增强，不改变对外接口）
